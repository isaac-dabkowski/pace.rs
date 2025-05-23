use anyhow::Result;

use crate::arrays::Arrays;
use crate::interpolation::InterpolationTable;
use crate::blocks::BlockType;
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// DNU data block
//
// Contains information on the number of delayed neutrons released
// per fission.
//=====================================================================
#[derive(Debug, Clone, Default)]
pub struct DNU (InterpolationTable);

impl DNU {
    // Evaluate the tabulated nu at an energy (given in MeV)
    pub fn evaluate(&self, energy: f64) -> Result<f64> {
        self.0.interpolate(energy).map_err(anyhow::Error::from)
    }
}

impl<'a> PullFromXXS<'a> for DNU {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We expect DNU if JXS(2) != 0
        let is_fissile = arrays.jxs.get(&BlockType::NU) != 0;

        // Validate that the block is there and get the start indexx
        let block_start = get_block_start(
            &BlockType::DNU,
            arrays,
            is_fissile,
            "DNU is expected if JXS(2) != 0, but DNU was not found.".to_string(),
        )?;

        // Calculate the block length, see the DNU description in the ACE spec
        let mut block_length = 1;
        block_length += InterpolationTable::get_table_length(block_start + block_length, arrays.xxs);

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for DNU {
    type Dependencies = ();

    fn process(data: &[f64], _arrays: &Arrays, _dependencies: ()) -> Self {
        // Construct the interpolation table which describes probabilities for the precursor group
        Self(InterpolationTable::process(&data[1..]))
    }
}

impl std::fmt::Display for DNU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DNU()")
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_dnu_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let dnu = parsed_ace.data_blocks.DNU.unwrap();
        assert!((dnu.evaluate(1e-11).unwrap() - 1.0).abs() < 1e-6);
        assert!((dnu.evaluate(30.0).unwrap() - 2.0).abs() < 1e-6);
        assert!((dnu.evaluate(10.0).unwrap() - 1.333333).abs() < 1e-6);
        assert!(dnu.evaluate(100.0).is_err());
    }
}