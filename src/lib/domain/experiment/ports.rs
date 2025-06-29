use std::future::Future;

use crate::domain::experiment::models::experiment::CreateExperimentError;
#[allow(unused_imports)]
use crate::domain::experiment::models::experiment::ExperimentName;
use crate::domain::experiment::models::experiment::{CreateExperimentRequest, Experiment};

/// `ExperimentService` is the public API for the experiment domain.
pub trait ExperimentService: Clone + Send + Sync + 'static {
    /// Asynchronously create a new [Experiment].
    ///
    /// # Errors
    ///
    /// - [CreateExperimentError::Duplicate] if an [Experiment] with the same [ExperimentName] already exists.
    fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> impl Future<Output = Result<Experiment, CreateExperimentError>> + Send;
}

/// `ExperimentRepository` represents a store of experiment data.
pub trait ExperimentRepository: Send + Sync + Clone + 'static {
    /// Asynchronously persist a new [Experiment].
    ///
    /// # Errors
    ///
    /// - MUST return [CreateExperimentError::Duplicate] if an [Experiment] with the same [ExperimentName]
    ///   already exists.
    fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> impl Future<Output = Result<Experiment, CreateExperimentError>> + Send;
}
