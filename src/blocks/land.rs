use std::collections::HashMap;
use std::ops::Deref;

use crate::arrays::Arrays;
use crate::blocks::{BlockType, MTR, TYR};
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};
use crate::helpers::MTNumber;

//=====================================================================
// LAND data block
//
// Contains location data of angular distirbutions for all reactions
// which produce secondary neutrons.
//=====================================================================
#[derive(Debug, Clone, PartialEq)]
pub struct LAND ( pub HashMap<usize, isize> );

impl Deref for LAND {
    type Target = HashMap<usize, isize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for LAND {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We always expect LAND.
        let always_expected = true;

        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &BlockType::LAND,
            arrays,
            always_expected,
            "LAND is always expected, but LAND was not found.".to_string(),
        )?;
        
        // Calculate the block length, see the LAND description in the ACE spec
        // We will always have data for elastic scattering, so we need to add 1 to the number of reactions
        let block_length = arrays.nxs.nr + 1;

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for LAND {
    type Dependencies = &'a Option<MTR>;

    fn process(data: &[f64], _arrays: &Arrays, mtr: &Option<MTR>) -> Self {
        // If we have available cross section identifiers from MTR, use them
        let mut angular_distribution_locs: HashMap<usize, isize> = if mtr.is_some() {
            data[1..]
                .iter()
                .enumerate()
                .map(|(i, &val)| (
                    mtr.as_ref().unwrap()[i],
                    val.to_bits() as isize
                ))
                .collect()
        } else {
            HashMap::new()
        };
        
        // We will always have an angular distribution for elastic scattering
        angular_distribution_locs.insert(MTNumber::ElasticScattering as usize, data[0].to_bits() as isize);

        Self ( angular_distribution_locs )
    }
}

impl LAND {
    pub fn mt_values_with_distributions(&self, tyr: &Option<TYR>) -> Vec<usize> {
        let mut mt_vals = Vec::new();
        if let Some(tyr_block) = tyr {
            // Get the reaction types with neutron release from the TYR block and remove
            // those which are shown in LAND as not having a distribution.
            for mt in tyr_block.keys() {
                if let Some(&val) = self.get(mt) {
                    if val != -1 {
                        mt_vals.push(*mt);
                    }
                }
            }
        }
        // Add in elastic scattering, which is always present in the AND block but is not in TYR
        mt_vals.push(MTNumber::ElasticScattering as usize);
        mt_vals
    }
}

impl std::fmt::Display for LAND {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LAND({} reactions)", self.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::{utils::get_parsed_test_file, helpers::MTNumber};

    #[tokio::test]
    async fn test_land_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let land = parsed_ace.data_blocks.LAND.unwrap();
        assert_eq!(land.get(&(MTNumber::ElasticScattering as usize)), Some(&1));
        assert_eq!(land.get(&(MTNumber::Fission as usize)), Some(&0));
    }
}