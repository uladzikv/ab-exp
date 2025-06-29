use std::future::Future;

use crate::domain::device::models::device::CreateDeviceError;
#[allow(unused_imports)]
use crate::domain::device::models::device::DeviceId;
use crate::domain::device::models::device::{CreateDeviceRequest, Device};

/// `DeviceService` is the public API for the device domain.
pub trait DeviceService: Clone + Send + Sync + 'static {
    /// Asynchronously create a new [Device].
    ///
    /// # Errors
    ///
    /// - [CreateDeviceError::Duplicate] if an [Device] with the same [DeviceId] already exists.
    fn create_device(
        &self,
        req: &CreateDeviceRequest,
    ) -> impl Future<Output = Result<Device, CreateDeviceError>> + Send;
}

/// `DeviceRepository` represents a store of device data.
pub trait DeviceRepository: Send + Sync + Clone + 'static {
    /// Asynchronously persist a new [Device].
    ///
    /// # Errors
    ///
    /// - MUST return [CreateDeviceError::Duplicate] if an [Device] with the same [DeviceId]
    ///   already exists.
    fn create_device(
        &self,
        req: &CreateDeviceRequest,
    ) -> impl Future<Output = Result<Device, CreateDeviceError>> + Send;
}
