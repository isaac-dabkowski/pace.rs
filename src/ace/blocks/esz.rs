// Represents the ESZ data block - this is the energy grid for our ACE file, along with several
// basic cross sections.
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::DataBlockType;
use crate::ace::blocks::block_traits::{PullFromXXS, Process};

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
    fn pull_from_xxs_array(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64] {
        // Block start index (binary XXS is zero indexed for speed)
        let block_start = jxs_array.get(&DataBlockType::ESZ) - 1;
        // Calculate the block end index, see the ESZ description in the ACE spec
        let num_energies = nxs_array.nes;
        let block_length = 5 * num_energies;
        let mut block_end = block_start + block_length;
        // Avoid issues if this is the last block in the file
        if block_end == xxs_array.len() + 1 {
            block_end -= 1;
        }
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl<'a> Process<'a> for ESZ {
    type Dependencies = &'a NxsArray;

    fn process(data: &[f64], nxs_array: &NxsArray) -> Self {
        let num_energy_points = nxs_array.nes;
        // Energy grid
        let energy = data[0..num_energy_points].to_vec();
        // Total cross section
        let total_xs = data[num_energy_points..2 * num_energy_points].to_vec();
        // Dissapearence cross section
        let dissapearance_xs = data[2 * num_energy_points..3 * num_energy_points].to_vec();
        // Elastic cross section
        let elastic_xs = data[3 * num_energy_points..4 * num_energy_points].to_vec();
        // Average heating numbers
        let average_heating_numbers = data[4 * num_energy_points..5 * num_energy_points].to_vec();
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