// Represents the LSIG data block - contains locations of incident neutron cross section values.
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::DataBlockType;

// See page 16 of the ACE format spec for a description of the LSIG block
#[derive(Debug, Clone, PartialEq)]
pub struct LSIG {
    pub xs_locs: Vec<usize>
}

impl LSIG {
    pub fn process(text_data: Vec<String>) -> Self {
        let time = std::time::SystemTime::now();
        let xs_locs: Vec<usize> = text_data
            .iter()
            .map(|val| val.parse().unwrap())
            .collect();
        println!(
            "⚛️  Time to process LSIG ⚛️ : {} μs",
            std::time::SystemTime::now().duration_since(time).unwrap().as_micros()
        );
        Self { xs_locs }
    }

    // Pull an LSIG block from a XXS array
    pub fn pull_from_ascii_xxs_array<'a>(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [&str]) -> &'a [&'a str] {
        // Block start index
        let block_start = jxs_array.get(&DataBlockType::LSIG);
        // Calculate the block end index, see the LSIG description in the ACE spec
        let num_reactions = nxs_array.ntr;
        let block_end = block_start + num_reactions;
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl std::fmt::Display for LSIG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LSIG({} xs)", self.xs_locs.len())
    }
}
