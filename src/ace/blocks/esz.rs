// Represents the ESZ data block - this is the energy grid for our ACE file, along with several
// basic cross sections.
use crate::ace::arrays::Arrays;
use crate::ace::blocks::DataBlockType;
use crate::ace::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

// See page 12 of the ACE format spec for a description of the ESZ block
#[derive(Debug, Clone, PartialEq)]
pub struct ESZ {
    pub energy: Vec<f64>,
    pub total_xs: Vec<f64>,
    pub dissapearance_xs: Vec<f64>,
    pub elastic_xs: Vec<f64>,
    pub average_heating_numbers: Vec<f64>,
}

impl<'a> PullFromXXS<'a> for ESZ {
    // Pull an ESZ block from a XXS array
    fn pull_from_xxs_array(always_expected: bool, arrays: &'a Arrays) -> Option<&'a [f64]> {
        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &DataBlockType::ESZ,
            arrays,
            always_expected,
            "Every ACE file should have an ESZ block, but one was not found.".to_string(),
        )?;

        // Calculate the block length, see the ESZ description in the ACE spec
        let num_energies = arrays.nxs.nes;
        let block_length = 5 * num_energies;

        // Return the block's raw data as a vector
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for ESZ {
    type Dependencies = ();

    fn process(data: &[f64], arrays: &Arrays, _dependencies: ()) -> Self {
        let num_energy_points = arrays.nxs.nes;
        // Energy grid
        let energy = Vec::from(&data[0..num_energy_points]);
        // Total cross section
        let total_xs = Vec::from(&data[num_energy_points..2 * num_energy_points]);
        // Dissapearence cross section
        let dissapearance_xs = Vec::from(&data[2 * num_energy_points..3 * num_energy_points]);
        // Elastic cross section
        let elastic_xs = Vec::from(&data[3 * num_energy_points..4 * num_energy_points]);
        // Average heating numbers
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
    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_esz_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let esz = parsed_ace.data_blocks.ESZ.unwrap();
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