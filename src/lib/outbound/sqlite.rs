use std::str::FromStr;

use anyhow::{Context, anyhow};
use chrono::Utc;
use sqlx::{Executor, SqlitePool, Transaction};
use sqlx::{QueryBuilder, sqlite::SqliteConnectOptions};
use uuid::Uuid;

use crate::domain::device::models::device::{
    CreateDeviceError, CreateDeviceRequest, Device, DeviceId, GetAllDevicesError,
    GetDeviceByIdError,
};
use crate::domain::device::ports::DeviceRepository;
use crate::domain::experiment::models::experiment::{
    CreateExperimentError, CreateExperimentRequest, DeviceExperiment, Experiment, ExperimentName,
    ExperimentVariants, FinishExperimentError, GetAllDeviceExperimentsError,
    GetAllExperimentsError, Variant as ExperimentVariant, VariantData, VariantDistribution,
};
use crate::domain::experiment::ports::ExperimentRepository;

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    pub async fn new(path: &str) -> Result<Sqlite, anyhow::Error> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str(path)
                .with_context(|| format!("invalid database path {}", path))?
                .pragma("foreign_keys", "ON"),
        )
        .await
        .with_context(|| format!("failed to open database at {}", path))?;

        Ok(Sqlite { pool })
    }

    async fn save_experiment(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        name: &ExperimentName,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();
        let id_as_string = id.to_string();
        let name = &name.to_string();
        let now = Utc::now();

        let query = sqlx::query!(
            "INSERT INTO experiments (id, name, created_at) VALUES ($1, $2, $3)",
            id_as_string,
            name,
            now,
        );

        tx.execute(query).await?;

        Ok(id)
    }

    async fn save_experiment_variants(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        experiment_id: &Uuid,
        variants: &ExperimentVariants,
    ) -> Result<(), sqlx::Error> {
        let experiment_id = experiment_id.to_string();
        let variants = variants.variants();

        let mut query_builder = QueryBuilder::new(
            "INSERT INTO experiment_variants (id, experiment_id,  data, distribution) ",
        );

        let query = query_builder
            .push_values(variants, |mut b, v| {
                let id = Uuid::new_v4().to_string();
                let data = v.data().to_string();
                let distribution = v.distribution().into_inner();

                b.push_bind(id)
                    .push_bind(&experiment_id)
                    .push_bind(data)
                    .push_bind(distribution);
            })
            .build();

        tx.execute(query).await?;

        Ok(())
    }

    async fn save_device(
        &self,
        tx: &mut Transaction<'_, sqlx::Sqlite>,
        id: &DeviceId,
    ) -> Result<Device, sqlx::Error> {
        let id_as_string = id.to_string();
        let now = Utc::now();

        let query = sqlx::query!(
            "INSERT INTO devices (id, created_at) VALUES ($1, $2)",
            id_as_string,
            now,
        );

        tx.execute(query).await?;

        Ok(Device::new(id.clone(), now))
    }
}

impl DeviceRepository for Sqlite {
    async fn create_device(&self, req: &CreateDeviceRequest) -> Result<Device, CreateDeviceError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to start SQLite transaction")?;

        let device = self.save_device(&mut tx, req.id()).await.map_err(|e| {
            if is_primary_key_constraint_violation(&e) {
                CreateDeviceError::Duplicate {
                    id: req.id().clone(),
                }
            } else {
                anyhow!(e)
                    .context(format!("failed to save device with id {}", req.id()))
                    .into()
            }
        })?;

        tx.commit()
            .await
            .context("failed to commit SQLite transaction")?;

        Ok(device)
    }

    async fn get_device_by_id(&self, id: &DeviceId) -> Result<Device, GetDeviceByIdError> {
        let id_as_string = id.to_owned().into_inner().to_string();

        let device = sqlx::query!(
            "SELECT id, created_at FROM devices WHERE id = $1",
            id_as_string
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => GetDeviceByIdError::NotFound { id: id.to_owned() },
            _ => anyhow!(e)
                .context(format!("failed to get device with id {}", id))
                .into(),
        })?;

        let created_at = device
            .created_at
            .parse()
            .context("failed to parse created_at as DateTime<Utc>")?;

        let device = Device::new(id.to_owned(), created_at);

        Ok(device)
    }
}

impl ExperimentRepository for Sqlite {
    async fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> Result<Uuid, CreateExperimentError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .context("failed to start SQLite transaction")?;

        let id = self
            .save_experiment(&mut tx, req.name())
            .await
            .map_err(|e| {
                if is_unique_constraint_violation(&e) {
                    CreateExperimentError::Duplicate {
                        name: req.name().clone(),
                    }
                } else {
                    anyhow!(e)
                        .context(format!(
                            "failed to save experiment with name {:?}",
                            req.name()
                        ))
                        .into()
                }
            })?;

        self.save_experiment_variants(&mut tx, &id, req.variants())
            .await
            .map_err(|e| anyhow!(e).context("failed to save experiment variants"))?;

        tx.commit()
            .await
            .context("failed to commit SQLite transaction")?;

        Ok(id)
    }

    async fn get_all_experiments(&self) -> Result<Vec<Experiment>, GetAllExperimentsError> {
        let experiment_rows =
            sqlx::query!("SELECT id, name, created_at, finished_at FROM experiments")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| {
                    GetAllExperimentsError::Unknown(
                        anyhow!(e).context("failed to fetch experiments"),
                    )
                })?;

        let mut experiments = Vec::new();
        for row in experiment_rows {
            let id = Uuid::parse_str(&row.id).context("invalid UUID format")?;
            let name = ExperimentName::new(&row.name)?;
            let created_at = row
                .created_at
                .parse()
                .context("failed to parse created_at as DateTime<Utc>")?;
            let finished_at = row
                .finished_at
                .map(|f| {
                    f.parse()
                        .context("failed to parse finished_at as DateTime<Utc>")
                })
                .transpose()?;

            let id_str = id.to_string();
            let variant_rows = sqlx::query!(
                "SELECT data, distribution FROM experiment_variants WHERE experiment_id = $1",
                id_str
            )
            .fetch_all(&self.pool)
            .await
            .context("failed to fetch experiment variants")?;

            let variants = variant_rows
                .into_iter()
                .map(|v| {
                    let data = VariantData::new(&v.data)?;
                    let distribution = VariantDistribution::new(v.distribution)?;

                    Ok(ExperimentVariant::new(distribution, data))
                })
                .collect::<Result<Vec<_>, GetAllExperimentsError>>()?;

            let validated_variants = ExperimentVariants::new(variants).map_err(|e| {
                GetAllExperimentsError::Unknown(anyhow!(e).context("invalid experiment variants"))
            })?;

            let experiment = Experiment::new(id, name, validated_variants, created_at, finished_at);

            experiments.push(experiment);
        }

        Ok(experiments)
    }

    async fn get_all_device_participating_experiments(
        &self,
        device_id: &DeviceId,
    ) -> Result<Vec<DeviceExperiment>, GetAllDeviceExperimentsError> {
        let create_device_req = CreateDeviceRequest::new(device_id.to_owned());
        let device = self.create_device(&create_device_req).await;

        if device.is_ok() {
            return Ok(vec![]);
        }

        if let Err(CreateDeviceError::Unknown(e)) = device {
            return Err(GetAllDeviceExperimentsError::Unknown(
                anyhow!(e).context("failed to create device"),
            ));
        }

        let device = self.get_device_by_id(device_id).await.map_err(|e| {
            GetAllDeviceExperimentsError::Unknown(anyhow!(e).context("failed to get device"))
        })?;

        let experiments = self.get_all_experiments().await.map_err(|e| {
            GetAllDeviceExperimentsError::Unknown(
                anyhow!(e).context("failed to get all experiments"),
            )
        })?;

        let device_experiments = experiments
            .into_iter()
            .filter(|exp| {
                exp.created_at().cmp(device.created_at()).is_ge() && exp.finished_at().is_none()
            })
            .map(|exp| {
                let data = exp
                    .variants()
                    .assign_variant(format!("{}", device.id().to_owned().into_inner()).as_str());

                DeviceExperiment::new(*exp.id(), exp.name().to_owned(), data.to_owned())
            })
            .collect();

        Ok(device_experiments)
    }

    async fn get_all_devices(&self) -> Result<Vec<Device>, GetAllDevicesError> {
        let rows = sqlx::query!("SELECT * FROM devices")
            .fetch_all(&self.pool)
            .await
            .context("failed to fetch devices")?;

        let mut devices = Vec::new();
        for row in rows {
            let id = Uuid::parse_str(&row.id)
                .context("invalid UUID format")?
                .to_string();
            let created_at = row
                .created_at
                .parse()
                .context("failed to parse created_at as DateTime<Utc>")?;

            let device_id = DeviceId::new(&id).context("failed to create device ID")?;

            let device = Device::new(device_id, created_at);
            devices.push(device);
        }

        Ok(devices)
    }

    async fn finish_experiment(&self, id: &Uuid) -> Result<Uuid, FinishExperimentError> {
        let id_as_string = id.to_string();
        let now = Utc::now();

        sqlx::query!(
            "UPDATE experiments SET finished_at = $1 WHERE id = $2",
            now,
            id_as_string,
        )
        .execute(&self.pool)
        .await
        .context("failed to finish experiment")?;

        Ok(id.to_owned())
    }
}

const UNIQUE_CONSTRAINT_VIOLATION_CODE: &str = "2067";

fn is_unique_constraint_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            if code == UNIQUE_CONSTRAINT_VIOLATION_CODE {
                return true;
            }
        }
    }

    false
}

const PRIMARYKEY_CONSTRAINT_VIOLATION_CODE: &str = "1555";

fn is_primary_key_constraint_violation(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if let Some(code) = db_err.code() {
            if code == PRIMARYKEY_CONSTRAINT_VIOLATION_CODE {
                return true;
            }
        }
    }

    false
}
