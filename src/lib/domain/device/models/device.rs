use uuid::Uuid;

use derive_more::{Display, From};
use thiserror::Error;

/// Represents always valid device identifier.
#[derive(Display, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(Uuid);

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0} is not a valid IDFA")]
pub struct DeviceIdError(String);
impl DeviceId {
    pub fn new(raw_idfa: &str) -> Result<Self, DeviceIdError> {
        match Uuid::try_parse(raw_idfa) {
            Ok(uuid) => {
                if uuid.is_nil() {
                    Err(DeviceIdError(raw_idfa.to_string()))
                } else {
                    Ok(DeviceId(uuid))
                }
            }
            Err(_) => Err(DeviceIdError(raw_idfa.to_string())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Device {
    id: DeviceId,
}

impl Device {
    pub fn new(id: DeviceId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }
}

/// Data required by the domain to create a [Device].
#[derive(Clone, Debug, PartialEq, Eq, Hash, From)]
pub struct CreateDeviceRequest {
    id: DeviceId,
}

impl CreateDeviceRequest {
    pub fn new(id: DeviceId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &DeviceId {
        &self.id
    }
}

#[derive(Debug, Error)]
pub enum CreateDeviceError {
    #[error("device with id {id} already exists")]
    Duplicate { id: DeviceId },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[cfg(test)]
mod device_id_tests {
    use super::*;

    #[test]
    fn test_new_success() {
        let raw_idfa = "550e8400-e29b-41d4-a716-446655440000";
        let result = DeviceId::new(raw_idfa);
        let expected = Ok(DeviceId(Uuid::try_parse(raw_idfa).unwrap()));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_idfa_is_restricted() {
        let raw_idfa = "00000000-0000-0000-0000-000000000000";
        let result = DeviceId::new(raw_idfa);
        let expected = Err(DeviceIdError(raw_idfa.to_string()));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_new_invalid_idfa() {
        let raw_idfa = "abracadabra";
        let result = DeviceId::new(raw_idfa);
        let expected = Err(DeviceIdError(raw_idfa.to_string()));

        assert_eq!(result, expected);
    }
}
