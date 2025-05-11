use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use strum::IntoEnumIterator;
use anyhow::Result;

use crate::blocks::BlockType;
use crate::utils::PaceMmap;

//=====================================================================
// Represents the complete JXS array from an ACE file. This array
// contains the starting indices of all data blocks in the XXS array.
// If a block is not present, the starting index is reported as 0.
//=====================================================================
#[derive(Clone, Debug, Default)]
pub struct JxsArray {
    pub block_starting_indices: HashMap<BlockType, usize>
}

impl Deref for JxsArray {
    type Target = HashMap<BlockType, usize>;

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
    pub fn get(&self, key: &BlockType) -> usize {
        *self.block_starting_indices.get(key).unwrap()
    }

    pub fn insert(&mut self, key: BlockType, value: usize) {
        self.block_starting_indices.insert(key, value);
    }
}

impl JxsArray {
    pub fn from_PACE(mmap: &PaceMmap) -> Result<Self> {
        let mut jxs_array = JxsArray::default();
        let jxs_array_entries = mmap.jxs_array();

        // Fill in our array by looping over all BlockTypes
        for block_type in BlockType::iter() {
            // Get the index at which we should store the value
            let jxs_index = JxsArray::index_from_data_block_type(&block_type);
            jxs_array.insert(block_type, jxs_array_entries[jxs_index]);
        }

        Ok(jxs_array)
    }

    // For a given BlockType, return the index in the JXS array which lists
    // its starting index in the main XXS array.
    #[inline]
    fn index_from_data_block_type(block_type: &BlockType) -> usize {
        match block_type {
            BlockType::ESZ =>  0,
            BlockType::NU =>  1,
            BlockType::MTR =>  2,
            BlockType::LQR =>  3,
            BlockType::TYR =>  4,
            BlockType::LSIG =>  5,
            BlockType::SIG =>  6,
            BlockType::LAND =>  7,
            BlockType::AND =>  8,
            BlockType::LDLW =>  9,
            BlockType::DLW => 10,
            BlockType::GPD => 11,
            BlockType::MTRP => 12,
            BlockType::LSIGP => 13,
            BlockType::SIGP => 14,
            BlockType::LANDP => 15,
            BlockType::ANDP => 16,
            BlockType::LDLWP => 17,
            BlockType::DLWP => 18,
            BlockType::YP => 19,
            BlockType::FIS => 20,
            BlockType::END => 21,
            BlockType::LUND => 22,
            BlockType::DNU => 23,
            BlockType::BDD => 24,
            BlockType::DNEDL => 25,
            BlockType::DNED => 26,
            BlockType::PTYPE => 29,
            BlockType::NTRO => 30,
            BlockType::NEXT => 31,
        }
    }
}
