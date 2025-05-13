use std::path::Path;

use anyhow::Result;

use crate::utils::{is_ascii_file, PaceMmap};
use crate::header::Header;
use crate::arrays::{IzawArray, JxsArray, NxsArray};
use crate::blocks::DataBlocks;
use crate::helpers;

#[derive(Clone)]
pub struct PaceData {
    pub header: Header,
    pub izaw_array: IzawArray,
    pub nxs_array: NxsArray,
    pub jxs_array: JxsArray,
    pub data_blocks: DataBlocks
}

impl PaceData {
    pub async fn from_PACE<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let path = file_path.as_ref();

        // If we have an ASCII file, request that it first be parsed to our own binary format
        // using crate::ace::binary_format::convert_ascii_to_binary
        if is_ascii_file(path)? {
            return Err(
                anyhow::anyhow!(
                    "File {} is ASCII, this should first be converted to binary format with \
                    crate::ace::binary_format::convert_ascii_to_binary", path.display())
            )
        }

        // We have a binary file, so we can proceed with parsing it
        // Create a memory map of the binary file
        let mmap = PaceMmap::from_PACE(path)?;

        // Process the header
        let header = Header::from_PACE(&mmap)?;

        // Process the IZAW array
        let izaw_array = IzawArray::from_PACE(&mmap)?;

        // Process the NXS array
        let nxs_array = NxsArray::from_PACE(&mmap)?;

        // Process the JXS array
        let jxs_array = JxsArray::from_PACE(&mmap)?;

        // Process the blocks out of the XXS array
        let data_blocks = DataBlocks::from_PACE(&mmap, &nxs_array, &jxs_array)?;

        Ok(Self { header, izaw_array, nxs_array, jxs_array, data_blocks})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_parse_test_file() {
        get_parsed_test_file().await;
    }

    // This test should only be run locally on a real ACE file
    // turn this on with `cargo test --features local`
    #[cfg(feature = "local")]
    #[tokio::test]
    async fn test_parse_local_test_file() {
        use crate::utils::local_get_parsed_test_file;
        local_get_parsed_test_file().await;
    }

    #[tokio::test]
    async fn test_reject_ascii() {
        // We can just test this on the License file
        let result = PaceData::from_PACE("LICENSE").await;
        assert!(result.is_err());
    }
}