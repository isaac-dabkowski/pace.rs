#![allow(dead_code)]

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use crate::ace::utils;
use crate::ace::arrays::NxsArray;

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

impl BlockType {
    fn from_jxs_index(value: usize) -> Option<Self> {
        match value {
            0 => Some(BlockType::ESZ),
            1 => Some(BlockType::NU),
            2 => Some(BlockType::MTR),
            3 => Some(BlockType::LQR),
            4 => Some(BlockType::TYR),
            5 => Some(BlockType::LSIG),
            6 => Some(BlockType::SIG),
            7 => Some(BlockType::LAND),
            8 => Some(BlockType::AND),
            9 => Some(BlockType::LDLW),
            10 => Some(BlockType::DLW),
            11 => Some(BlockType::GPD),
            12 => Some(BlockType::MTRP),
            13 => Some(BlockType::LSIGP),
            14 => Some(BlockType::SIGP),
            15 => Some(BlockType::LANDP),
            16 => Some(BlockType::ANDP),
            17 => Some(BlockType::LDLWP),
            18 => Some(BlockType::DLWP),
            19 => Some(BlockType::YP),
            20 => Some(BlockType::FIS),
            21 => Some(BlockType::END),
            22 => Some(BlockType::LUND),
            23 => Some(BlockType::DNU),
            24 => Some(BlockType::BDD),
            25 => Some(BlockType::DNEDL),
            26 => Some(BlockType::DNED),
            29 => Some(BlockType::PTYPE),
            30 => Some(BlockType::NTRO),
            31 => Some(BlockType::NEXT),
            _ => None, // Return None for invalid integers
        }
    }
}

use crate::ace::blocks::BlockType;
// Represents the complete JXS array from an ACE file
#[derive(Clone, Debug, Default)]
pub struct JxsArray {
    pub block_bounds: HashMap<BlockType, Option<JxsEntry>>
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
            if let Some(block_type) = BlockType::from_jxs_index(i) {
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
    type Target = HashMap<BlockType, Option<JxsEntry>>;

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
        assert_eq!(jxs.get(&BlockType::ESZ).unwrap(), &Some(JxsEntry::new(1, 2)));
        assert_eq!(jxs.get(&BlockType::NU).unwrap(), &None);
        assert_eq!(jxs.get(&BlockType::MTR).unwrap(), &Some(JxsEntry::new(3, 1)));
        assert_eq!(jxs.get(&BlockType::LQR).unwrap(), &Some(JxsEntry::new(4, 1)));
        assert_eq!(jxs.get(&BlockType::TYR).unwrap(), &Some(JxsEntry::new(5, 1)));
        assert_eq!(jxs.get(&BlockType::LSIG).unwrap(), &Some(JxsEntry::new(6, 1)));
        assert_eq!(jxs.get(&BlockType::SIG).unwrap(), &Some(JxsEntry::new(7, 1)));
        assert_eq!(jxs.get(&BlockType::LAND).unwrap(), &Some(JxsEntry::new(8, 1)));
        assert_eq!(jxs.get(&BlockType::AND).unwrap(), &Some(JxsEntry::new(9, 1)));
        assert_eq!(jxs.get(&BlockType::LDLW).unwrap(), &Some(JxsEntry::new(10, 4)));
        assert_eq!(jxs.get(&BlockType::DLW).unwrap(), &None);
        assert_eq!(jxs.get(&BlockType::GPD).unwrap(), &None);
        assert_eq!(jxs.get(&BlockType::MTRP).unwrap(), &None);
        assert_eq!(jxs.get(&BlockType::LSIGP).unwrap(), &Some(JxsEntry::new(14, 1)));
        assert_eq!(jxs.get(&BlockType::SIGP).unwrap(), &Some(JxsEntry::new(15, 1)));
        assert_eq!(jxs.get(&BlockType::LANDP).unwrap(), &Some(JxsEntry::new(16, 1)));
        assert_eq!(jxs.get(&BlockType::ANDP).unwrap(), &Some(JxsEntry::new(17, 1)));
        assert_eq!(jxs.get(&BlockType::LDLWP).unwrap(), &Some(JxsEntry::new(18, 1)));
        assert_eq!(jxs.get(&BlockType::DLWP).unwrap(), &Some(JxsEntry::new(19, 1)));
        assert_eq!(jxs.get(&BlockType::YP).unwrap(), &Some(JxsEntry::new(20, 1)));
        assert_eq!(jxs.get(&BlockType::FIS).unwrap(), &Some(JxsEntry::new(21, 1)));
        assert_eq!(jxs.get(&BlockType::END).unwrap(), &Some(JxsEntry::new(22, 1)));
        assert_eq!(jxs.get(&BlockType::LUND).unwrap(), &Some(JxsEntry::new(23, 1)));
        assert_eq!(jxs.get(&BlockType::DNU).unwrap(), &Some(JxsEntry::new(24, 1)));
        assert_eq!(jxs.get(&BlockType::BDD).unwrap(), &Some(JxsEntry::new(25, 1)));
        assert_eq!(jxs.get(&BlockType::DNEDL).unwrap(), &Some(JxsEntry::new(26, 1)));
        assert_eq!(jxs.get(&BlockType::DNED).unwrap(), &Some(JxsEntry::new(27, 1)));
        assert_eq!(jxs.get(&BlockType::PTYPE).unwrap(), &Some(JxsEntry::new(30, 1)));
        assert_eq!(jxs.get(&BlockType::NTRO).unwrap(), &Some(JxsEntry::new(31, 1)));
        assert_eq!(jxs.get(&BlockType::NEXT).unwrap(), &Some(JxsEntry::new(32, 68)));
    }
}