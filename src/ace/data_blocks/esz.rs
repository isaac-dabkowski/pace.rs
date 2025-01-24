use crate::ace::data_blocks::block_processor::BlockConstruction;

// Represents the ESZ data block
#[derive(Debug, Clone, PartialEq)]
pub struct ESZ {
    pub energy: Vec<f64>,
    pub total_xs: Vec<f64>,
    pub dissapearance_xs: Vec<f64>,
    pub elastic_xs: Vec<f64>,
    pub average_heating_numbers: Vec<f64>,
}

impl BlockConstruction for ESZ {
    // See page 12 of the ACE format spec for a description of the ESZ block
    fn construct(text_data: Vec<String>, nxs_array: &crate::ace::arrays::NxsArray) -> Self {
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
        // Dissaeparence cross section
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
}

#[cfg(test)]
mod ascii_tests {
    use crate::ace::utils;

    #[test]
    fn test_esz_parsing() {
        let parsed_ace = utils::get_parsed_ascii_for_testing();
        let esz = parsed_ace.data_blocks.ESZ.as_ref().unwrap();
        assert_eq!(esz.energy.len(), parsed_ace.num_energies());
        assert_eq!(esz.total_xs.len(), parsed_ace.num_energies());
        assert_eq!(esz.dissapearance_xs.len(), parsed_ace.num_energies());
        assert_eq!(esz.elastic_xs.len(), parsed_ace.num_energies());
        assert_eq!(esz.average_heating_numbers.len(), parsed_ace.num_energies());
    }
}