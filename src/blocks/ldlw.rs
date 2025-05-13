use std::collections::HashMap;
use std::ops::Deref;

use crate::arrays::Arrays;
use crate::blocks::{BlockType, MTR};
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// LDLW data block
//
// Contains location data of energy distributions for secondary netruons.
//=====================================================================
#[derive(Debug, Clone, PartialEq)]
pub struct LDLW ( pub HashMap<usize, usize> );

impl Deref for LDLW {
    type Target = HashMap<usize, usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for LDLW {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We expect LDLW if NXS(5) != 0
        let has_secondary_neutron_reactions = arrays.nxs.nr != 0;

        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &BlockType::LDLW,
            arrays,
            has_secondary_neutron_reactions,
            "LDLW is expected if NXS(5) != 0, but LDLW was not found.".to_string(),
        )?;
        
        // Calculate the block length, see the LDLW description in the ACE spec
        let block_length = arrays.nxs.nr;

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for LDLW {
    type Dependencies = &'a Option<MTR>;

    fn process(data: &[f64], _arrays: &Arrays, mtr: &Option<MTR>) -> Self {
        // If we have available cross section identifiers from MTR, use them
        let energy_distribution_locs: HashMap<usize, usize> = if mtr.is_some() {
            data[1..]
                .iter()
                .enumerate()
                .map(|(i, &val)| (
                    mtr.as_ref().unwrap()[i],
                    val.to_bits() as usize
                ))
                .collect()
        } else {
            HashMap::new()
        };

        Self ( energy_distribution_locs )
    }
}

impl std::fmt::Display for LDLW {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LDLW({} reactions)", self.len())
    }
}

// #[cfg(test)]
// mod tests {
//     use crate::{utils::get_parsed_test_file, helpers::MTNumber};

//     #[tokio::test]
//     async fn test_land_parsing() {
//         let parsed_pace = get_parsed_test_file().await;

//         // Check contents
//         let ldlw = parsed_pace.data_blocks.LDLW.unwrap();
//         assert_eq!(ldlw.get(&(MTNumber::ElasticScattering as usize)), Some(&1));
//         assert_eq!(ldlw.get(&(MTNumber::Fission as usize)), Some(&0));
//     }
// }