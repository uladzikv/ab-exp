use std::future::Future;

use uuid::Uuid;

use crate::domain::device::models::device::{Device, DeviceId, GetAllDevicesError};
#[allow(unused_imports)]
use crate::domain::experiment::models::experiment::ExperimentName;
use crate::domain::experiment::models::experiment::{
    CreateExperimentError, DeviceExperiment, FinishExperimentError, GetAllDeviceExperimentsError,
    GetAllExperimentsError, StaticticsExperiment,
};
use crate::domain::experiment::models::experiment::{CreateExperimentRequest, Experiment};

/// `ExperimentService` is the public API for the experiment domain.
pub trait ExperimentService: Clone + Send + Sync + 'static {
    fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> impl Future<Output = Result<Uuid, CreateExperimentError>> + Send;

    fn get_all_experiments(
        &self,
    ) -> impl Future<Output = Result<Vec<Experiment>, GetAllExperimentsError>> + Send;

    fn get_all_device_participating_experiments(
        &self,
        id: &DeviceId,
    ) -> impl Future<Output = Result<Vec<DeviceExperiment>, GetAllDeviceExperimentsError>> + Send;

    fn finish_experiment(
        &self,
        id: &Uuid,
    ) -> impl Future<Output = Result<Uuid, FinishExperimentError>> + Send;

    fn get_all_devices(
        &self,
    ) -> impl Future<Output = Result<Vec<Device>, GetAllDevicesError>> + Send;

    fn get_statistics(
        &self,
        devices: Vec<Device>,
    ) -> impl Future<Output = Result<Vec<StaticticsExperiment>, GetAllExperimentsError>> + Send;
}

/// `ExperimentRepository` represents a store of experiment data.
pub trait ExperimentRepository: Send + Sync + Clone + 'static {
    fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> impl Future<Output = Result<Uuid, CreateExperimentError>> + Send;

    fn get_all_devices(
        &self,
    ) -> impl Future<Output = Result<Vec<Device>, GetAllDevicesError>> + Send;

    fn get_all_experiments(
        &self,
    ) -> impl Future<Output = Result<Vec<Experiment>, GetAllExperimentsError>> + Send;

    fn get_all_device_participating_experiments(
        &self,
        id: &DeviceId,
    ) -> impl Future<Output = Result<Vec<DeviceExperiment>, GetAllDeviceExperimentsError>> + Send;

    fn finish_experiment(
        &self,
        id: &Uuid,
    ) -> impl Future<Output = Result<Uuid, FinishExperimentError>> + Send;
}
