
use std::collections::HashMap;
use std::ops::Deref;

use crate::arrays::Arrays;
use crate::blocks::{BlockType, MTR};
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// LQR data block
//
// Contains Q values for different reactions. See of the ACE format
// spec for a description of the LQR block.
//=====================================================================
#[derive(Debug, Clone, PartialEq)]
pub struct LQR ( pub HashMap<usize, f64> );

impl Deref for LQR {
    type Target = HashMap<usize, f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for LQR {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We expect LQR if NXS(4) (NTR) != 0
        let has_xs_other_than_elastic = arrays.nxs.ntr != 0;

        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &BlockType::LQR,
            arrays,
            has_xs_other_than_elastic,
            "LQR is expected if NXS(4) (NTR) != 0, but LQR was not found.".to_string(),
        )?;

        // Calculate the block length, see the LQR description in the ACE spec
        let num_reactions = arrays.nxs.ntr;
        let block_length = num_reactions;

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for LQR {
    type Dependencies = &'a Option<MTR>;

    fn process(data: &[f64], _arrays: &Arrays, mtr: &Option<MTR>) -> Self {
        Self(data.iter().enumerate().map(|(i, &q)| (mtr.as_ref().unwrap()[i], q)).collect())
    }
}

impl std::fmt::Display for LQR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LQR({} reactions)", self.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_lqr_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let lqr = parsed_ace.data_blocks.LQR.unwrap();
        assert_eq!(lqr[&18], 41.0);
    }
}