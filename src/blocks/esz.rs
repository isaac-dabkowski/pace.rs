use crate::arrays::Arrays;
use crate::blocks::BlockType;
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// ESZ data block
//
// This is the energy grid for the ACE file, along with several basic
// cross sections. See the ACE format spec for a description of the
// ESZ block.
//=====================================================================

#[derive(Debug, Clone, PartialEq)]
pub struct ESZ {
    pub energy: Vec<f64>,
    pub total_xs: Vec<f64>,
    pub dissapearance_xs: Vec<f64>,
    pub elastic_xs: Vec<f64>,
    pub average_heating_numbers: Vec<f64>,
}

impl<'a> PullFromXXS<'a> for ESZ {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We always expect ESZ to be present in the ACE file.
        let always_expected = true;

        // Get the starting index of the block in the XXS array
        let block_start = get_block_start(
            &BlockType::ESZ,
            arrays,
            always_expected,
            "Every ACE file should have an ESZ block, but one was not found.".to_string(),
        )?;

        // Calculate the block length, see the ESZ description in the ACE spec
        let num_energies = arrays.nxs.nes;
        let block_length = 5 * num_energies;

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for ESZ {
    type Dependencies = ();

    fn process(data: &[f64], arrays: &Arrays, _dependencies: ()) -> Self {
        let num_energy_points = arrays.nxs.nes;
        let energy = Vec::from(&data[0..num_energy_points]);
        let total_xs = Vec::from(&data[num_energy_points..2 * num_energy_points]);
        let dissapearance_xs = Vec::from(&data[2 * num_energy_points..3 * num_energy_points]);
        let elastic_xs = Vec::from(&data[3 * num_energy_points..4 * num_energy_points]);
        let average_heating_numbers = Vec::from(&data[4 * num_energy_points..5 * num_energy_points]);
        Self {
            energy,
            total_xs,
            dissapearance_xs,
            elastic_xs,
            average_heating_numbers,
        }
    }
}

impl std::fmt::Display for ESZ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ESZ({} energies)", self.energy.len())
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_esz_parsing() {
        let parsed_pace = get_parsed_test_file().await;

        // Check parsed ESZ block from custom test file
        let esz = parsed_pace.data_blocks.ESZ.unwrap();
        assert_eq!(esz.energy.len(), 3);
        assert_eq!(esz.total_xs.len(), esz.energy.len());
        assert_eq!(esz.dissapearance_xs.len(), esz.energy.len());
        assert_eq!(esz.elastic_xs.len(), esz.energy.len());
        assert_eq!(esz.average_heating_numbers.len(), esz.energy.len());
        assert_eq!(esz.energy, vec![1.0, 2.0, 3.0]);
        assert_eq!(esz.total_xs, vec![100.0, 150.0, 200.0]);
        assert_eq!(esz.dissapearance_xs, vec![0.1, 0.15, 0.2]);
        assert_eq!(esz.elastic_xs, vec![5.0, 6.0, 7.0]);
        assert_eq!(esz.average_heating_numbers, vec![2.0, 4.0, 6.0]);
    }
}