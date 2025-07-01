use std::future::Future;

#[allow(unused_imports)]
use crate::domain::device::models::device::DeviceId;
use crate::domain::device::models::device::{CreateDeviceError, GetDeviceByIdError};
use crate::domain::device::models::device::{CreateDeviceRequest, Device};

/// `DeviceService` is the public API for the device domain.
pub trait DeviceService: Clone + Send + Sync + 'static {
    fn create_device(
        &self,
        req: &CreateDeviceRequest,
    ) -> impl Future<Output = Result<Device, CreateDeviceError>> + Send;
}

/// `DeviceRepository` represents a store of device data.
pub trait DeviceRepository: Send + Sync + Clone + 'static {
    fn create_device(
        &self,
        req: &CreateDeviceRequest,
    ) -> impl Future<Output = Result<Device, CreateDeviceError>> + Send;

    fn get_device_by_id(
        &self,
        id: &DeviceId,
    ) -> impl Future<Output = Result<Device, GetDeviceByIdError>> + Send;
}
