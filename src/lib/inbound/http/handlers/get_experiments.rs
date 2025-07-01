use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::domain::device::models::device::{DeviceId, DeviceIdError};
use crate::domain::experiment::models::experiment::{
    DeviceExperiment, GetAllDeviceExperimentsError, GetAllExperimentsError,
};
use crate::domain::experiment::models::experiment::{Experiment, Variant as ExperimentVariant};
use crate::domain::experiment::ports::ExperimentService;
use crate::inbound::http::AppState;

#[derive(Debug, Clone)]
pub struct ApiSuccess<T: Serialize + PartialEq>(StatusCode, Json<ApiResponseBody<T>>);

impl<T> PartialEq for ApiSuccess<T>
where
    T: Serialize + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1.0 == other.1.0
    }
}

impl<T: Serialize + PartialEq> ApiSuccess<T> {
    fn new(status: StatusCode, data: T) -> Self {
        ApiSuccess(status, Json(ApiResponseBody::new(data)))
    }
}

impl<T: Serialize + PartialEq> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    InternalServerError(String),
    UnprocessableEntity(String),
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

impl From<GetAllExperimentsError> for ApiError {
    fn from(e: GetAllExperimentsError) -> Self {
        tracing::error!("{:?}", e);
        Self::InternalServerError("Internal server error".to_string())
    }
}

impl From<GetAllDeviceExperimentsError> for ApiError {
    fn from(e: GetAllDeviceExperimentsError) -> Self {
        tracing::error!("{:?}", e);
        Self::InternalServerError("Internal server error".to_string())
    }
}

impl From<DeviceIdError> for ApiError {
    fn from(e: DeviceIdError) -> Self {
        Self::UnprocessableEntity(e.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        use ApiError::*;

        match self {
            InternalServerError(e) => {
                tracing::error!("{}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponseBody::new_error(
                        "Internal server error".to_string(),
                    )),
                )
                    .into_response()
            }
            UnprocessableEntity(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ApiResponseBody::new_error(message)),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiResponseBody<T: Serialize + PartialEq> {
    data: T,
}

impl<T: Serialize + PartialEq> ApiResponseBody<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl ApiResponseBody<ApiErrorData> {
    pub fn new_error(message: String) -> Self {
        Self {
            data: ApiErrorData { message },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiErrorData {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ExperimentResponseData {
    id: String,
    name: String,
    variants: Vec<Variant>,
}

impl From<&Experiment> for ExperimentResponseData {
    fn from(experiment: &Experiment) -> Self {
        Self {
            id: experiment.id().to_string(),
            name: experiment.name().to_string(),
            variants: experiment
                .variants()
                .variants()
                .iter()
                .map(|variant| variant.into())
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AllExperimentsResponseData {
    experiments: Vec<ExperimentResponseData>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AllDeviceExperimentsResponseData {
    experiments: Vec<DeviceExperimentResponseData>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceExperimentResponseData {
    id: String,
    name: String,
    data: String,
}

impl From<&DeviceExperiment> for DeviceExperimentResponseData {
    fn from(experiment: &DeviceExperiment) -> Self {
        Self {
            id: experiment.id().to_string(),
            name: experiment.name().to_string(),
            data: experiment.data().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum GetAllExperimentsResponseData {
    Public(AllExperimentsResponseData),
    Device(AllDeviceExperimentsResponseData),
}

impl From<&Vec<Experiment>> for GetAllExperimentsResponseData {
    fn from(experiments: &Vec<Experiment>) -> Self {
        Self::Public(AllExperimentsResponseData {
            experiments: experiments
                .iter()
                .map(|experiment| experiment.into())
                .collect(),
        })
    }
}

impl From<&Vec<DeviceExperiment>> for GetAllExperimentsResponseData {
    fn from(experiments: &Vec<DeviceExperiment>) -> Self {
        Self::Device(AllDeviceExperimentsResponseData {
            experiments: experiments
                .iter()
                .map(|experiment| experiment.into())
                .collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Variant {
    distribution: f64,
    data: String,
}

impl From<&ExperimentVariant> for Variant {
    fn from(variant: &ExperimentVariant) -> Self {
        Self {
            distribution: variant.distribution().into_inner(),
            data: variant.data().to_string(),
        }
    }
}

pub async fn get_experiments<ES: ExperimentService>(
    headers: HeaderMap,
    State(state): State<AppState<ES>>,
) -> Result<ApiSuccess<GetAllExperimentsResponseData>, ApiError> {
    let device_id = headers
        .get(HeaderName::from_static("x-device-id"))
        .and_then(|v| v.to_str().ok());

    match device_id {
        Some(device_id) => {
            let device_id = DeviceId::new(device_id)?;

            state
                .experiment_service
                .get_all_device_participating_experiments(&device_id)
                .await
                .map_err(ApiError::from)
                .map(|ref experiments| ApiSuccess::new(StatusCode::OK, experiments.into()))
        }
        None => state
            .experiment_service
            .get_all_experiments()
            .await
            .map_err(ApiError::from)
            .map(|ref experiments| ApiSuccess::new(StatusCode::OK, experiments.into())),
    }
}
