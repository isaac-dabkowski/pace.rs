use std::time::Instant;

use crate::ace::arrays::Arrays;
use crate::ace::blocks::DataBlockType;

pub fn get_block_start(block_type: &DataBlockType, arrays: &Arrays, is_expected: bool, panic_message: String) -> Option<usize> {
    // If the block type's start index is non-zero, the block is present in the XXS array
    let start_index = arrays.jxs.get(block_type);
    // The block is expected
    if is_expected {
        // Panic if the block is not present
        if start_index == 0 {
            panic!("{}", panic_message);
        } else {
            // The block is present, return the start index
            // Note that the XXS array in our binary format is zero indexed (which does not match the ACE spec)
           return Some(start_index - 1);
        }
    // The block is not expected
    } else {
        // If the block is present, panic
        if start_index != 0 {
            panic!("{}: Block was found when it was not expected.", block_type);
        } else {
            // The block is not present, return None
            return None;
        }
    }
}

pub fn block_range_to_slice<'a>(block_start: usize, block_length: usize, arrays: &'a Arrays) -> &'a [f64] {
    let mut block_end = block_start + block_length;
    if block_end == arrays.xxs.len() + 1 {
        block_end -= 1;
    }
    &arrays.xxs[block_start..block_end]
}

pub trait PullFromXXS<'a> {
    fn pull_from_xxs_array(is_expected: bool, arrays: &'a Arrays) -> Option<&'a [f64]>
    where
        Self: Sized;
}

pub trait Process<'a> {
    type Dependencies;

    fn process(data: &'a [f64], arrays: &Arrays, dependencies: Self::Dependencies) -> Self
    where
        Self: Sized;
}

pub trait Parse<'a>: PullFromXXS<'a> + Process<'a> {
    fn parse(is_expected: bool, arrays: &'a Arrays, dependencies: Self::Dependencies) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(data) = Self::pull_from_xxs_array(is_expected, arrays) {
            Some(Self::process(data, arrays, dependencies))
        } else {
            None
        }
    }
}

impl<'a, T> Parse<'a> for T
where
    T: Process<'a> + PullFromXXS<'a>,
{}
