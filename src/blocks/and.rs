// Represents the AND data block - this contains incident neutron cross section data
// See the ACE format spec for a description of the AND block
use std::collections::HashMap;
use std::ops::Deref;

use crate::arrays::Arrays;
use crate::blocks::{BlockType, TYR, LAND};
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};
use crate::interpolation::InterpolationScheme;
use crate::angular_distributions::{
    AngularDistribution,
    IsotropicAngularDistribution,
    TabulatedAngularDistribution,
    EquiprobableBinsAngularDistribution,
    EnergyDependentAngularDistribution,
};

type AngularDistributionMap = HashMap<usize, EnergyDependentAngularDistribution>;


//=====================================================================
// AND data block
//
// Contains energy-dependent angular distributions for all reactions
// which produce secondary neutrons.
//=====================================================================
#[derive(Debug, Clone)]
pub struct AND ( pub AngularDistributionMap);

impl<'a> Deref for AND {
    type Target = AngularDistributionMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for AND {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // The AND block should always exist
        let always_expected = true;

        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &BlockType::AND,
            arrays,
            always_expected,
            "AND is always expected, but AND was not found.".to_string(),
        )?;

        // Calculate the block length, see the AND description in the ACE spec
        // - The AND block is fairly complex. To speed up getting its length, we will first pull the
        //   angular distribution locations from the LAND block.
        // - We will ignore the values in the LAND block that are -1 or 0.
        //     - Any entries in the LAND block that are -1 mean that the angular distribution is not provided.
        //     - Any entries in the LAND block that are 0 mean that the angular distribution is completely isotropic for all energies any no distribution is provided.
        // - The maximum value from the LAND block data is the location of the last reaction in the AND block (relative to the start of the block).
        let last_and_entry_relative_index = LAND::pull_from_xxs_array(arrays)?
            .iter()
            .map(|&x| x.to_bits() as isize)
            .filter(|&x| x != -1 && x != 0)
            .max()
            .unwrap_or(1)
            .abs() as usize;
        let last_and_entry_start = block_start + last_and_entry_relative_index;

        // Now that we have the last entry in the AND block, we can skip ahead to its last energy point.
        // The first entry is the number of energy points at which tabulated angular distributions (Ne).
        let last_and_num_energies = arrays.xxs[last_and_entry_start - 1].to_bits() as usize;
        // The next (Ne) entries are the number of energy points at which the last angular distribution is defined.
        // Following the energy grid, we have (Ne) location identifiers for the angular distributions,
        // we will pull these and find the maximum value from the list. This is the location of the last
        // angular distribution for the last entry in the AND block.
        let last_and_final_entry_maximum_relative_index = arrays.xxs[last_and_entry_start + last_and_num_energies..last_and_entry_start + 2 * last_and_num_energies]
            .iter()
            .map(|&x| x.to_bits() as isize)
            .filter(|&x| x != 0)
            .max_by_key(|x| x.abs())
            .unwrap_or(0);

        // Now, we will go to that distribution and get its length.
        let last_distribution_length = match last_and_final_entry_maximum_relative_index {
            0 => {
                // If the maximum distribution locator for all energies in the last entry is zero, then it was isotropic for
                // all energies and no distribution is provided.
                0
            },
            n if n < 0 => {
                // If the locator is negative, we have a tabulated scattering distribution.
                // Get the number points in the distribution.
                let num_points = arrays.xxs[block_start + last_and_final_entry_maximum_relative_index.abs() as usize].to_bits() as usize;
                // The tables length past the realtive index is 3 times the number of points,
                // since we have the scattering cosine values, a PDF, and a CDF.
                3 * num_points
            },
            n if n > 0 => {
                // If the locator is positive, we have a 32 equiprobable bin distribution, which means
                // we have 33 points to define the bins.
                33
            },
            _ => {
                // The last entry is not isotropic for all energies and a distribution is provided.
                // We will set the last entry to be the start of the last distribution's final energy point.
                panic!("Unexpected value for last AND distribution locator: {}", last_and_final_entry_maximum_relative_index);
            }
        };

        // We can now calculate the length of the AND block.
        let block_length = last_and_final_entry_maximum_relative_index.abs() as usize + last_distribution_length + 1;

    // Return the block's raw data as a slice
    Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for AND {
    type Dependencies = (&'a Option<TYR>, &'a Option<LAND>);

    fn process(data: &[f64], _arrays: &Arrays, dependencies: (&Option<TYR>, &Option<LAND>)) -> Self {
        let (tyr, land) = (
            dependencies.0,
            dependencies.1.clone().unwrap(),
        );

        let mut distributions = AngularDistributionMap::new();

        // Loop over our different reactions with angular distribution data
        for mt in land.mt_values_with_distributions(tyr).iter() {
            // Get the index of the reaction in the AND block using the LAND block
            let mt_index = land.get(mt).unwrap();

            // If the index is 0, we have an isotropic distribution for all energies
            if mt_index == &0 {
                // Create an isotropic angular distribution for all energies
                distributions.insert(*mt,
                    EnergyDependentAngularDistribution::new_fully_isotropic()
                );
                continue;
            }

            // We have an actual energy dependent distribution
            let mt_index = mt_index.abs() as usize;
            // Get the number of energy points for this reaction
            let num_energy_points = data[mt_index - 1].to_bits() as usize;
            // Pull ranges in the data array for the energy points and locators
            let energy_range = mt_index..mt_index + num_energy_points;
            let locators_range = mt_index + num_energy_points..mt_index + 2 * num_energy_points;

            // Pull the energy values at which we have angular distributions
            let energy = (&data[energy_range]).to_vec();
            // Get the angular distribution locators for this reaction
            let distribution_locators = &data[locators_range].iter()
                .map(|&x| x.to_bits() as isize)
                .collect::<Vec<isize>>();

            // Loop over the locators and create the angular distributions
            let mut angular_distributions = Vec::new();
            for &locator in distribution_locators {
                // Make the proper angular distribution based on the locator value
                let distribution  = match locator {
                    // If the locator is negative, we have a tabulated scattering distribution
                    n if n < 0 => {
                        // The first index is the interpolation scheme
                        let start_index = locator.abs() as usize - 1;
                        let tabulated_angular_distribution = make_tabulated_distribution_from_data(&data, start_index);
                        // Create the angular distribution
                        AngularDistribution::Tabulated(tabulated_angular_distribution)
                    },
                    // If the locator is positive, we have a 32-bin equiprobable distribution
                    n if n > 0 => {
                        let cos_theta_bins = &data[locator as usize..locator as usize + 33];
                        AngularDistribution::EquiprobableBins(
                            EquiprobableBinsAngularDistribution::new(cos_theta_bins.to_vec()).unwrap()
                        )
                    },
                    // If the locator is zero, we have an isotropic distribution
                    _ => AngularDistribution::Isotropic(IsotropicAngularDistribution {}),
                };
                angular_distributions.push(distribution);
            }
            
            // Insert the energy dependent angular distribution into the map
            distributions.insert(*mt,
                EnergyDependentAngularDistribution {
                    energy: energy,
                    distributions: angular_distributions,
                }
            );
        }

        Self(distributions)
    }
}

impl std::fmt::Display for AND {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AND({} reactions)", self.len())
    }
}

fn make_tabulated_distribution_from_data(data: &[f64], start_index: usize) -> TabulatedAngularDistribution {
    // First, get the interpolation scheme
    let interpolation_scheme = InterpolationScheme::from(data[start_index].to_bits() as usize);
    // Next, get the number of points in the distribution
    let num_points_index = start_index + 1;
    let num_points = data[num_points_index].to_bits() as usize;
    // Next, get the cos theta values at which the distribution is defined
    let cos_theta_values_index = num_points_index + 1;
    let cos_theta_value_range = cos_theta_values_index..cos_theta_values_index + num_points;
    let cos_theta_values = &data[cos_theta_value_range];
    // Finally, get the cos theta CDF values
    let cos_theta_cdf_index = cos_theta_values_index + 2 * num_points;
    let cos_theta_cdf_range = cos_theta_cdf_index..cos_theta_cdf_index + num_points;
    let cos_theta_cdf_values = &data[cos_theta_cdf_range];
    // Create the angular distribution
    TabulatedAngularDistribution::new(
        interpolation_scheme,
        cos_theta_values.to_vec(),
        cos_theta_cdf_values.to_vec(),
    ).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::get_parsed_test_file;
    use crate::helpers::MTNumber;

    #[tokio::test]
    async fn test_and_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        let and = parsed_ace.data_blocks.AND.unwrap();

        // Check that the reactions with angular distributions are present
        assert!(and.contains_key(&(MTNumber::ElasticScattering as usize)));
        assert!(and.contains_key(&(MTNumber::Fission as usize)));
    }

    #[tokio::test]
    async fn test_and_energy_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        let and = parsed_ace.data_blocks.AND.unwrap();
        let scatter_dist = and.get(&(MTNumber::ElasticScattering as usize)).unwrap();
        let fission_dist = and.get(&(MTNumber::Fission as usize)).unwrap();

        // Check that the energy values are correct
        assert_eq!(scatter_dist.energy.len(), 3);
        assert_eq!(scatter_dist.energy, vec![1.0E-11, 0.0E+00, 3.0E+01]);
        assert_eq!(fission_dist.energy.len(), 2);
        assert_eq!(fission_dist.energy, vec![1.0E-11, 3.0E+01]);
    }

    #[tokio::test]
    async fn test_and_distribution_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        let and = parsed_ace.data_blocks.AND.unwrap();
        let scatter_dist = and.get(&(MTNumber::ElasticScattering as usize)).unwrap();
        let fission_dist = and.get(&(MTNumber::Fission as usize)).unwrap();

        let isotropic_distribution = AngularDistribution::Isotropic(IsotropicAngularDistribution {});
        let tabulated_distribution1 = AngularDistribution::Tabulated(
            TabulatedAngularDistribution::new(
                InterpolationScheme::LinLin,
                vec![-1.0, 0.0, 1.0],
                vec![0.0, 0.5, 1.0],
            ).unwrap()
        );
        let tabulated_distribution2 = AngularDistribution::Tabulated(
            TabulatedAngularDistribution::new(
                InterpolationScheme::LinLin,
                vec![0.0, 0.25, 0.5, 0.75, 1.0],
                vec![0.0, 0.25, 0.5, 0.75, 1.0],
            ).unwrap()
        );

        // Check that the energy values are correct
        assert_eq!(scatter_dist.distributions.len(), 3);
        assert_eq!(scatter_dist.distributions, vec![tabulated_distribution1, tabulated_distribution2, isotropic_distribution.clone()]);
        assert_eq!(fission_dist.distributions.len(), 2);
        assert_eq!(fission_dist.distributions, vec![isotropic_distribution.clone(), isotropic_distribution]);
    }
}