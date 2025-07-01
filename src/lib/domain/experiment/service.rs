use uuid::Uuid;

use crate::domain::device::models::device::DeviceId;
use crate::domain::experiment::models::experiment::{
    CreateExperimentError, CreateExperimentRequest, DeviceExperiment, Experiment,
    FinishExperimentError, GetAllDeviceExperimentsError, GetAllExperimentsError,
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

    async fn finish_experiment(&self, id: &Uuid) -> Result<Uuid, FinishExperimentError> {
        self.repo.finish_experiment(id).await
    }
}
