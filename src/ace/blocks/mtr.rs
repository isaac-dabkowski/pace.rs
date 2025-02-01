// Represents the MTR data block - this contains the MT numbers for the incident neutron cross
// sections avaiable in the file.
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::DataBlockType;

// See page 12 of the ACE format spec for a description of the MTR block
#[derive(Debug, Clone, PartialEq)]
pub struct MTR {
    pub reaction_types: Vec<usize>
}

impl MTR {
    pub fn process(text_data: Vec<String>) -> Self {
        let reaction_types: Vec<usize> = text_data
            .iter()
            .map(|val| val.parse().unwrap())
            .collect();
        Self { reaction_types }
    }

    // Pull an ESZ block from a XXS array
    pub fn pull_from_ascii_xxs_array(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &[String]) -> Vec<String> {
        // Block start index
        let block_start = jxs_array.get(&DataBlockType::MTR);
        // Calculate the block end index, see the ESZ description in the ACE spec
        let num_reactions = nxs_array.ntr;
        let block_end = block_start + num_reactions;
        // Return the block
        xxs_array[block_start..block_end].to_vec()
    }
}

impl std::fmt::Display for MTR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MTR({} reactions)", self.reaction_types.len())
    }
}