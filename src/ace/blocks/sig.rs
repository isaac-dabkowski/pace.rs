// Represents the SIG data block - this contains incident neutron cross section data

// See page 17 of the ACE format spec for a description of the SIG block
use std::{collections::HashMap, iter::zip};

use crate::helpers::reaction_type_from_MT;
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::{DataBlockType, ESZ, MTR, LSIG};

type MT = usize;
type CrossSectionMap = HashMap<MT, CrossSection>;

#[derive(Debug, Clone)]
pub struct CrossSection {
    pub mt: usize,
    pub energy: Vec<f64>,
    pub xs_val: Vec<f64>
}

impl std::fmt::Display for CrossSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CrossSection(MT={} {})", self.mt, reaction_type_from_MT(self.mt))
    }
}

#[derive(Debug, Clone)]
pub struct SIG {
    pub xs: CrossSectionMap
}

impl SIG {
    pub fn process(text_data: Vec<String>, mtr: MTR, lsig: LSIG, esz: ESZ) -> Self {
        let mut xs = CrossSectionMap::default();

        // Loop over cross sections
        for (mt, start_pos) in zip(mtr.reaction_types, lsig.xs_locs) {
            // Get the first position in the energy grid where we have a cross section value.
            // This is simply the first entry in the section of the SIG block which corresponds
            // to our cross section.
            let energy_start_index: usize = text_data[start_pos - 1].parse().unwrap();
            // Get the number of entries we have for the cross section
            let num_xs_values: usize = text_data[start_pos].parse().unwrap();

            // Get the cross section values
            let xs_val: Vec<f64> = text_data[start_pos+1..start_pos+1+num_xs_values]
                .iter()
                .map(|val| val.parse().unwrap())
                .collect();

            // Get the corresponding energy values
            let energy: Vec<f64> = esz.energy[energy_start_index - 1..(energy_start_index - 1 + num_xs_values)].to_vec();

            // Construct the cross section and add it to our map
            xs.insert(mt, CrossSection{ mt, energy, xs_val });
        }
        Self { xs }
    }

    // Pull n SIG block from a XXS array
    pub fn pull_from_ascii_xxs_array<'a>(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [&str]) -> &'a [&'a str] {
        // Block start index
        let block_start = jxs_array.get(&DataBlockType::SIG);

        // Loop over the number of cross sections
        let mut current_offset: usize = 1;
        for _ in 0..nxs_array.ntr {
            // Get the number of energy points in the cross section
            let num_entries: usize = xxs_array[block_start + current_offset].trim().parse().unwrap();
            // Jump forward to the next cross section
            current_offset += num_entries + 2;
        }
        // Calculate the block end index, see the SIG description in the ACE spec
        let block_end = block_start + current_offset;
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl std::fmt::Display for SIG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_xs: Vec<CrossSection> = self.xs.values().cloned().collect();
        sorted_xs.sort_by_key(|xs| xs.mt);
        let xs_string = sorted_xs.iter()
            .map(|xs| format!("{}", xs))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "SIG({})", xs_string)
    }
}