use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::domain::experiment::models::experiment::{
    CreateExperimentError, DistributionSumError, ExperimentVariants, VariantData,
    VariantDistribution, VariantDistributionInvalidError,
};
use crate::domain::experiment::models::experiment::{
    CreateExperimentRequest, Experiment, ExperimentName, ExperimentNameEmptyError,
    Variant as ExperimentVariant, VariantDataEmptyError,
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

impl From<CreateExperimentError> for ApiError {
    fn from(e: CreateExperimentError) -> Self {
        match e {
            CreateExperimentError::Duplicate { name } => {
                Self::UnprocessableEntity(format!("experiment with name {} already exists", name))
            }
            CreateExperimentError::Unknown(cause) => {
                tracing::error!("{:?}\n{}", cause, cause.backtrace());
                Self::InternalServerError("Internal server error".to_string())
            }
        }
    }
}

impl From<ParseCreateExperimentHttpRequestError> for ApiError {
    fn from(e: ParseCreateExperimentHttpRequestError) -> Self {
        let message = match e {
            ParseCreateExperimentHttpRequestError::Name(_) => {
                "experiment name cannot be empty".to_string()
            }
            ParseCreateExperimentHttpRequestError::VariantData(_) => {
                "variant data cannot be empty".to_string()
            }
            ParseCreateExperimentHttpRequestError::VariantDistribution(cause) => {
                format!("{cause}")
            }
            ParseCreateExperimentHttpRequestError::DistributionSum(cause) => {
                format!("{cause}")
            }
        };

        Self::UnprocessableEntity(message)
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateExperimentRequestBody {
    name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CreateExperimentResponseData {
    id: String,
}

impl From<&Experiment> for CreateExperimentResponseData {
    fn from(experiment: &Experiment) -> Self {
        Self {
            id: experiment.id().to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Variant {
    distribution: f64,
    data: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CreateExperimentHttpRequestBody {
    name: String,
    variants: Vec<Variant>,
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

impl CreateExperimentHttpRequestBody {
    fn try_into_domain(
        self,
    ) -> Result<CreateExperimentRequest, ParseCreateExperimentHttpRequestError> {
        let name = ExperimentName::new(&self.name)?;
        let variants = &self
            .variants
            .iter()
            .map(|v| {
                let data = VariantData::new(&v.data)?;
                let distribution = VariantDistribution::new(v.distribution)?;

                Ok(ExperimentVariant::new(distribution, data))
            })
            .collect::<Result<Vec<ExperimentVariant>, ParseCreateExperimentHttpRequestError>>()?;

        let validated_variants = ExperimentVariants::new(variants.to_owned())?;

        Ok(CreateExperimentRequest::new(name, validated_variants))
    }
}

pub async fn create_experiment<ES: ExperimentService>(
    State(state): State<AppState<ES>>,
    Json(body): Json<CreateExperimentHttpRequestBody>,
) -> Result<ApiSuccess<CreateExperimentResponseData>, ApiError> {
    let domain_req = body.try_into_domain()?;
    state
        .experiment_service
        .create_experiment(&domain_req)
        .await
        .map_err(ApiError::from)
        .map(|ref experiment| ApiSuccess::new(StatusCode::CREATED, experiment.into()))
}
