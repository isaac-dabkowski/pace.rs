// Represents the SIG data block - this contains incident neutron cross section data

use std::sync::Mutex;
// See page 17 of the ACE format spec for a description of the SIG block
use std::collections::HashMap;

use rayon::prelude::*;

use crate::helpers::reaction_type_from_MT;
use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::{DataBlockType, ESZ, MTR, LSIG};
use crate::ace::blocks::block_traits::{PullFromXXS, Process};

type MT = usize;
type CrossSectionMap = HashMap<MT, CrossSection>;

#[derive(Debug, Clone)]
pub struct CrossSection {
    pub mt: MT,
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

impl<'a> PullFromXXS<'a> for SIG {
    fn pull_from_xxs_array(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64] {
        // Block start index (binary XXS is zero indexed for speed)
        let block_start = jxs_array.get(&DataBlockType::SIG) - 1;

        // Loop over the number of cross sections
        let mut current_offset: usize = 1;
        for _ in 0..nxs_array.ntr {
            // Get the number of energy points in the cross section
            let num_entries = xxs_array[block_start + current_offset].to_bits() as usize;
            // Jump forward to the next cross section
            current_offset += num_entries + 2;
        }
        // Calculate the block end index, see the SIG description in the ACE spec
        let mut block_end = block_start + current_offset;
        // Avoid issues if this is the last block in the file
        if block_end == xxs_array.len() + 1 {
            block_end -= 1;
        }
        // Return the block
        &xxs_array[block_start..block_end]
    }
}

impl<'a> Process<'a> for SIG {
    type Dependencies = (&'a MTR, &'a LSIG, &'a ESZ);

    fn process(data: &[f64], dependencies: (&MTR, &LSIG, &ESZ)) -> Self {
        let (mtr, lsig, esz) = dependencies;

        let xs = Mutex::new(CrossSectionMap::default()); // Use Mutex for thread-safe access

        // Parallelize the loop over cross sections using par_iter()
        mtr.reaction_types.par_iter().zip(lsig.xs_locs.par_iter()).for_each(|(mt, start_pos)| {
            // Get the first position in the energy grid where we have a cross section value
            let energy_start_index: usize = data[start_pos - 1].to_bits() as usize;
            // Get the number of entries we have for the cross section
            let num_xs_values: usize = data[*start_pos].to_bits() as usize;

            // Get the cross section values
            let xs_val: Vec<f64> = data[start_pos + 1..start_pos + 1 + num_xs_values].to_vec();
            // Get the corresponding energy values
            let energy: Vec<f64> = esz.energy[energy_start_index - 1..(energy_start_index - 1 + num_xs_values)].to_vec();
        
            // Lock the Mutex and insert into the CrossSectionMap
            let mut xs_lock = xs.lock().unwrap();
            xs_lock.insert(*mt, CrossSection { mt: *mt, energy, xs_val });
        });

        Self {
            xs: xs.into_inner().unwrap(), // Access the final xs map
        }
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

#[cfg(test)]
mod tests {
    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_sig_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let sig = parsed_ace.data_blocks.SIG.unwrap();
        assert!(sig.xs.contains_key(&18));

        let fission_xs = sig.xs.get(&18).unwrap();
        assert_eq!(fission_xs.energy.len(), 3);
        assert_eq!(fission_xs.xs_val.len(), fission_xs.energy.len());
        assert_eq!(fission_xs.energy, vec![1.0, 2.0, 3.0]);
        assert_eq!(fission_xs.xs_val, vec![17.0, 38.0, 100.0]);
    }
}