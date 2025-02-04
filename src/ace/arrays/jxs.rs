use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use strum::IntoEnumIterator;

use crate::ace::utils;
use crate::ace::blocks::DataBlockType;


// Represents the complete JXS array from an ACE file
#[derive(Clone, Debug, Default)]
pub struct JxsArray {
    pub block_starting_indices: HashMap<DataBlockType, usize>
}

impl Deref for JxsArray {
    type Target = HashMap<DataBlockType, usize>;

    fn deref(&self) -> &Self::Target {
        &self.block_starting_indices
    }
}

impl DerefMut for JxsArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block_starting_indices
    }
}

impl JxsArray {
    pub fn get(&self, key: &DataBlockType) -> usize {
        *self.block_starting_indices.get(key).unwrap_or_else(|| panic!("Could not find {} in JXS array", key))
    }
}

impl JxsArray {
    // Creates a new JxsArray from an ASCII file reader and NXS array information.
    pub fn from_ascii_file(reader: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        let mut jxs_array = JxsArray::default();
        // A JXS array consists of 4 lines, each with eight integers.
        let jxs_array_text = utils::read_lines(reader, 4)?;

        // Parse to integers
        let jxs_array_entries: Vec<usize> = jxs_array_text
            .iter()
            .flat_map(
                |s| {
                    s.split_whitespace() 
                        .map(|num| num.parse::<usize>())
                        .filter_map(Result::ok)
                    }
                )
            .collect();
        
        // Fill in our array by looping over all DataBlockTypes
        for block_type in DataBlockType::iter() {
            // Get the index at which we should store the value
            let jxs_index = JxsArray::index_from_data_block_type(&block_type);
            jxs_array.insert(block_type, jxs_array_entries[jxs_index]);
        }

        Ok(jxs_array)
    }

    // Creates a new JxsArray from an ASCII file reader and NXS array information.
    pub fn from_binary_file(reader: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        let mut jxs_array = JxsArray::default();
        // A JXS array consists of 32 integers.
        let jxs_array_entries = utils::read_usizes(32, reader);

        // Fill in our array by looping over all DataBlockTypes
        for block_type in DataBlockType::iter() {
            // Get the index at which we should store the value
            let jxs_index = JxsArray::index_from_data_block_type(&block_type);
            jxs_array.insert(block_type, jxs_array_entries[jxs_index]);
        }

        Ok(jxs_array)
    }

    // For a given DataBlockType, return the index in the JXS array which lists
    // its starting index in the main XXS array.
    fn index_from_data_block_type(block_type: &DataBlockType) -> usize {
        match block_type {
            DataBlockType::ESZ =>  0,
            DataBlockType::NU =>  1,
            DataBlockType::MTR =>  2,
            DataBlockType::LQR =>  3,
            DataBlockType::TYR =>  4,
            DataBlockType::LSIG =>  5,
            DataBlockType::SIG =>  6,
            DataBlockType::LAND =>  7,
            DataBlockType::AND =>  8,
            DataBlockType::LDLW =>  9,
            DataBlockType::DLW => 10,
            DataBlockType::GPD => 11,
            DataBlockType::MTRP => 12,
            DataBlockType::LSIGP => 13,
            DataBlockType::SIGP => 14,
            DataBlockType::LANDP => 15,
            DataBlockType::ANDP => 16,
            DataBlockType::LDLWP => 17,
            DataBlockType::DLWP => 18,
            DataBlockType::YP => 19,
            DataBlockType::FIS => 20,
            DataBlockType::END => 21,
            DataBlockType::LUND => 22,
            DataBlockType::DNU => 23,
            DataBlockType::BDD => 24,
            DataBlockType::DNEDL => 25,
            DataBlockType::DNED => 26,
            DataBlockType::PTYPE => 29,
            DataBlockType::NTRO => 30,
            DataBlockType::NEXT => 31,
        }
    }

    // Pull the value from a parsed JXS array by its index
    // Note that this is the 1-indexed value from the ACE spec
    pub fn value_at_index(&self, index: usize) -> Option<usize> {
        match index {
            1 => Some(self.get(&DataBlockType::ESZ)),
            2 => Some(self.get(&DataBlockType::NU)),
            3 => Some(self.get(&DataBlockType::MTR)),
            4 => Some(self.get(&DataBlockType::LQR)),
            5 => Some(self.get(&DataBlockType::TYR)),
            6 => Some(self.get(&DataBlockType::LSIG)),
            7 => Some(self.get(&DataBlockType::SIG)),
            8 => Some(self.get(&DataBlockType::LAND)),
            9 => Some(self.get(&DataBlockType::AND)),
            10 => Some(self.get(&DataBlockType::LDLW)),
            11 => Some(self.get(&DataBlockType::DLW)),
            12 => Some(self.get(&DataBlockType::GPD)),
            13 => Some(self.get(&DataBlockType::MTRP)),
            14 => Some(self.get(&DataBlockType::LSIGP)),
            15 => Some(self.get(&DataBlockType::SIGP)),
            16 => Some(self.get(&DataBlockType::LANDP)),
            17 => Some(self.get(&DataBlockType::ANDP)),
            18 => Some(self.get(&DataBlockType::LDLWP)),
            19 => Some(self.get(&DataBlockType::DLWP)),
            20 => Some(self.get(&DataBlockType::YP)),
            21 => Some(self.get(&DataBlockType::FIS)),
            22 => Some(self.get(&DataBlockType::END)),
            23 => Some(self.get(&DataBlockType::LUND)),
            24 => Some(self.get(&DataBlockType::DNU)),
            25 => Some(self.get(&DataBlockType::BDD)),
            26 => Some(self.get(&DataBlockType::DNEDL)),
            27 => Some(self.get(&DataBlockType::DNED)),
            30 => Some(self.get(&DataBlockType::PTYPE)),
            31 => Some(self.get(&DataBlockType::NTRO)),
            32 => Some(self.get(&DataBlockType::NEXT)),
            _ => None
        }
    }
}

#[cfg(test)]
mod ascii_tests {
    use super::*;

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

        // Parse the array
        let jxs = JxsArray::from_ascii_file(&mut reader).expect("Failed to parse JXS array");

        // Check fields
        assert_eq!(jxs.get(&DataBlockType::ESZ), 1);
        assert_eq!(jxs.get(&DataBlockType::NU), 0);
        assert_eq!(jxs.get(&DataBlockType::MTR), 3);
        assert_eq!(jxs.get(&DataBlockType::LQR), 4);
        assert_eq!(jxs.get(&DataBlockType::TYR), 5);
        assert_eq!(jxs.get(&DataBlockType::LSIG), 6);
        assert_eq!(jxs.get(&DataBlockType::SIG), 7);
        assert_eq!(jxs.get(&DataBlockType::LAND), 8);
        assert_eq!(jxs.get(&DataBlockType::AND), 9);
        assert_eq!(jxs.get(&DataBlockType::LDLW), 10);
        assert_eq!(jxs.get(&DataBlockType::DLW), 0);
        assert_eq!(jxs.get(&DataBlockType::GPD), 0);
        assert_eq!(jxs.get(&DataBlockType::MTRP), 0);
        assert_eq!(jxs.get(&DataBlockType::LSIGP), 14);
        assert_eq!(jxs.get(&DataBlockType::SIGP), 15);
        assert_eq!(jxs.get(&DataBlockType::LANDP), 16);
        assert_eq!(jxs.get(&DataBlockType::ANDP), 17);
        assert_eq!(jxs.get(&DataBlockType::LDLWP), 18);
        assert_eq!(jxs.get(&DataBlockType::DLWP), 19);
        assert_eq!(jxs.get(&DataBlockType::YP), 20);
        assert_eq!(jxs.get(&DataBlockType::FIS), 21);
        assert_eq!(jxs.get(&DataBlockType::END), 22);
        assert_eq!(jxs.get(&DataBlockType::LUND), 23);
        assert_eq!(jxs.get(&DataBlockType::DNU), 24);
        assert_eq!(jxs.get(&DataBlockType::BDD), 25);
        assert_eq!(jxs.get(&DataBlockType::DNEDL), 26);
        assert_eq!(jxs.get(&DataBlockType::DNED), 27);
        assert_eq!(jxs.get(&DataBlockType::PTYPE), 30);
        assert_eq!(jxs.get(&DataBlockType::NTRO), 31);
        assert_eq!(jxs.get(&DataBlockType::NEXT), 32);
    }

    #[test]
    fn test_value_at_index() {
        // Simulate ACE JXS array
        let jxs_text = concat!(
            "    1    0    3    4    5    6    7    8\n",
            "    9   10    0    0    0   14   15   16\n",
            "   17   18   19   20   21   22   23   24\n",
            "   25   26   27   28   29   30   31   32\n"
        );
        let mut reader = utils::create_reader_from_string(jxs_text);

        // Parse the array
        let jxs = JxsArray::from_ascii_file(&mut reader).expect("Failed to parse JXS array");

        // Check values at specific indices
        assert_eq!(jxs.value_at_index(1), Some(1));
        assert_eq!(jxs.value_at_index(2), Some(0));
        assert_eq!(jxs.value_at_index(3), Some(3));
        assert_eq!(jxs.value_at_index(4), Some(4));
        assert_eq!(jxs.value_at_index(5), Some(5));
        assert_eq!(jxs.value_at_index(6), Some(6));
        assert_eq!(jxs.value_at_index(7), Some(7));
        assert_eq!(jxs.value_at_index(8), Some(8));
        assert_eq!(jxs.value_at_index(9), Some(9));
        assert_eq!(jxs.value_at_index(10), Some(10));
        assert_eq!(jxs.value_at_index(11), Some(0));
        assert_eq!(jxs.value_at_index(12), Some(0));
        assert_eq!(jxs.value_at_index(13), Some(0));
        assert_eq!(jxs.value_at_index(14), Some(14));
        assert_eq!(jxs.value_at_index(15), Some(15));
        assert_eq!(jxs.value_at_index(16), Some(16));
        assert_eq!(jxs.value_at_index(17), Some(17));
        assert_eq!(jxs.value_at_index(18), Some(18));
        assert_eq!(jxs.value_at_index(19), Some(19));
        assert_eq!(jxs.value_at_index(20), Some(20));
        assert_eq!(jxs.value_at_index(21), Some(21));
        assert_eq!(jxs.value_at_index(22), Some(22));
        assert_eq!(jxs.value_at_index(23), Some(23));
        assert_eq!(jxs.value_at_index(24), Some(24));
        assert_eq!(jxs.value_at_index(25), Some(25));
        assert_eq!(jxs.value_at_index(26), Some(26));
        assert_eq!(jxs.value_at_index(27), Some(27));
        assert_eq!(jxs.value_at_index(30), Some(30));
        assert_eq!(jxs.value_at_index(31), Some(31));
        assert_eq!(jxs.value_at_index(32), Some(32));
        assert_eq!(jxs.value_at_index(33), None);
    }
}
