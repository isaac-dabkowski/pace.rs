use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use crate::ace::utils;

// Represents the NXS array from an ACE file. See page 10 of the ACE format spec for a description.
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