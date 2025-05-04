use std::error::Error;

use crate::ace::binary_format::AceBinaryMmap;

// Indices for different values within NXS array.
// See the ACE format spec for a description.
#[derive(Debug)]
enum NxsIndex {
    XxsLen = 0,
    Za = 1,
    Nes = 2,
    Ntr = 3,
    Nr = 4,
    Ntrp = 5,
    Ntype = 6,
    Npcr = 7,
    S = 8,
    Z = 9,
    A = 10,
}

// Represents the NXS array from an ACE file. See the ACE format spec for a description.
// The NXS array contains critical information on the structure of the main XXS data array to
// follow.
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
    pub fn from_file(mmap: &AceBinaryMmap) -> Result<Self, Box<dyn Error>> {
        // Zero-copy Conversion to usize from memory mapped file
        let nxs_array = mmap.nxs_array();

        Ok(Self {
            xxs_len: nxs_array[NxsIndex::XxsLen as usize],
            za: nxs_array[NxsIndex::Za as usize],
            nes: nxs_array[NxsIndex::Nes as usize],
            ntr: nxs_array[NxsIndex::Ntr as usize],
            nr: nxs_array[NxsIndex::Nr as usize],
            ntrp: nxs_array[NxsIndex::Ntrp as usize],
            ntype: nxs_array[NxsIndex::Ntype as usize],
            npcr: nxs_array[NxsIndex::Npcr as usize],
            s: nxs_array[NxsIndex::S as usize],
            z: nxs_array[NxsIndex::Z as usize],
            a: nxs_array[NxsIndex::A as usize],
        })
    }

    // Pull the value from a parsed NXS array by its index
    // Note that this is the 1-indexed value from the ACE spec
    pub fn value_at_index(&self, index: usize) -> Option<usize> {
        match index {
            1 => Some(self.xxs_len),
            2 => Some(self.za),
            3 => Some(self.nes),
            4 => Some(self.ntr),
            5 => Some(self.nr),
            6 => Some(self.ntrp),
            7 => Some(self.ntype),
            8 => Some(self.npcr),
            9 => Some(self.s),
            10 => Some(self.z),
            11 => Some(self.a),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_at_index() {
        let nxs = NxsArray {
            xxs_len: 86843,
            za: 5010,
            nes: 941,
            ntr: 55,
            nr: 35,
            ntrp: 38,
            ntype: 2,
            npcr: 0,
            s: 0,
            z: 5,
            a: 10,
        };

        assert_eq!(nxs.value_at_index(1), Some(86843));
        assert_eq!(nxs.value_at_index(2), Some(5010));
        assert_eq!(nxs.value_at_index(3), Some(941));
        assert_eq!(nxs.value_at_index(4), Some(55));
        assert_eq!(nxs.value_at_index(5), Some(35));
        assert_eq!(nxs.value_at_index(6), Some(38));
        assert_eq!(nxs.value_at_index(7), Some(2));
        assert_eq!(nxs.value_at_index(8), Some(0));
        assert_eq!(nxs.value_at_index(9), Some(0));
        assert_eq!(nxs.value_at_index(10), Some(5));
        assert_eq!(nxs.value_at_index(11), Some(10));
        assert_eq!(nxs.value_at_index(12), None);
    }
}