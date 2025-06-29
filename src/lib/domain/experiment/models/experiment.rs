use derive_more::{Display, From};
use thiserror::Error;
use uuid::Uuid;

/// Represents always valid experiment name.
#[derive(Display, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExperimentName(String);

#[derive(Clone, Debug, Error, PartialEq)]
#[error("experiment name cannot be empty")]
pub struct ExperimentNameEmptyError;
impl ExperimentName {
    pub fn new(raw_name: &str) -> Result<Self, ExperimentNameEmptyError> {
        let trimmed = raw_name.trim();
        if trimmed.is_empty() {
            Err(ExperimentNameEmptyError)
        } else {
            Ok(Self(trimmed.to_string()))
        }
    }
}

/// Represents always valid variant distribution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VariantDistribution(f64);

#[derive(Clone, Debug, Error, PartialEq)]
#[error("variant distribution should be more than zero and less than or equal to 100")]
pub struct VariantDistributionInvalidError;
impl VariantDistribution {
    pub fn new(value: f64) -> Result<Self, VariantDistributionInvalidError> {
        if value <= 0.0 || value > 100.0 {
            Err(VariantDistributionInvalidError)
        } else {
            Ok(Self(value))
        }
    }

    pub fn into_inner(self) -> f64 {
        self.0
    }
}

/// Represents always valid variant data.
#[derive(Display, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VariantData(String);

#[derive(Clone, Debug, Error)]
#[error("variant data cannot be empty")]
pub struct VariantDataEmptyError;
impl VariantData {
    pub fn new(raw_data: &str) -> Result<Self, VariantDataEmptyError> {
        if raw_data.is_empty() {
            Err(VariantDataEmptyError)
        } else {
            Ok(Self(raw_data.to_string()))
        }
    }
}

/// Represents always valid variant.
#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    distribution: VariantDistribution,
    data: VariantData,
}

impl Variant {
    pub fn new(distribution: VariantDistribution, data: VariantData) -> Self {
        Self { distribution, data }
    }

    pub fn distribution(&self) -> &VariantDistribution {
        &self.distribution
    }

    pub fn data(&self) -> &VariantData {
        &self.data
    }
}

/// Represents always valid list of variants.
#[derive(Clone, Debug, PartialEq)]
pub struct ExperimentVariants(Vec<Variant>);

#[derive(Clone, Debug, Error, PartialEq)]
#[error("sum of distributions is not equal to 100")]
pub struct DistributionSumError;
impl ExperimentVariants {
    /// Allowed deviation for the sum of distributions (in percentage).
    const EPSILON: f64 = 0.2;

    pub fn new(variants: Vec<Variant>) -> Result<Self, DistributionSumError> {
        let distributions: Vec<_> = variants.iter().map(|v| v.distribution()).collect();
        Self::validate_distribution(distributions.as_slice())?;

        Ok(Self(variants))
    }

    pub fn variants(&self) -> &Vec<Variant> {
        &self.0
    }

    /// Validates that the sum of distribution array elements equals 100% within a specified tolerance.
    ///
    /// # Arguments
    /// * `distributions` - array of distribution shares (in percentages).
    ///
    /// # Returns
    /// * `Ok(())` if the sum equals 100% within the specified tolerance.
    /// * `Err(DistributionSumError)` with error description otherwise.
    fn validate_distribution(
        distributions: &[&VariantDistribution],
    ) -> Result<(), DistributionSumError> {
        // Check if the array is empty
        if distributions.is_empty() {
            return Err(DistributionSumError);
        }

        // Calculate the sum of all elements
        let sum: f64 = distributions.iter().map(|d| d.0).sum();

        // Check if the sum is within 100% ± epsilon
        if (sum - 100.0).abs() > Self::EPSILON {
            return Err(DistributionSumError);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Experiment {
    id: Uuid,
    name: ExperimentName,
    variants: ExperimentVariants,
}

impl Experiment {
    pub fn new(id: Uuid, name: ExperimentName, variants: ExperimentVariants) -> Self {
        Self { id, name, variants }
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn name(&self) -> &ExperimentName {
        &self.name
    }

    pub fn variants(&self) -> &ExperimentVariants {
        &self.variants
    }
}

/// Data required by the domain to create an [Experiment].
#[derive(Clone, Debug, From)]
pub struct CreateExperimentRequest {
    name: ExperimentName,
    variants: ExperimentVariants,
}

impl CreateExperimentRequest {
    pub fn new(name: ExperimentName, variants: ExperimentVariants) -> Self {
        Self { name, variants }
    }

    pub fn name(&self) -> &ExperimentName {
        &self.name
    }

    pub fn variants(&self) -> &ExperimentVariants {
        &self.variants
    }
}

#[derive(Debug, Error)]
pub enum CreateExperimentError {
    #[error("experiment with name {name} already exists")]
    Duplicate { name: ExperimentName },
    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[cfg(test)]
mod variant_distribution_tests {
    use super::*;

    #[test]
    fn test_new_success() {
        let value = 75.0;
        let result = VariantDistribution::new(value);
        let expected = Ok(VariantDistribution(75.0));

        assert_eq!(result, expected);
    }

    #[test]
    fn test_distribution_is_invalid() {
        let value = 0.0;
        let result = VariantDistribution::new(value);
        let expected = Err(VariantDistributionInvalidError);

        assert_eq!(result, expected);
    }
}

#[cfg(test)]
mod experiment_tests {
    use super::*;

    #[test]
    fn test_new_success() {
        let variant_1_data = VariantData::new("10").unwrap();
        let variant_1_distribution = VariantDistribution::new(75.0).unwrap();
        let variant_1 = Variant::new(variant_1_distribution, variant_1_data);

        let variant_2_data = VariantData::new("20").unwrap();
        let variant_2_distribution = VariantDistribution::new(10.0).unwrap();
        let variant_2 = Variant::new(variant_2_distribution, variant_2_data);

        let variant_3_data = VariantData::new("50").unwrap();
        let variant_3_distribution = VariantDistribution::new(5.0).unwrap();
        let variant_3 = Variant::new(variant_3_distribution, variant_3_data);

        let variant_4_data = VariantData::new("5").unwrap();
        let variant_4_distribution = VariantDistribution::new(10.0).unwrap();
        let variant_4 = Variant::new(variant_4_distribution, variant_4_data);

        let experiment_variants_result = ExperimentVariants::new(vec![
            variant_1.clone(),
            variant_2.clone(),
            variant_3.clone(),
            variant_4.clone(),
        ]);

        let experiment_variants_expected = Ok(ExperimentVariants(vec![
            variant_1.clone(),
            variant_2.clone(),
            variant_3.clone(),
            variant_4.clone(),
        ]));

        assert_eq!(experiment_variants_result, experiment_variants_expected);

        let experiment_name_result = ExperimentName::new("price");
        let experiment_name_expected = Ok(ExperimentName("price".to_string()));

        assert_eq!(experiment_name_result, experiment_name_expected);

        let experiment_id = Uuid::new_v4();
        let experiment_result = Experiment::new(
            experiment_id,
            experiment_name_result.unwrap(),
            experiment_variants_result.unwrap(),
        );
        let experiment_expected = Experiment {
            id: experiment_id,
            name: ExperimentName("price".to_string()),
            variants: ExperimentVariants(vec![variant_1, variant_2, variant_3, variant_4]),
        };

        assert_eq!(experiment_result, experiment_expected);
    }

    #[test]
    fn test_new_experiment_variants_in_tolerance_success() {
        let variant_1_data = VariantData::new("version-1-url").unwrap();
        let variant_1_distribution = VariantDistribution::new(33.3).unwrap();
        let variant_1 = Variant::new(variant_1_distribution, variant_1_data);

        let variant_2_data = VariantData::new("version-2-url").unwrap();
        let variant_2_distribution = VariantDistribution::new(33.3).unwrap();
        let variant_2 = Variant::new(variant_2_distribution, variant_2_data);

        let variant_3_data = VariantData::new("version-3-url").unwrap();
        let variant_3_distribution = VariantDistribution::new(33.3).unwrap();
        let variant_3 = Variant::new(variant_3_distribution, variant_3_data);

        let experiment_variants_result = ExperimentVariants::new(vec![
            variant_1.clone(),
            variant_2.clone(),
            variant_3.clone(),
        ]);

        let experiment_variants_expected =
            Ok(ExperimentVariants(vec![variant_1, variant_2, variant_3]));

        assert_eq!(experiment_variants_result, experiment_variants_expected);
    }

    #[test]
    fn test_new_invalid_experiment_variants() {
        let variant_1_data = VariantData::new("version-1-url").unwrap();
        let variant_1_distribution = VariantDistribution::new(60.0).unwrap();
        let variant_1 = Variant::new(variant_1_distribution, variant_1_data);

        let variant_2_data = VariantData::new("version-2-url").unwrap();
        let variant_2_distribution = VariantDistribution::new(41.5).unwrap();
        let variant_2 = Variant::new(variant_2_distribution, variant_2_data);

        let experiment_variants_result =
            ExperimentVariants::new(vec![variant_1.clone(), variant_2.clone()]);

        let experiment_variants_expected = Err(DistributionSumError);

        assert_eq!(experiment_variants_result, experiment_variants_expected);
    }
}
