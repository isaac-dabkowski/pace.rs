use std::collections::HashMap;

// Represents the LQR data block - contains Q values for different reactions.
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::{DataBlockType, MTR};
use crate::ace::blocks::block_traits::Process;

type MT = usize;

// See page 15 of the ACE format spec for a description of the LQR block
#[derive(Debug, Clone, PartialEq)]
pub struct LQR {
    pub q_vals: HashMap<MT, f64>
}

impl LQR {
    pub fn pull_from_xxs_array<'a>(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64] {
        // Block start index (binary XXS is zero indexed for speed)
        let block_start = jxs_array.get(&DataBlockType::LQR) - 1;
        // Calculate the block end index, see the LQR description in the ACE spec
        let num_reactions = nxs_array.ntr;
        let mut block_end = block_start + num_reactions;
        // Avoid issues if this is the last block in the file
        if block_end == xxs_array.len() + 1 {
            block_end -= 1;
        }
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl<'a> Process<'a> for LQR {
    type Dependencies = &'a MTR;

    fn process(data: &[f64], mtr: &MTR) -> Self {
        let q_vals: HashMap<MT, f64> = data
            .iter()
            .enumerate()
            .map(|(i, &q)| (mtr.reaction_types[i], q))
            .collect();

        Self { q_vals }
    }
}

impl std::fmt::Display for LQR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "LQR({} reactions)", self.q_vals.len())
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
        assert_eq!(lqr.q_vals[&18], 41.0);
    }
}