use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::error::Error;

use strum::IntoEnumIterator;

use crate::ace::blocks::DataBlockType;
use crate::ace::binary_format::AceBinaryMmap;

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
    pub fn from_file(mmap: &AceBinaryMmap) -> Result<Self, Box<dyn Error>> {
        let mut jxs_array = JxsArray::default();
        let jxs_array_entries = mmap.jxs_array();

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
    #[inline]
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
    fn test_value_at_index() {
        let mut jxs = JxsArray::default();

        jxs.insert(DataBlockType::ESZ, 1);
        jxs.insert(DataBlockType::NU, 0);
        jxs.insert(DataBlockType::SIG, 20);

        // Check values at specific indices
        assert_eq!(jxs.value_at_index(1), Some(1));
        assert_eq!(jxs.value_at_index(2), Some(0));
        assert_eq!(jxs.value_at_index(7), Some(20));
        assert_eq!(jxs.value_at_index(33), None);
    }
}
