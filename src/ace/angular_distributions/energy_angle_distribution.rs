use std::error::Error;

use crate::ace::angular_distributions::{AngularDistribution, IsotropicAngularDistribution};
use crate::unitf64::UnitF64;

use super::angular_distribution_types::SampleAngle;

// This struct contains all of the angular distributions for different energies for a reaction.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct EnergyDependentAngularDistribution {
    pub energy: Vec<f64>,
    pub distributions: Vec<AngularDistribution>,
}

impl EnergyDependentAngularDistribution {

    pub fn new_fully_isotropic() -> Self {
        // Make a fully isotropic distribution over all energies
        Self {
            energy: vec![1.0e-11, 3.0e1],
            distributions: vec![
                AngularDistribution::Isotropic(IsotropicAngularDistribution {}),
                AngularDistribution::Isotropic(IsotropicAngularDistribution {})
            ],
        }
    }

    pub fn sample_cos_theta_at_energy(&self, energy: f64, unitf64: UnitF64) -> Result<f64, Box<dyn std::error::Error>> {
        // Check if the energy is within the range of the distribution
        if energy < self.energy[0] || energy > self.energy[self.energy.len() - 1] {
            return Err(Box::new(EnergyDependentAngularDistributionError(format!(
                "Energy {} is out of range [{}, {}]",
                energy, self.energy[0], self.energy[self.energy.len() - 1]
            ))));
        }

        // Find the energy index for the given energy
        let energy_index = self.energy.iter().position(|&e| e >= energy).unwrap();

        // If the energy is exactly equal to a value in the energy array, same the distribution at that index
        if energy == self.energy[energy_index] {
            self.distributions[energy_index].sample_cos_theta(unitf64)
        } else {
            // Otherwise, we need to interpolate between the two closest distributions
            let lower_energy_index = energy_index - 1;
            let upper_energy_index = energy_index;

            // Interpolate the distributions at the two closest energies
            let lower_distribution = &self.distributions[lower_energy_index];
            let upper_distribution = &self.distributions[upper_energy_index];

            // Calculate the interpolation factor
            let factor = (energy - self.energy[lower_energy_index]) / (self.energy[upper_energy_index] - self.energy[lower_energy_index]);

            // Sample from both distributions and interpolate the result
            let lower_sample = lower_distribution.sample_cos_theta(unitf64)?;
            let upper_sample = upper_distribution.sample_cos_theta(unitf64)?;

            // Interpolate the samples
            Ok(lower_sample + (upper_sample - lower_sample) * factor)
        }
    }
}

impl<'a> std::fmt::Display for EnergyDependentAngularDistribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnergyDependentAngularDistribution({} energies)", self.energy.len())
    }
}

#[derive(Debug)]
pub struct EnergyDependentAngularDistributionError(String);

impl Error for EnergyDependentAngularDistributionError {}

impl std::fmt::Display for EnergyDependentAngularDistributionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use crate::ace::interpolation::InterpolationScheme;
    use crate::ace::angular_distributions::{
        IsotropicAngularDistribution,
        TabulatedAngularDistribution,
        EquiprobableBinsAngularDistribution
    };

    fn make_test_distribution() -> EnergyDependentAngularDistribution {
        let energy = vec![1.0, 2.0, 3.0];
        let isotropic_distribution = AngularDistribution::Isotropic(IsotropicAngularDistribution {});
        let tabulated_distribution = AngularDistribution::Tabulated(
            TabulatedAngularDistribution::new(
                InterpolationScheme::LinLin,
                vec![0.0, 0.5, 1.0],
                vec![0.0, 0.5, 1.0],
            ).unwrap()
        );
        let equiprobable_bins_distribution = AngularDistribution::EquiprobableBins(
            EquiprobableBinsAngularDistribution::new(
                Vec::from_iter((0..33).map(|i| i as f64 / (33 - 1) as f64 - 1.0)),
            ).unwrap()
        );
        let distributions = vec![isotropic_distribution, tabulated_distribution, equiprobable_bins_distribution];

        EnergyDependentAngularDistribution { energy, distributions }
    }

    #[test]
    fn test_sample_cos_theta_at_energy_on_bounds() {
        let energy_dependent_angular_distribution = make_test_distribution();
        // Test isotropic distribution
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(1.0, UnitF64(0.0));
        assert_eq!(result.unwrap(), -1.0);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(1.0, UnitF64(0.5));
        assert_eq!(result.unwrap(), 0.0);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(1.0, UnitF64(1.0));
        assert_eq!(result.unwrap(), 1.0);
        // Test tabulated distribution
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(2.0, UnitF64(0.0));
        assert_eq!(result.unwrap(), 0.0);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(2.0, UnitF64(0.5));
        assert_eq!(result.unwrap(), 0.5);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(2.0, UnitF64(1.0));
        assert_eq!(result.unwrap(), 1.0);
        // Test equiprobable bins distribution
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(3.0, UnitF64(0.0));
        assert_eq!(result.unwrap(), -1.0);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(3.0, UnitF64(0.5));
        assert_eq!(result.unwrap(), -0.5);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(3.0, UnitF64(1.0));
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_sample_cos_theta_at_energy_off_bounds() {
        let energy_dependent_angular_distribution = make_test_distribution();
        // Test interpolation between isotropic and tabulated distribution
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(1.5, UnitF64(0.0));
        assert_eq!(result.unwrap(), -0.5);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(1.5, UnitF64(0.5));
        assert_eq!(result.unwrap(), 0.25);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(1.5, UnitF64(1.0));
        assert_eq!(result.unwrap(), 1.0);
        // Test interpolation between tabulated and equiprobable distribution
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(2.5, UnitF64(0.0));
        assert_eq!(result.unwrap(), -0.5);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(2.5, UnitF64(0.5));
        assert_eq!(result.unwrap(), 0.0);
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(2.5, UnitF64(1.0));
        assert_eq!(result.unwrap(), 0.5);
    }

    #[test]
    fn test_sample_cos_theta_at_energy_out_of_range() {
        let energy_dependent_angular_distribution = make_test_distribution();
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(4.0, UnitF64(0.0));
        assert!(result.is_err());
        let result = energy_dependent_angular_distribution.sample_cos_theta_at_energy(0.0, UnitF64(0.0));
        assert!(result.is_err());
    }
}
