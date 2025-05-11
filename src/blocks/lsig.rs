use std::ops::Deref;

use crate::arrays::Arrays;
use crate::blocks::BlockType;
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// LSIG data block
//
// Contains locations of incident neutron cross section values. See the
// ACE format spec for a description of the LSIG block.
//=====================================================================
#[derive(Debug, Clone, PartialEq)]
pub struct LSIG ( pub Vec<usize> );

impl Deref for LSIG {
    type Target = Vec<usize>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for LSIG {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We expect LSIG if NXS(4) (NTR) != 0.
        let has_xs_other_than_elastic = arrays.nxs.ntr != 0;

        // Get the starting index of the block in the XXS array
        let block_start = get_block_start(
            &BlockType::LSIG,
            arrays,
            has_xs_other_than_elastic,
            "LSIG is expected if NXS(4) (NTR) != 0, but LSIG was not found.".to_string(),
        )?;
        
        // Calculate the block length, see the LSIG description in the ACE spec
        let num_reactions = arrays.nxs.ntr;
        let block_length = num_reactions;

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for LSIG {
    type Dependencies = ();

    fn process(data: &[f64], _arrays: &Arrays, _dependencies: ()) -> Self {
        Self(data.iter().map(|val| val.to_bits() as usize).collect())
    }
}

impl std::fmt::Display for LSIG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LSIG({} xs)", self.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_lsig_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let lsig = parsed_ace.data_blocks.LSIG.unwrap();
        assert_eq!(lsig.len(), 1);
        assert_eq!(lsig[0], 1);
    }
}