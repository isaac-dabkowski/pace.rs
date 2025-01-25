use std::convert::From;
use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use std::ops::Deref;
use std::io::{BufReader, Read, Seek, SeekFrom};

use crate::ace::data_blocks;
use crate::ace::arrays::{JxsEntry, JxsArray, NxsArray};

type BlockTextMap = HashMap<DataBlockType, Option<Vec<String>>>;

// Enum of all block types in continuous neutron ACE file
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataBlockType {
    ESZ,    // Energy table
    NU,     // Fission nu data
    MTR,    // MT array
    LQR,    // Q-value array
    TYR,    // Reaction type array
    LSIG,   // Table of cross section locators
    SIG,    // Cross sections
    LAND,   // Table of angular distribution locators
    AND,    // Angular distributions
    LDLW,   // Table of energy distribution locators
    DLW,    // Energy distributions
    GPD,    // Photon production data
    MTRP,   // Photon production MT array
    LSIGP,  // Table of photon production cross section locators
    SIGP,   // Photon production cross sections
    LANDP,  // Table of photon production angular distribution locators
    ANDP,   // Photon production angular distributions
    LDLWP,  // Table of photon production energy distribution locators
    DLWP,   // Photon production energy distributions
    YP,     // Table of yield multipliers
    FIS,    // Total fission cross section
    END,    // Last word of the conventional table
    LUND,   // Probability tables
    DNU,    // Delayed nu-bar data
    BDD,    // Basic delayed neutron precursor data
    DNEDL,  // Table of delayed neutron energy distribution locators
    DNED,   // Delayed neutron energy distributions
    PTYPE,  // Particle type array
    NTRO,   // Array containing number of particle production reactions
    NEXT,   // Table of particle production locators
}

impl std::fmt::Display for DataBlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Holds the start and end location of a block in the XXS array
#[derive(Clone, Debug, PartialEq)]
struct BlockBounds {
    start: usize,
    end: usize,
}

impl From<&JxsEntry> for BlockBounds {
    fn from(jxs_entry: &JxsEntry) -> Self {
        BlockBounds {
            start: jxs_entry.loc,
            end: jxs_entry.loc + jxs_entry.len
        }
    }
}


#[derive(Clone, Debug, PartialEq, Default)]
pub struct DataBlocks {
    pub ESZ: Option<data_blocks::ESZ>
}

impl DataBlocks {
    // Create a new BlockProcessor from a XXS array, the NXS and JXS array are used to
    // determine the start and end locations of each block
    pub fn from_ascii_file(reader: &mut BufReader<File>, nxs_array: &NxsArray, jxs_array: &JxsArray) -> Result<Self, Box<dyn Error>> {
        // Split XXS array into raw text correspoding to each block 
        let block_map = DataBlocks::split_xxs_into_blocks(reader, jxs_array)?;

        // TODO: Use the AsyncTaskDag to set up our block workflows here
        // Builder struct assists in initialization
        let mut builder = DataBlocksBuilder::new();
        for (block_type, block_text) in block_map {
            if let Some(text) = block_text {
                builder.process_block(block_type, text, nxs_array);
            }
        }
        let data_blocks = builder.build();
        Ok(data_blocks)
    }

    // Splits the XXS array into blocks of text corresponding to each block type
    fn split_xxs_into_blocks(reader: &mut BufReader<File>, jxs_array: &JxsArray) -> Result<BlockTextMap, Box<dyn Error>> {
        let mut block_map = HashMap::new();
        // Loop over the JXS array and create a BlockBounds for each block which exists
        for (block_type, jxs_entry) in jxs_array.iter().filter_map(|(key, value)| {
            value.as_ref().map(|v| (key, v))
        }) {
            let block_bounds = BlockBounds::from(jxs_entry);
            let block_text = DataBlocks::get_block_from_xxs(reader, block_bounds)
                .unwrap_or_else(|_| panic!("Error processing block: {:?}", block_type));
            block_map.insert(block_type.clone(), Some(block_text));
        }
        Ok(block_map)
    }

    // Get the text of a block from the XXS array
    fn get_block_from_xxs(reader: &mut BufReader<File>, block_bounds: BlockBounds) -> Result<Vec<String>, Box<dyn Error>> {
        // Determine the start and end lines of the block
        let start_line = block_bounds.start / 4 + usize::from(block_bounds.start % 4 != 0);
        let end_line = block_bounds.end / 4 + usize::from(block_bounds.end % 4 != 0);
        // A block may start or end mid-line, so we need to determine
        // the start index within the first line
        let start_offset = (block_bounds.start - 1) % 4;
        let block_length = block_bounds.end - block_bounds.start;
        // Get the lines from the XXS array
        let block_lines = DataBlocks::get_lines_from_xxs(reader, start_line, end_line)?;
        // Split on whitespace
        let block_text: Vec<String> = block_lines.iter()
            .flat_map(|s| s.split_whitespace())
            .skip(start_offset)
            .take(block_length)
            .map(|s| s.to_string())
            .collect();
        Ok(block_text)
    }

    // Support quick access of a specified range of lines within the XXS array - range is [start, end] w/ 1-based indexing
    fn get_lines_from_xxs(reader: &mut BufReader<File>, start: usize, end: usize) -> Result<Vec<String>, Box<dyn Error>> {
        let xxs_start_location = reader.stream_position()?;
        // Lines are up to 81 ASCII characters long with newline character
        let line_length = 81;
        let mut lines = Vec::new();
        for line_number in start..=end {
            // Calculate the offset of the line
            let offset = xxs_start_location + (line_length * (line_number - 1)) as u64;
            reader.seek(SeekFrom::Start(offset))?;
            // Read line up to newline character
            let mut buffer = vec![0; line_length - 1];
            let bytes_read = reader.read(&mut buffer)?;
            buffer.truncate(bytes_read);
            let line = String::from_utf8_lossy(&buffer).to_string();
            lines.push(line);
        }
        // Move reader back to the start of the XXS array
        reader.seek(SeekFrom::Start(xxs_start_location))?;
        Ok(lines)
    }
}

pub trait BlockConstruction {
    fn construct(text_data: Vec<String>, nxs_array: &NxsArray) -> Self;
}

struct DataBlocksBuilder {
    inner: DataBlocks
}

impl DataBlocksBuilder {
    fn new() -> Self {
        Self {
            inner: DataBlocks::default()
        }
    }

    fn process_block(&mut self, block_type: DataBlockType, block_text: Vec<String>, nxs_array: &NxsArray) {
        match block_type {
            DataBlockType::ESZ => self.inner.ESZ = Some(data_blocks::ESZ::construct(block_text, nxs_array)),
            // DataBlockType::NU => self.inner.NU = Some(NU::construct(block_text)),
            // DataBlockType::MTR => self.inner.MTR = Some(MTR::construct(block_text)),
            // DataBlockType::LQR => self.inner.LQR = Some(LQR::construct(block_text)),
            // DataBlockType::TYR => self.inner.TYR = Some(TYR::construct(block_text)),
            // DataBlockType::LSIG => self.inner.LSIG = Some(LSIG::construct(block_text)),
            // DataBlockType::SIG => self.inner.SIG = Some(SIG::construct(block_text)),
            // DataBlockType::LAND => self.inner.LAND = Some(LAND::construct(block_text)),
            // DataBlockType::AND => self.inner.AND = Some(AND::construct(block_text)),
            // DataBlockType::LDLW => self.inner.LDLW = Some(LDLW::construct(block_text)),
            // DataBlockType::DLW => self.inner.DLW = Some(DLW::construct(block_text)),
            // DataBlockType::GPD => self.inner.GPD = Some(GPD::construct(block_text)),
            // DataBlockType::MTRP => self.inner.MTRP = Some(MTRP::construct(block_text)),
            // DataBlockType::LSIGP => self.inner.LSIGP = Some(LSIGP::construct(block_text)),
            // DataBlockType::SIGP => self.inner.SIGP = Some(SIGP::construct(block_text)),
            // DataBlockType::LANDP => self.inner.LANDP = Some(LANDP::construct(block_text)),
            // DataBlockType::ANDP => self.inner.ANDP = Some(ANDP::construct(block_text)),
            // DataBlockType::LDLWP => self.inner.LDLWP = Some(LDLWP::construct(block_text)),
            // DataBlockType::DLWP => self.inner.DLWP = Some(DLWP::construct(block_text)),
            // DataBlockType::YP => self.inner.YP = Some(YP::construct(block_text)),
            // DataBlockType::FIS => self.inner.FIS = Some(FIS::construct(block_text)),
            // DataBlockType::END => self.inner.END = Some(END::construct(block_text)),
            // DataBlockType::LUND => self.inner.LUND = Some(LUND::construct(block_text)),
            // DataBlockType::DNU => self.inner.DNU = Some(DNU::construct(block_text)),
            // DataBlockType::BDD => self.inner.BDD = Some(BDD::construct(block_text)),
            // DataBlockType::DNEDL => self.inner.DNEDL = Some(DNEDL::construct(block_text)),
            // DataBlockType::DNED => self.inner.DNED = Some(DNED::construct(block_text)),
            // DataBlockType::PTYPE => self.inner.PTYPE = Some(PTYPE::construct(block_text)),
            // DataBlockType::NTRO => self.inner.NTRO = Some(NTRO::construct(block_text)),
            // DataBlockType::NEXT => self.inner.NEXT = Some(NEXT::construct(block_text)),
            _ => log::debug!("Field {:?} not yet implemented", block_type)
        };
    }
    
    fn build(self) -> DataBlocks {
        self.inner
    }
}

#[cfg(test)]
mod ascii_tests {
    use super::*;
    use crate::ace::utils;

    fn create_test_jxs_array() -> JxsArray {
        let mut jxs_array = JxsArray::default();
        jxs_array.insert(DataBlockType::ESZ, Some(JxsEntry { loc: 1, len: 4 }));
        jxs_array.insert(DataBlockType::NU, Some(JxsEntry { loc: 5, len: 4 }));
        jxs_array
    }

    fn create_example_xxs() -> BufReader<File> {
        utils::create_reader_from_string(
            concat!(
                "   1.00000000000E+00   2.00000000000E+00   3.00000000000E+00   4.00000000000E+00\n",
                "   5.00000000000E+00   6.00000000000E+00   7.00000000000E+00   8.00000000000E+00\n",
                "   9.00000000000E+00   1.00000000000E+01   1.10000000000E+01   1.20000000000E+01\n",
                "   1.30000000000E+01   1.40000000000E+01   1.50000000000E+01   1.60000000000E+01\n"
            )
        )
    }

    #[test]
    fn test_block_bounds_from_jxs_entry() {
        let jxs_entry = JxsEntry { loc: 1, len: 4 };
        let block_bounds: BlockBounds = BlockBounds::from(&jxs_entry);
        assert_eq!(block_bounds, BlockBounds { start: 1, end: 5 });
    }

    #[test]
    fn test_blocks_from_ascii_file() {
        let jxs_array = create_test_jxs_array();
        let mut reader = create_example_xxs();
        let block_map = DataBlocks::split_xxs_into_blocks(&mut reader, &jxs_array).unwrap();
        assert!(block_map.contains_key(&DataBlockType::ESZ));
        assert!(block_map.contains_key(&DataBlockType::NU));
    }

    #[test]
    fn test_get_block_from_xxs() {
        let mut reader = create_example_xxs();
        let block_bounds = BlockBounds { start: 2, end: 4 };
        let block_text = DataBlocks::get_block_from_xxs(&mut reader, block_bounds).unwrap();
        assert_eq!(block_text, vec!["2.00000000000E+00".to_string(), "3.00000000000E+00".to_string()]);
    }

    #[test]
    fn test_get_lines_from_xxs() {
        let mut reader = create_example_xxs();
        let lines = DataBlocks::get_lines_from_xxs(&mut reader, 1, 2).unwrap();
        assert_eq!(
            lines,
            vec![
                "   1.00000000000E+00   2.00000000000E+00   3.00000000000E+00   4.00000000000E+00".to_string(),
                "   5.00000000000E+00   6.00000000000E+00   7.00000000000E+00   8.00000000000E+00".to_string()
            ]
        );
    }
}