// Represents the LQR data block - contains Q values for different reactions.
use std::collections::HashMap;
use std::ops::Deref;

use crate::ace::arrays::Arrays;
use crate::ace::blocks::block_types::MT;
use crate::ace::blocks::{DataBlockType, MTR};
use crate::ace::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

// See of the ACE format spec for a description of the LQR block
#[derive(Debug, Clone, PartialEq)]
pub struct LQR ( pub HashMap<MT, f64> );

impl Deref for LQR {
    type Target = HashMap<MT, f64>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for LQR {
    fn pull_from_xxs_array(has_xs_other_than_elastic: bool, arrays: &'a Arrays) -> Option<&'a [f64]> {
        // If the block type's start index is non-zero, the block is present in the XXS array
        // We expect LQR if NXS(4) (NTR) != 0
        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &DataBlockType::LQR,
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

    fn process(data: &[f64], arrays: &Arrays, mtr: &Option<MTR>) -> Self {
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
    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_lqr_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let lqr = parsed_ace.data_blocks.LQR.unwrap();
        assert_eq!(lqr[&18], 41.0);
    }
}