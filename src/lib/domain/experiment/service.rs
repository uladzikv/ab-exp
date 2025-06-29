use crate::domain::experiment::models::experiment::CreateExperimentError;
use crate::domain::experiment::models::experiment::{CreateExperimentRequest, Experiment};
use crate::domain::experiment::ports::{ExperimentRepository, ExperimentService};

/// Canonical implementation of the [ExperimentService] port, through which the experiment domain API is
/// consumed.
#[derive(Debug, Clone)]
pub struct Service<R: ExperimentRepository> {
    repo: R,
}

impl<R: ExperimentRepository> Service<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }
}

impl<R: ExperimentRepository> ExperimentService for Service<R> {
    /// Create the [Experiment] specified in `req`.
    ///
    /// # Errors
    ///
    /// - Propagates any [CreateExperimentError] returned by the [ExperimentRepository].
    async fn create_experiment(
        &self,
        req: &CreateExperimentRequest,
    ) -> Result<Experiment, CreateExperimentError> {
        self.repo.create_experiment(req).await
    }
}
