use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::domain::device::models::device::{DeviceIdError, GetAllDevicesError};
use crate::domain::experiment::models::experiment::{
    DeviceExperiment, GetAllDeviceExperimentsError, GetAllExperimentsError, StaticticsExperiment,
    StatisticsVariant,
};
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

impl From<GetAllDevicesError> for ApiError {
    fn from(e: GetAllDevicesError) -> Self {
        tracing::error!("{:?}", e);
        Self::InternalServerError("Internal server error".to_string())
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
#[serde(rename_all = "camelCase")]
pub struct StatisticsExperimentResponseData {
    id: String,
    name: String,
    total_devices: usize,
    variants: Vec<Variant>,
}

impl From<&StaticticsExperiment> for StatisticsExperimentResponseData {
    fn from(experiment: &StaticticsExperiment) -> Self {
        Self {
            id: experiment.id().to_string(),
            name: experiment.name().to_string(),
            total_devices: experiment.total_devices(),
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
pub struct GetAllStatisticsExperimentsResponseData {
    experiments: Vec<StatisticsExperimentResponseData>,
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    data: String,
    total_devices: usize,
    percentage_devices: f64,
}

impl From<&StatisticsVariant> for Variant {
    fn from(variant: &StatisticsVariant) -> Self {
        Self {
            data: variant.data().to_string(),
            total_devices: variant.total_devices(),
            percentage_devices: variant.percentage_devices(),
        }
    }
}

impl From<&Vec<StaticticsExperiment>> for GetAllStatisticsExperimentsResponseData {
    fn from(experiments: &Vec<StaticticsExperiment>) -> Self {
        Self {
            experiments: experiments
                .iter()
                .map(|experiment| experiment.into())
                .collect(),
        }
    }
}

pub async fn get_statistics<ES: ExperimentService>(
    State(state): State<AppState<ES>>,
) -> Result<ApiSuccess<GetAllStatisticsExperimentsResponseData>, ApiError> {
    let devices = state
        .experiment_service
        .get_all_devices()
        .await
        .map_err(ApiError::from)?;

    state
        .experiment_service
        .get_statistics(devices)
        .await
        .map_err(ApiError::from)
        .map(|ref experiments| ApiSuccess::new(StatusCode::OK, experiments.into()))
}
