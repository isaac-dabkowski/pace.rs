use anyhow::Result;

use crate::utils::PaceMmap;

//=====================================================================
// Represents the NXS array from an ACE file. See the ACE format spec
// for a description.  The NXS array contains critical information on
// the structure of the main XXS data array to follow.
//=====================================================================
#[derive(Clone, Debug, PartialEq)]
pub struct NxsArray {
    pub xxs_len: usize, // Number of entries in XXS array
    pub za: usize,      // ZA of isotope
    pub nes: usize,     // Number of energies
    pub ntr: usize,     // Number of reactions excluding elastic scattering
    pub nr: usize,      // Number of reactions having secondary neutrons excluding elastic scattering
    pub ntrp: usize,    // Number of photon production reactions
    pub ntype: usize,   // Number of particle types for which production data is given
    pub npcr: usize,    // Number of delayed neutron precurser families
    pub s: usize,       // Excited state (>2.0.0 Header only)
    pub z: usize,       // Atomic number (>2.0.0 Header only)
    pub a: usize,       // Atomic mass number (>2.0.0 Header only)
}

impl NxsArray {
    pub fn from_PACE(mmap: &PaceMmap) -> Result<Self> {
        let nxs_array: &[usize] = mmap.nxs_array();

        Ok(Self {
            xxs_len: nxs_array[0],
            za: nxs_array[1],
            nes: nxs_array[2],
            ntr: nxs_array[3],
            nr: nxs_array[4],
            ntrp: nxs_array[5],
            ntype: nxs_array[6],
            npcr: nxs_array[7],
            s: nxs_array[8],
            z: nxs_array[9],
            a: nxs_array[10],
        })
    }
}
