

use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::{DataBlockType, InterpolationTable};
use crate::ace::blocks::block_traits::Process;

#[derive(Debug, Clone, Default)]
pub struct BDD {
    pub decay_constants: Vec<f64>,
    pub precursor_tables: Vec<InterpolationTable>
}

impl BDD {
    pub fn pull_from_xxs_array<'a>(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64] {
        let mut block_length = 0;

        // Block start index (binary XXS is zero indexed for speed)
        let block_start = jxs_array.get(&DataBlockType::BDD) - 1;
        // Loop over all precursor groups
        for _ in 0..nxs_array.npcr {
            // Account for the decay constant
            block_length += 1;
            // Get the length of the precursor group data
            let precursor_group_data_length = InterpolationTable::get_table_length(block_start + block_length, xxs_array);
            block_length += precursor_group_data_length;
        }

        // Avoid issues if this is the last block in the file
        let mut block_end = block_start + block_length;
        if block_end == xxs_array.len() + 1 {
            block_end -= 1;
        }
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl<'a> Process<'a> for BDD {
    type Dependencies = &'a NxsArray;

    fn process(data: &[f64], nxs_array: &NxsArray) -> Self {
        let mut decay_constants = Vec::new();
        let mut precursor_tables = Vec::new();

        // Loop over all precursor groups
        let mut offset = 0;
        for _ in 0..nxs_array.npcr {
            // Grab the decay constant
            decay_constants.push(data[offset] * 1e8);
            offset += 1;
            // Construct the interpolation table which describes probabilities for the precursor group
            let precursor_group_data_length = InterpolationTable::get_table_length(offset, data);
            precursor_tables.push(InterpolationTable::process(&data[offset..offset+precursor_group_data_length]));
            offset += precursor_group_data_length;
        }

        BDD {decay_constants, precursor_tables}
    }
}

impl std::fmt::Display for BDD {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BDD({} precursor groups)", self.precursor_tables.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_bdd_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let bdd = parsed_ace.data_blocks.BDD.unwrap();
        assert_eq!(bdd.decay_constants.len(), 6);
        assert_eq!(bdd.precursor_tables.len(), 6);
        assert_eq!(
            bdd.decay_constants,
            vec![0.01, 0.03, 0.05, 0.09, 0.3, 0.5]
        );
    }
}