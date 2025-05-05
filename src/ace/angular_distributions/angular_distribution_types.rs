use std::error::Error;
use std::ops::Deref;

use crate::unitf64::UnitF64;
use crate::ace::interpolation::{InterpolationScheme, InterpolationTable};

// Trait to sample the cosine of the scattering angle from a given
// angular distribution provided with a random number from [0, 1].
// As with all sampling methods in the PACE library, the user is responsible for providing a random
// number in the range [0.0, 1.0]. This is checked in debug builds, but not in release builds.
pub trait SampleAngle {
    fn sample_cos_theta(&self, unitf64: UnitF64) -> Result<f64, Box<dyn Error>>;
}

// Define an enum to represent the three possible angular distribution types
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub enum AngularDistribution {
    Isotropic(IsotropicAngularDistribution),
    Tabulated(TabulatedAngularDistribution),
    EquiprobableBins(EquiprobableBinsAngularDistribution),
}

impl SampleAngle for AngularDistribution {
    fn sample_cos_theta(&self, unitf64: UnitF64) -> Result<f64, Box<dyn Error>> {
        match self {
            AngularDistribution::Isotropic(distribution) => distribution.sample_cos_theta(unitf64),
            AngularDistribution::Tabulated(distribution) => distribution.sample_cos_theta(unitf64),
            AngularDistribution::EquiprobableBins(distribution) => distribution.sample_cos_theta(unitf64),
        }
    }
}

#[derive(Debug)]
pub struct AngularDistributionError(String);

impl Error for AngularDistributionError {}

impl std::fmt::Display for AngularDistributionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// There are a number of different types of angular distributions that can be used in the ACE format.
// Isotropic scattering
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct IsotropicAngularDistribution {}

impl SampleAngle for IsotropicAngularDistribution {
    fn sample_cos_theta(&self, unitf64: UnitF64) -> Result<f64, Box<dyn Error>> {
        Ok(2.0 * unitf64.0 - 1.0)
    }
}

// Tabulated cosine of the scattering angle with interpolation
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct TabulatedAngularDistribution ( pub InterpolationTable );

impl Deref for TabulatedAngularDistribution {
    type Target = InterpolationTable;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TabulatedAngularDistribution {
    pub fn new(
        interpolation_scheme: InterpolationScheme,
        cos_theta_bins: Vec<f64>,
        cos_theta_cdf: Vec<f64>,
    ) -> Result<Self, AngularDistributionError> {
        // Only histogram and linlin are supported for tabulated angular distributions in the ACE spec.
        if interpolation_scheme != InterpolationScheme::Histogram
            && interpolation_scheme != InterpolationScheme::LinLin
        {
            return Err(AngularDistributionError(format!(
                "TabulatedAngularDistribution: Unsupported interpolation scheme for tabulated angular distribution: {:?}",
                interpolation_scheme
            )));
        }
        // Ensure that the cos_theta_bins and cos_theta_cdf are of the same length
        if cos_theta_bins.len() != cos_theta_cdf.len() {
            return Err(AngularDistributionError(format!(
                "TabulatedAngularDistribution: cos_theta_bins ({}) and cos_theta_cdf ({}) must be of the same length",
                cos_theta_bins.len(),
                cos_theta_cdf.len()
            )));
        }
        // Build an interpolation table for the cosine of the scattering angle
        // Because we are sampling from a CDF, the x values are the CDF values
        // and the y values are the cos(theta) values.
        let cos_theta_table =
            InterpolationTable::from_x_and_y(cos_theta_cdf, cos_theta_bins, interpolation_scheme);
        Ok(Self(cos_theta_table))
    }
}

impl SampleAngle for TabulatedAngularDistribution {
    fn sample_cos_theta(&self, unitf64: UnitF64) -> Result<f64, Box<dyn Error>> {
        self.interpolate(unitf64.0)
    }
}

// Special ACE type, 32 equiprobably bins of cos theta
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct EquiprobableBinsAngularDistribution ( pub InterpolationTable );

impl Deref for EquiprobableBinsAngularDistribution {
    type Target = InterpolationTable;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl EquiprobableBinsAngularDistribution {
    pub fn new(cos_theta_bins: Vec<f64>) -> Result<Self, AngularDistributionError> {
        const CAPACITY: usize = 33; // 32 bins + 1 for the last bin boundary
        // Exactly 33 points are required to define the 32 bins
        if cos_theta_bins.len() != CAPACITY {
            return Err(AngularDistributionError(format!(
                "EquiprobableBinsAngularDistribution: Expected {} cos(theta) bin boundaries, got {}",
                CAPACITY,
                cos_theta_bins.len()
            )));
        }

        // Ensure all cos_theta_bins are in the range [-1, 1]
        for &cos_theta in &cos_theta_bins {
            if cos_theta < -1.0 || cos_theta > 1.0 {
                return Err(AngularDistributionError(format!(
                    "EquiprobableBinsAngularDistribution: cos(theta) bin value {} is out of range [-1, 1]",
                    cos_theta
                )));
            }
        }

        // Sort the cos_theta_bins into ascending order
        let mut cos_theta_bins = cos_theta_bins.clone();
        cos_theta_bins.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Make the CDF for the bins
        let cos_theta_cdf: Vec<f64> =
            Vec::from_iter((0..CAPACITY).map(|i| i as f64 / (CAPACITY - 1) as f64));

        // Build an interpolation table for the cosine of the scattering angle
        // Because we are sampling from a CDF, the x values are the CDF values
        // and the y values are the cos(theta) values.
        let cos_theta_table =
            InterpolationTable::from_x_and_y(cos_theta_cdf, cos_theta_bins, InterpolationScheme::LinLin);
        Ok(Self(cos_theta_table))
    }
}

impl SampleAngle for EquiprobableBinsAngularDistribution {
    fn sample_cos_theta(&self, unitf64: UnitF64) -> Result<f64, Box<dyn Error>> {
        self.interpolate(unitf64.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isotropic_angular_distribution() {
        let distribution = IsotropicAngularDistribution {};
        let unitf64 = UnitF64(0.5);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, 0.0);

        let unitf64 = UnitF64(0.0);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, -1.0);

        let unitf64 = UnitF64(1.0);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_tabulated_angular_distribution() {
        let interpolation_scheme = InterpolationScheme::LinLin;
        let cos_theta_bins = vec![-1.0, 0.0, 1.0];
        let cos_theta_cdf = vec![0.0, 0.5, 1.0];
        let distribution = TabulatedAngularDistribution::new(
            interpolation_scheme,
            cos_theta_bins,
            cos_theta_cdf
        ).expect("Failed to create TabulatedAngularDistribution");

        let unitf64 = UnitF64(0.0);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, -1.0);

        let unitf64 = UnitF64(0.25);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, -0.5);

        let unitf64 = UnitF64(0.5);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, 0.0);

        let unitf64 = UnitF64(0.75);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, 0.5);

        let unitf64 = UnitF64(1.0);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_tabulated_angular_distribution_invalid_interpolation() {
        let interpolation_scheme = InterpolationScheme::LogLog; // Unsupported scheme
        let cos_theta_bins = vec![-1.0, 0.0, 1.0];
        let cos_theta_cdf = vec![0.0, 0.5, 1.0];
        assert!(TabulatedAngularDistribution::new(interpolation_scheme, cos_theta_bins, cos_theta_cdf).is_err());
    }

    #[test]
    fn test_equiprobable_bins_angular_distribution() {
        let cos_theta_bins: Vec<f64> = Vec::from_iter((0..33).map(|i| i as f64 / (33 - 1) as f64 * 2.0 - 1.0));
        let distribution = EquiprobableBinsAngularDistribution::new(cos_theta_bins).expect("Failed to create EquiprobableBinsAngularDistribution");

        let unitf64 = UnitF64(0.0);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, -1.0);

        let unitf64 = UnitF64(0.5);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, 0.0);

        let unitf64 = UnitF64(1.0);
        let result = distribution.sample_cos_theta(unitf64).unwrap();
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_equiprobable_bins_angular_distribution_invalid_bins() {
        let cos_theta_bins = vec![-1.0, 0.0, 0.1, 1.0];
        assert!(EquiprobableBinsAngularDistribution::new(cos_theta_bins).is_err());
    }

    #[test]
    fn test_equiprobable_bins_angular_distribution_out_of_range() {
        let mut cos_theta_bins: Vec<f64> = Vec::from_iter((0..33).map(|i| i as f64 / (33 - 1) as f64));
        cos_theta_bins[0] = -1.5;
        assert!(EquiprobableBinsAngularDistribution::new(cos_theta_bins).is_err());
    }
}
