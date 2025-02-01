// Represents the ESZ data block - this is the energy grid for our ACE file, along with several
// basic cross sections.
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::DataBlockType;

// See page 12 of the ACE format spec for a description of the ESZ block
#[derive(Debug, Clone, PartialEq)]
pub struct ESZ {
    pub energy: Vec<f64>,
    pub total_xs: Vec<f64>,
    pub dissapearance_xs: Vec<f64>,
    pub elastic_xs: Vec<f64>,
    pub average_heating_numbers: Vec<f64>,
}

impl ESZ {
    pub fn process(text_data: Vec<String>, nxs_array: &NxsArray) -> Self {
        let num_energy_points = nxs_array.nes;
        // Energy grid
        let energy: Vec<f64> = text_data[0..num_energy_points]
            .iter()
            .map(|val| val.parse().unwrap())
            .collect();
        // Total cross section
        let total_xs: Vec<f64> = text_data[num_energy_points..2 * num_energy_points]
            .iter()
            .map(|val| val.parse().unwrap())
            .collect();
        // Dissapearence cross section
        let dissapearance_xs: Vec<f64> = text_data[2 * num_energy_points..3 * num_energy_points]
            .iter()
            .map(|val| val.parse().unwrap())
            .collect();
        // Elastic cross section
        let elastic_xs: Vec<f64> = text_data[3 * num_energy_points..4 * num_energy_points]
            .iter()
            .map(|val| val.parse().unwrap())
            .collect();
        // Average heating numbers
        let average_heating_numbers: Vec<f64> = text_data[4 * num_energy_points..5 * num_energy_points]
            .iter()
            .map(|val| val.parse().unwrap())
            .collect();
        Self {
            energy,
            total_xs,
            dissapearance_xs,
            elastic_xs,
            average_heating_numbers,
        }
    }

    // Pull an ESZ block from a XXS array
    pub fn pull_from_ascii_xxs_array<'a>(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [&str]) -> &'a [&'a str] {
        // Block start index
        let block_start = jxs_array.get(&DataBlockType::ESZ);
        // Calculate the block end index, see the ESZ description in the ACE spec
        let num_energies = nxs_array.nes;
        let block_length = 5 * num_energies;
        let block_end = block_start + block_length;
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl std::fmt::Display for ESZ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ESZ({} energies)", self.energy.len())
    }
}
