// Represents the SIG data block - this contains incident neutron cross section data

use std::sync::Mutex;
// See the ACE format spec for a description of the SIG block
use std::collections::HashMap;
use std::time::Instant;

use rayon::prelude::*;

use crate::helpers::reaction_type_from_MT;
use crate::ace::arrays::Arrays;
use crate::ace::blocks::block_types::MT;
use crate::ace::blocks::{DataBlockType, ESZ, MTR, LSIG};
use crate::ace::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

type CrossSectionMap = HashMap<MT, CrossSection>;

#[derive(Debug, Clone)]
pub struct CrossSection {
    pub mt: MT,
    pub energy: Vec<f64>,
    pub xs_val: Vec<f64>,
}

impl<'a> std::fmt::Display for CrossSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CrossSection(MT={} {})", self.mt, reaction_type_from_MT(self.mt))
    }
}

#[derive(Debug, Clone)]
pub struct SIG {
    pub xs: CrossSectionMap
}

impl<'a> PullFromXXS<'a> for SIG {
    fn pull_from_xxs_array(has_xs_other_than_elastic: bool, arrays: &'a Arrays) -> Option<&'a [f64]> {
        // If the block type's start index is non-zero, the block is present in the XXS array
        // We expect SIG if NXS(4) (NTR) != 0
        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &DataBlockType::SIG,
            arrays,
            has_xs_other_than_elastic,
            "SIG is expected if NXS(4) (NTR) != 0, but SIG was not found.".to_string(),
        )?;

        // Calculate the block length, see the SIG description in the ACE spec
        // Loop over the number of cross sections
        let mut block_length: usize = 1;
        for _ in 0..arrays.nxs.ntr {
            // Get the number of energy points in the cross section
            let num_entries = arrays.xxs[block_start + block_length].to_bits() as usize;
            // Jump forward to the next cross section
            block_length += num_entries + 2;
        }

        // Return the block's raw data as a vector
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for SIG {
    type Dependencies = (&'a Option<MTR>, &'a Option<LSIG>, &'a Option<ESZ>);

    fn process(data: &[f64], _arrays: &Arrays, dependencies: (&Option<MTR>, &Option<LSIG>, &Option<ESZ>)) -> Self {
        let (mtr, lsig, esz) = (
            dependencies.0.as_ref().unwrap(),
            dependencies.1.as_ref().unwrap(),
            dependencies.2.as_ref().unwrap(),
        );

        let xs = Mutex::new(CrossSectionMap::default()); // Use Mutex for thread-safe access

        // Parallelize the loop over cross sections using par_iter()
        mtr.reaction_types.par_iter().zip(lsig.xs_locs.par_iter()).for_each(|(mt, start_pos)| {
            // Get the first position in the energy grid where we have a cross section value
            let energy_start_index: usize = data[start_pos - 1].to_bits() as usize;
            // Get the number of entries we have for the cross section
            let num_xs_values: usize = data[*start_pos].to_bits() as usize;

            // Get the cross section values
            let xs_val = Vec::from(&data[start_pos + 1..start_pos + 1 + num_xs_values]);
            // Get the corresponding energy values
            let energy = Vec::from(&esz.energy[energy_start_index - 1..(energy_start_index - 1 + num_xs_values)]);
        
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