// Represents the MTR data block - this contains the MT numbers for the incident neutron cross
// sections avaiable in the file.
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::DataBlockType;
use crate::ace::blocks::block_traits::{PullFromXXS, Process};

// See page 12 of the ACE format spec for a description of the MTR block
#[derive(Debug, Clone, PartialEq)]
pub struct MTR {
    pub reaction_types: Vec<usize>
}

impl<'a> PullFromXXS<'a> for MTR {
     fn pull_from_xxs_array(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64] {
        // Block start index (binary XXS is zero indexed for speed)
        let block_start = jxs_array.get(&DataBlockType::MTR) - 1;
        // Calculate the block end index, see the MTR description in the ACE spec
        let num_reactions = nxs_array.ntr;
        let block_end = block_start + num_reactions;
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl<'a> Process<'a> for MTR {
    type Dependencies = ();

    fn process(data: &[f64], dependencies: ()) -> Self {
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