use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use crate::ace::utils;

// Represents the NXS array from an ACE file. See page 10 of the ACE format spec for a description.
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
    pub fn from_ascii_file(reader: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        // A NXS array consists of 2 lines, each with eight integers.
        let nxs_array_text = utils::read_lines(reader, 2)?;

        let nxs_array: Vec<usize> = nxs_array_text
            .iter()
            .flat_map(
                |s| {
                    s.split_whitespace() 
                        .map(|num| num.parse::<usize>())
                        .filter_map(Result::ok)
                    }
                )
            .collect();

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
mod ascii_tests {
    use super::*;

    #[test]
    fn test_nxs_parsing() {
        // Simulate ACE NXS array
        let nxs_text = concat!(
            "    86843     5010      941       55       35       38        2        0\n",
            "        0        5       10        0        0        0        0        0\n"
        );
        let mut reader = utils::create_reader_from_string(nxs_text);

        // Parse the array
        let nxs = NxsArray::from_ascii_file(&mut reader).expect("Failed to parse NXS array");

        // Check fields
        let expected_nxs = NxsArray {
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
        assert_eq!(nxs, expected_nxs);
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