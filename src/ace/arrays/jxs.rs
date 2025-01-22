#![allow(dead_code)]

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use crate::ace::utils;
use crate::ace::arrays::NxsArray;
use crate::ace::data_blocks::DataBlockType;

// Represents an entry within the JXS array containing location and length of a data block
#[derive(Debug, Clone, PartialEq)]
pub struct JxsEntry {
    pub loc: usize,  // Starting location of the data block
    pub len: usize,  // Length of the data block
}

impl JxsEntry {
    /// Creates a new JxsEntry from location and length values
    fn new(loc: usize, len: usize) -> Self {
        Self { loc, len }
    }

    /// Creates an Option<JxsEntry> from a pair of values, returning None if loc is 0
    fn from_pair(loc: usize, len: usize) -> Option<Self> {
        if loc == 0 {
            None
        } else {
            Some(Self::new(loc, len))
        }
    }
}

impl DataBlockType {
    fn from_jxs_index(value: usize) -> Option<Self> {
        match value {
            0 => Some(DataBlockType::ESZ),
            1 => Some(DataBlockType::NU),
            2 => Some(DataBlockType::MTR),
            3 => Some(DataBlockType::LQR),
            4 => Some(DataBlockType::TYR),
            5 => Some(DataBlockType::LSIG),
            6 => Some(DataBlockType::SIG),
            7 => Some(DataBlockType::LAND),
            8 => Some(DataBlockType::AND),
            9 => Some(DataBlockType::LDLW),
            10 => Some(DataBlockType::DLW),
            11 => Some(DataBlockType::GPD),
            12 => Some(DataBlockType::MTRP),
            13 => Some(DataBlockType::LSIGP),
            14 => Some(DataBlockType::SIGP),
            15 => Some(DataBlockType::LANDP),
            16 => Some(DataBlockType::ANDP),
            17 => Some(DataBlockType::LDLWP),
            18 => Some(DataBlockType::DLWP),
            19 => Some(DataBlockType::YP),
            20 => Some(DataBlockType::FIS),
            21 => Some(DataBlockType::END),
            22 => Some(DataBlockType::LUND),
            23 => Some(DataBlockType::DNU),
            24 => Some(DataBlockType::BDD),
            25 => Some(DataBlockType::DNEDL),
            26 => Some(DataBlockType::DNED),
            29 => Some(DataBlockType::PTYPE),
            30 => Some(DataBlockType::NTRO),
            31 => Some(DataBlockType::NEXT),
            _ => None, // Return None for invalid integers
        }
    }
}


// Represents the complete JXS array from an ACE file
#[derive(Clone, Debug, Default)]
pub struct JxsArray {
    pub block_bounds: HashMap<DataBlockType, Option<JxsEntry>>
}

impl JxsArray {
    // Creates a new JxsArray from an ASCII file reader and NXS array information.
    pub fn from_ascii_file(reader: &mut BufReader<File>, nxs_array: &NxsArray) -> Result<Self, Box<dyn Error>> {
        let mut jxs_array = JxsArray::default();
        // A JXS array consists of 4 lines, each with eight integers.
        let jxs_array_text = utils::read_lines(reader, 4)?;

        // Parse to integers
        let parsed_jxs_array: Vec<usize> = jxs_array_text
            .iter()
            .flat_map(
                |s| {
                    s.split_whitespace() 
                        .map(|num| num.parse::<usize>())
                        .filter_map(Result::ok)
                    }
                )
            .collect();

        // Fill in our array
        for i in 0..32 {
            // Skip any indices we do not have support for currently
            if let Some(block_type) = DataBlockType::from_jxs_index(i) {
                let loc = parsed_jxs_array[i];
                match loc == 0 {
                    // Block does not exist
                    true => {
                        jxs_array.insert(block_type, None);
                    },
                    // Block exists
                    false => {
                        // Loop forward to find the length of the block
                        let mut next_i = i + 1;
                        while next_i < 32 && parsed_jxs_array[next_i] == 0 {
                            next_i += 1;
                        }
                        let len = if next_i != 32 {
                            parsed_jxs_array[next_i] - loc
                        } else {
                            nxs_array.xxs_len - loc
                        };
                        jxs_array.insert(block_type, Some(JxsEntry::new(loc, len)));
                    }
                }
            }
        }
        Ok(jxs_array)
    }
}

impl Deref for JxsArray {
    type Target = HashMap<DataBlockType, Option<JxsEntry>>;

    fn deref(&self) -> &Self::Target {
        &self.block_bounds
    }
}

impl DerefMut for JxsArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block_bounds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jxs_entry_new() {
        let entry = JxsEntry::new(10, 20);
        assert_eq!(entry.loc, 10);
        assert_eq!(entry.len, 20);
    }

    #[test]
    fn test_jxs_parsing() {
        // Simulate ACE JXS array
        let jxs_text = concat!(
            "    1    0    3    4    5    6    7    8\n",
            "    9   10    0    0    0   14   15   16\n",
            "   17   18   19   20   21   22   23   24\n",
            "   25   26   27   28   29   30   31   32\n"
        );
        let mut reader = utils::create_reader_from_string(jxs_text);

        // Simulate NXS array
        let nxs = NxsArray {
            xxs_len: 100,
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

        // Parse the array
        let jxs = JxsArray::from_ascii_file(&mut reader, &nxs).expect("Failed to parse JXS array");

        // Check fields
        assert_eq!(jxs.get(&DataBlockType::ESZ).unwrap(), &Some(JxsEntry::new(1, 2)));
        assert_eq!(jxs.get(&DataBlockType::NU).unwrap(), &None);
        assert_eq!(jxs.get(&DataBlockType::MTR).unwrap(), &Some(JxsEntry::new(3, 1)));
        assert_eq!(jxs.get(&DataBlockType::LQR).unwrap(), &Some(JxsEntry::new(4, 1)));
        assert_eq!(jxs.get(&DataBlockType::TYR).unwrap(), &Some(JxsEntry::new(5, 1)));
        assert_eq!(jxs.get(&DataBlockType::LSIG).unwrap(), &Some(JxsEntry::new(6, 1)));
        assert_eq!(jxs.get(&DataBlockType::SIG).unwrap(), &Some(JxsEntry::new(7, 1)));
        assert_eq!(jxs.get(&DataBlockType::LAND).unwrap(), &Some(JxsEntry::new(8, 1)));
        assert_eq!(jxs.get(&DataBlockType::AND).unwrap(), &Some(JxsEntry::new(9, 1)));
        assert_eq!(jxs.get(&DataBlockType::LDLW).unwrap(), &Some(JxsEntry::new(10, 4)));
        assert_eq!(jxs.get(&DataBlockType::DLW).unwrap(), &None);
        assert_eq!(jxs.get(&DataBlockType::GPD).unwrap(), &None);
        assert_eq!(jxs.get(&DataBlockType::MTRP).unwrap(), &None);
        assert_eq!(jxs.get(&DataBlockType::LSIGP).unwrap(), &Some(JxsEntry::new(14, 1)));
        assert_eq!(jxs.get(&DataBlockType::SIGP).unwrap(), &Some(JxsEntry::new(15, 1)));
        assert_eq!(jxs.get(&DataBlockType::LANDP).unwrap(), &Some(JxsEntry::new(16, 1)));
        assert_eq!(jxs.get(&DataBlockType::ANDP).unwrap(), &Some(JxsEntry::new(17, 1)));
        assert_eq!(jxs.get(&DataBlockType::LDLWP).unwrap(), &Some(JxsEntry::new(18, 1)));
        assert_eq!(jxs.get(&DataBlockType::DLWP).unwrap(), &Some(JxsEntry::new(19, 1)));
        assert_eq!(jxs.get(&DataBlockType::YP).unwrap(), &Some(JxsEntry::new(20, 1)));
        assert_eq!(jxs.get(&DataBlockType::FIS).unwrap(), &Some(JxsEntry::new(21, 1)));
        assert_eq!(jxs.get(&DataBlockType::END).unwrap(), &Some(JxsEntry::new(22, 1)));
        assert_eq!(jxs.get(&DataBlockType::LUND).unwrap(), &Some(JxsEntry::new(23, 1)));
        assert_eq!(jxs.get(&DataBlockType::DNU).unwrap(), &Some(JxsEntry::new(24, 1)));
        assert_eq!(jxs.get(&DataBlockType::BDD).unwrap(), &Some(JxsEntry::new(25, 1)));
        assert_eq!(jxs.get(&DataBlockType::DNEDL).unwrap(), &Some(JxsEntry::new(26, 1)));
        assert_eq!(jxs.get(&DataBlockType::DNED).unwrap(), &Some(JxsEntry::new(27, 1)));
        assert_eq!(jxs.get(&DataBlockType::PTYPE).unwrap(), &Some(JxsEntry::new(30, 1)));
        assert_eq!(jxs.get(&DataBlockType::NTRO).unwrap(), &Some(JxsEntry::new(31, 1)));
        assert_eq!(jxs.get(&DataBlockType::NEXT).unwrap(), &Some(JxsEntry::new(32, 68)));
    }
}