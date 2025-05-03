

use crate::ace::arrays::Arrays;
use crate::ace::blocks::{DataBlockType, InterpolationTable};
use crate::ace::blocks::block_traits::{get_block_start, block_range_to_vec, PullFromXXS, Process};

#[derive(Debug, Clone, Default)]
pub struct BDD {
    pub decay_constants: Vec<f64>,
    pub precursor_tables: Vec<InterpolationTable>
}

impl<'a> PullFromXXS<'a> for BDD {
    fn pull_from_xxs_array(is_fissile: bool, arrays: &Arrays) -> Option<Vec<f64>> {
        // We expect DNU if JXS(2) != 0
        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &DataBlockType::BDD,
            arrays,
            is_fissile,
            "BDD is expected if JXS(2) != 0, but BDD was not found.".to_string(),
        )?;

        let mut block_length = 0;

        // Loop over all precursor groups
        for _ in 0..arrays.nxs.npcr {
            // Account for the decay constant
            block_length += 1;
            // Get the length of the precursor group data
            let precursor_group_data_length = InterpolationTable::get_table_length(block_start + block_length, arrays.xxs);
            block_length += precursor_group_data_length;
        }

        // Return the block's raw data as a vector
        Some(block_range_to_vec(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for BDD {
    type Dependencies = ();

    fn process(data: Vec<f64>, arrays: &Arrays, _dependencies: ()) -> Self {
        let mut decay_constants = Vec::new();
        let mut precursor_tables = Vec::new();

        // Loop over all precursor groups
        let mut offset = 0;
        for _ in 0..arrays.nxs.npcr {
            // Grab the decay constant
            decay_constants.push(data[offset] * 1e8);
            offset += 1;
            // Construct the interpolation table which describes probabilities for the precursor group
            let precursor_group_data_length = InterpolationTable::get_table_length(offset, &data);
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