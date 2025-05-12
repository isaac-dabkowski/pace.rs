use std::ops::Deref;
use std::sync::Mutex;
use std::collections::HashMap;

use rayon::prelude::*;

use crate::helpers::reaction_type_from_MT;
use crate::arrays::Arrays;
use crate::blocks::{BlockType, ESZ, MTR, LSIG};
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// SIG data block
//
// Contains incident neutron cross section data for the ACE file. See
// the ACE format spec for a description of the SIG block.
//=====================================================================
#[derive(Debug, Clone)]
pub struct SIG ( pub SigCrossSectionMap );

impl Deref for SIG {
    type Target = SigCrossSectionMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for SIG {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We expect SIG if NXS(4) (NTR) != 0
        let has_xs_other_than_elastic = arrays.nxs.ntr != 0;

        // Get the starting index of the block in the XXS array
        let block_start = get_block_start(
            &BlockType::SIG,
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

        // Return the block's raw data as a slice
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

        let xs = Mutex::new(SigCrossSectionMap::default()); // Use Mutex for thread-safe access

        // Parallelize the loop over cross sections using par_iter()
        mtr.par_iter().zip(lsig.par_iter()).for_each(|(mt, start_pos)| {
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
            xs_lock.insert(*mt, SigCrossSection { mt: *mt, energy, xs_val });
        });

        Self(xs.into_inner().unwrap())
    }
}

impl std::fmt::Display for SIG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut sorted_xs: Vec<SigCrossSection> = self.values().cloned().collect();
        sorted_xs.sort_by_key(|xs| xs.mt);
        let xs_string = sorted_xs.iter()
            .map(|xs| format!("{}", xs))
            .collect::<Vec<String>>()
            .join(", ");
        write!(f, "SIG({})", xs_string)
    }
}

//=====================================================================
// Helper struct to represent a cross section.
//=====================================================================
type SigCrossSectionMap = HashMap<usize, SigCrossSection>;

#[derive(Debug, Clone)]
pub struct SigCrossSection {
    pub mt: usize,
    pub energy: Vec<f64>,
    pub xs_val: Vec<f64>,
}

impl<'a> std::fmt::Display for SigCrossSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CrossSection(MT={} {})", self.mt, reaction_type_from_MT(self.mt))
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_sig_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let sig = parsed_ace.data_blocks.SIG.unwrap();
        assert!(sig.contains_key(&18));

        let fission_xs = sig.get(&18).unwrap();
        assert_eq!(fission_xs.energy.len(), 3);
        assert_eq!(fission_xs.xs_val.len(), fission_xs.energy.len());
        assert_eq!(fission_xs.energy, vec![1.0, 2.0, 3.0]);
        assert_eq!(fission_xs.xs_val, vec![17.0, 38.0, 100.0]);
    }
}