// Represents the LSIG data block - contains locations of incident neutron cross section values.
use crate::ace::arrays::Arrays;
use crate::ace::blocks::DataBlockType;
use crate::ace::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

// See the ACE format spec for a description of the LSIG block
#[derive(Debug, Clone, PartialEq)]
pub struct LSIG {
    pub xs_locs: Vec<usize>
}

impl<'a> PullFromXXS<'a> for LSIG {
    fn pull_from_xxs_array(has_xs_other_than_elastic: bool, arrays: &'a Arrays) -> Option<&'a [f64]> {
        // If the block type's start index is non-zero, the block is present in the XXS array
        // We expect LSIG if NXS(4) (NTR) != 0
        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &DataBlockType::LSIG,
            arrays,
            has_xs_other_than_elastic,
            "LSIG is expected if NXS(4) (NTR) != 0, but LSIG was not found.".to_string(),
        )?;
        
        // Calculate the block length, see the LSIG description in the ACE spec
        let num_reactions = arrays.nxs.ntr;
        let block_length = num_reactions;

        // Return the block's raw data as a vector
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for LSIG {
    type Dependencies = ();

    fn process(data: &[f64], _arrays: &Arrays, _dependencies: ()) -> Self {
        let xs_locs: Vec<usize> = data
            .iter()
            .map(|val| val.to_bits() as usize)
            .collect();

        Self { xs_locs }
    }
}

impl std::fmt::Display for LSIG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LSIG({} xs)", self.xs_locs.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_lsig_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let lsig = parsed_ace.data_blocks.LSIG.unwrap();
        assert_eq!(lsig.xs_locs, vec![1]);
    }
}