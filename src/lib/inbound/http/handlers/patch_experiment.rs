use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::domain::experiment::models::experiment::{
    DistributionSumError, FinishExperimentError, VariantDistributionInvalidError,
};
use crate::domain::experiment::models::experiment::{
    ExperimentNameEmptyError, VariantDataEmptyError,
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
    NotFound(String),
    Conflict(String),
    Unauthorized,
    Forbidden,
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        Self::InternalServerError(e.to_string())
    }
}

impl From<FinishExperimentError> for ApiError {
    fn from(e: FinishExperimentError) -> Self {
        match e {
            FinishExperimentError::NotFound { id } => {
                Self::NotFound(format!("experiment with id {} not found", id))
            }
            FinishExperimentError::Finished { id } => {
                Self::Conflict(format!("experiment with id {} is already finished", id))
            }
            FinishExperimentError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
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
            NotFound(message) => (
                StatusCode::NOT_FOUND,
                Json(ApiResponseBody::new_error(message)),
            )
                .into_response(),
            Conflict(message) => (
                StatusCode::CONFLICT,
                Json(ApiResponseBody::new_error(message)),
            )
                .into_response(),
            Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(ApiResponseBody::new_error("Unauthorized".to_string())),
            )
                .into_response(),
            Forbidden => (
                StatusCode::FORBIDDEN,
                Json(ApiResponseBody::new_error("Forbidden".to_string())),
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateExperimentRequestBody {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PatchExperimentResponseData {
    id: String,
}

impl From<&Uuid> for PatchExperimentResponseData {
    fn from(id: &Uuid) -> Self {
        Self { id: id.to_string() }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Variant {
    distribution: f64,
    data: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PatchExperimentHttpRequestBody {
    status: ExperimentStatusHttpRequest,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExperimentStatusHttpRequest {
    Finished,
}

#[derive(Debug, Clone, Error)]
enum ParseCreateExperimentHttpRequestError {
    #[error(transparent)]
    Name(#[from] ExperimentNameEmptyError),
    #[error(transparent)]
    VariantData(#[from] VariantDataEmptyError),
    #[error(transparent)]
    VariantDistribution(#[from] VariantDistributionInvalidError),
    #[error(transparent)]
    DistributionSum(#[from] DistributionSumError),
}

pub async fn patch_experiment<ES: ExperimentService>(
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    State(state): State<AppState<ES>>,
    Json(_): Json<PatchExperimentHttpRequestBody>,
) -> Result<ApiSuccess<PatchExperimentResponseData>, ApiError> {
    let auth_key = headers.get("Authorization").ok_or(ApiError::Unauthorized)?;

    match auth_key.to_str() {
        Ok(auth_key) => {
            if auth_key != state.auth_token {
                return Err(ApiError::Forbidden);
            }
        }
        Err(_) => return Err(ApiError::Unauthorized),
    }

    state
        .experiment_service
        .finish_experiment(&id)
        .await
        .map_err(ApiError::from)
        .map(|ref experiment| ApiSuccess::new(StatusCode::OK, experiment.into()))
}
