// Represents the MTR data block - this contains the MT numbers for the incident neutron cross
// sections avaiable in the file.
use crate::ace::arrays::Arrays;
use crate::ace::blocks::DataBlockType;
use crate::ace::blocks::block_traits::{get_block_start, block_range_to_vec, PullFromXXS, Process};

// See page 12 of the ACE format spec for a description of the MTR block
#[derive(Debug, Clone, PartialEq)]
pub struct MTR {
    pub reaction_types: Vec<usize>
}

impl<'a> PullFromXXS<'a> for MTR {
    fn pull_from_xxs_array(has_xs_other_than_elastic: bool, arrays: &Arrays) -> Option<Vec<f64>> {
        // If the block type's start index is non-zero, the block is present in the XXS array
        // We expect MTR if NXS(4) (NTR) != 0
        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &DataBlockType::MTR,
            arrays,
            has_xs_other_than_elastic,
            "MTR is expected if NXS(4) (NTR) != 0, but LQR was not found.".to_string(),
        )?;
        
        // Calculate the block end index, see the MTR description in the ACE spec
        let num_reactions = arrays.nxs.ntr;
        let block_length = num_reactions;

        // Return the block's raw data as a vector
        Some(block_range_to_vec(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for MTR {
    type Dependencies = ();

    fn process(data: Vec<f64>, arrays: &Arrays, dependencies: ()) -> Self {
        let reaction_types: Vec<usize> = data
            .iter()
            .map(|val| val.to_bits() as usize)
            .collect();

        Self { reaction_types }
    }
}

impl std::fmt::Display for MTR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MTR({} reactions)", self.reaction_types.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_mtr_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let mtr = parsed_ace.data_blocks.MTR.unwrap();
        assert_eq!(mtr.reaction_types, vec![18]);
    }
}