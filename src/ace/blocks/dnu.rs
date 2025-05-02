use std::error::Error;

use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::{DataBlockType, InterpolationTable};
use crate::ace::blocks::block_traits::{PullFromXXS, Process};

#[derive(Debug, Clone, Default)]
pub struct DNU (InterpolationTable);

impl DNU {
    // Evaluate the tabulated nu at an energy (given in MeV)
    pub fn evaluate(&self, energy: f64) -> Result<f64, Box<dyn Error>> {
        self.0.interpolate(energy)
    }
}

impl<'a> PullFromXXS<'a> for DNU {
    fn pull_from_xxs_array(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64] {
        // Block start index (binary XXS is zero indexed for speed)
        let mut block_length = 1;
        let block_start = jxs_array.get(&DataBlockType::DNU) - 1;
        block_length += InterpolationTable::get_table_length(block_start + block_length, xxs_array);

        // Avoid issues if this is the last block in the file
        let mut block_end = block_start + block_length;
        if block_end == xxs_array.len() + 1 {
            block_end -= 1;
        }
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl<'a> Process<'a> for DNU {
    type Dependencies = ();

    fn process(data: &[f64], dependencies: ()) -> Self {
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
    use super::*;

    use crate::ace::utils::get_parsed_test_file;

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