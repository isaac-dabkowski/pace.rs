use crate::arrays::Arrays;
use crate::blocks::BlockType;

//=====================================================================
// Every block in the XXS array needs to implement the following traits:
// - PullFromXXS:
//     - Pull the data from the XXS array, this should implement
//       all of the logic needed to determine the end of the block.
//       we return a slice from the XXS array.
// - Process:
//     - Process the data from the XXS array. This is all of the logic
//       which converts the data from the slice produced by PullFromXXS
//       into the final data structure.
//
// If both of these traits are implemented, we automatically implement
// the Parse trait, which calls the two other traits in order to parse
// the data from the XXS array.
//=====================================================================

// Pull from the XXS array, return a slice of the XXS array if the block exists.
// If the block does not exist, return None.
pub trait PullFromXXS<'a> {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]>
    where
        Self: Sized;
}

// Process the slice from PullFromXXS into the final data structure.
// Because some blocks depend on values in other blocks, we have a flexible
// dependency system. The dependencies are passed in as a parameter to
// the process function.
pub trait Process<'a> {
    type Dependencies;

    fn process(data: &'a [f64], arrays: &Arrays, dependencies: Self::Dependencies) -> Self
    where
        Self: Sized;
}

// Pull a block from the XXS array and process it into the final data structure.
// This is the main function which is called to parse a block from the XXS array,
// and it is implemented for all blocks which implement the PullFromXXS and Process traits.
pub trait Parse<'a>: PullFromXXS<'a> + Process<'a> {
    fn parse(arrays: &'a Arrays, dependencies: Self::Dependencies) -> Option<Self>
    where
        Self: Sized,
    {
        if let Some(data) = Self::pull_from_xxs_array(arrays) {
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


//=====================================================================
// Helper functions to make working with the XXS array easier.
//=====================================================================

// Once the NXS and JXS arrays are loaded, we can use them to determine whether or not
// a block is expected to be present in the XXS array. This function takes in a boolean
// indicating whether or not the block is expected to be present, and we panic if the block
// is not present when it is expected, or if the block is present when it is not expected,
// both of which are indicative of a bug in the code or a serious error in the PACE file.
pub fn get_block_start(block_type: &BlockType, arrays: &Arrays, is_expected: bool, panic_message: String) -> Option<usize> {
    // If the block type's start index is non-zero, the block is present in the XXS array
    let start_index = arrays.jxs.get(block_type);
    // The block is expected
    if is_expected {
        // Panic if the block is not present - something has gone very wrong.
        if start_index == 0 {
            panic!("{}", panic_message);
        } else {
            // The block is present, return the start index
            // Note that the XXS array in the PACE binary format is zero
            // indexed (which does not match the ACE spec)
           return Some(start_index - 1);
        }
    // The block is not expected
    } else {
        // If the block is present, panic - something has gone very wrong.
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