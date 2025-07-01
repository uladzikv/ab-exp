use uuid::Uuid;

use crate::domain::device::models::device::{Device, DeviceId, GetAllDevicesError};
use crate::domain::experiment::models::experiment::{
    CreateExperimentError, CreateExperimentRequest, DeviceExperiment, Experiment,
    FinishExperimentError, GetAllDeviceExperimentsError, GetAllExperimentsError,
    StaticticsExperiment, StatisticsVariant, StatisticsVariants, VariantData,
};
use crate::domain::experiment::ports::{ExperimentRepository, ExperimentService};

#[derive(Debug, Clone)]
pub struct Service<R: ExperimentRepository> {
    repo: R,
}

impl<R: ExperimentRepository> Service<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: ExperimentRepository> ExperimentService for Service<R> {
    async fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> Result<Uuid, CreateExperimentError> {
        self.repo.create_experiment(req).await
    }

    async fn get_all_experiments(&self) -> Result<Vec<Experiment>, GetAllExperimentsError> {
        self.repo.get_all_experiments().await
    }

    async fn get_all_device_participating_experiments(
        &self,
        id: &DeviceId,
    ) -> Result<Vec<DeviceExperiment>, GetAllDeviceExperimentsError> {
        self.repo.get_all_device_participating_experiments(id).await
    }

    async fn get_all_devices(&self) -> Result<Vec<Device>, GetAllDevicesError> {
        self.repo.get_all_devices().await
    }

    async fn get_statistics(
        &self,
        devices: Vec<Device>,
    ) -> Result<Vec<StaticticsExperiment>, GetAllExperimentsError> {
        let experiments = self.repo.get_all_experiments().await?;

        let experiments: Vec<StaticticsExperiment> = experiments
            .iter()
            .map(|exp| {
                let participants: Vec<&Device> = devices
                    .iter()
                    .filter(|dev| exp.created_at().cmp(dev.created_at()).is_ge())
                    .collect();

                let total_devices = participants.len();

                let variants_data: Vec<&VariantData> = participants
                    .iter()
                    .map(|p| {
                        exp.variants()
                            .assign_variant(format!("{}", p.id().to_owned().into_inner()).as_str())
                    })
                    .collect();

                let statistics_variants: Vec<StatisticsVariant> = exp
                    .variants()
                    .variants()
                    .iter()
                    .map(move |variant| {
                        let assigned_total_devices = variants_data
                            .iter()
                            .filter(|v| v.to_string() == variant.data().to_string())
                            .count();
                        let percentage_devices =
                            (assigned_total_devices as f64 / total_devices as f64) * 100.0;

                        StatisticsVariant::new(
                            variant.data().to_owned(),
                            assigned_total_devices,
                            percentage_devices,
                        )
                    })
                    .collect();

                let id = exp.id().to_owned();
                let name = exp.name().to_owned();
                let variants = StatisticsVariants::new(statistics_variants);

                StaticticsExperiment::new(id, name, total_devices, variants)
            })
            .collect();

        Ok(experiments)
    }

    async fn finish_experiment(&self, id: &Uuid) -> Result<Uuid, FinishExperimentError> {
        self.repo.finish_experiment(id).await
    }
}
