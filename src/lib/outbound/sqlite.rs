use std::str::FromStr;

use anyhow::{Context, anyhow};
use sqlx::{Executor, SqlitePool, Transaction};
use sqlx::{QueryBuilder, sqlite::SqliteConnectOptions};
use uuid::Uuid;

use crate::domain::experiment::models::experiment::CreateExperimentError;
use crate::domain::experiment::models::experiment::{
    CreateExperimentRequest, Experiment, ExperimentName, ExperimentVariants,
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
        let now = chrono::Utc::now();

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
}

impl ExperimentRepository for Sqlite {
    async fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> Result<Experiment, CreateExperimentError> {
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

        Ok(Experiment::new(
            id,
            req.name().clone(),
            req.variants().clone(),
        ))
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
