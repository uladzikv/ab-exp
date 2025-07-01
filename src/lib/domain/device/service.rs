use crate::domain::device::models::device::CreateDeviceError;
use crate::domain::device::models::device::{CreateDeviceRequest, Device};
use crate::domain::device::ports::{DeviceRepository, DeviceService};

/// Canonical implementation of the [DeviceService] port, through which the device domain API is
/// consumed.
#[derive(Debug, Clone)]
pub struct Service<R: DeviceRepository> {
    repo: R,
}

impl<R: DeviceRepository> Service<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: DeviceRepository> DeviceService for Service<R> {
    async fn create_device(&self, req: &CreateDeviceRequest) -> Result<Device, CreateDeviceError> {
        self.repo.create_device(req).await
    }
}
