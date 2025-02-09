use std::path::Path;
use std::error::Error;

use crate::ace::utils::is_ascii_file;
use crate::ace::binary_format::AceBinaryMmap;
use crate::ace::header::AceHeader;
use crate::ace::arrays::{IzawArray, JxsArray, NxsArray};
use crate::ace::blocks::DataBlocks;
use crate::helpers;

#[derive(Clone)]
pub struct AceIsotopeData {
    pub header: AceHeader,
    pub izaw_array: IzawArray,
    pub nxs_array: NxsArray,
    pub jxs_array: JxsArray,
    pub data_blocks: DataBlocks
}

impl AceIsotopeData {
    pub async fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        let path = file_path.as_ref();

        // If we have an ASCII file, request that it first be parsed to our own binary format
        // using crate::ace::binary_format::convert_ascii_to_binary
        if is_ascii_file(path)? {
            return Err(
                format!(
                    "File {} is ASCII, this should first be converted to binary format with \
                    crate::ace::binary_format::convert_ascii_to_binary", path.display()).into()
            )
        }

        // We have a binary file, so we can proceed with parsing it
        // Create a memory map of the binary file
        let mmap = AceBinaryMmap::from_file(path)?;

        // Process the header
        let header = AceHeader::from_file(&mmap)?;

        // Process the IZAW array
        let izaw_array = IzawArray::from_file(&mmap)?;

        // Process the NXS array
        let nxs_array = NxsArray::from_file(&mmap)?;

        // Process the JXS array
        let jxs_array = JxsArray::from_file(&mmap)?;

        // Process the blocks out of the XXS array
        let data_blocks = DataBlocks::from_file(&mmap, &nxs_array, &jxs_array)?;

        Ok(Self { header, izaw_array, nxs_array, jxs_array, data_blocks})
    }

    // ZAID of the isotope
    #[inline]
    pub fn zaid(&self) -> String {
        self.header.zaid.clone()
    }

    // SZAID of the isotope (version 2.0.0 and later)
    #[inline]
    pub fn szaid(&self) -> Option<String> {
        self.header.szaid.clone()
    }

    // Atomic mass fraction
    #[inline]
    pub fn atomic_mass_fraction(&self) -> f64 {
        self.header.atomic_mass_fraction
    }

    // kT
    #[inline]
    pub fn kT(&self) -> f64 {
        self.header.kT
    }

    // Temperature in Kelvin
    #[inline]
    pub fn temperature(&self) -> f64 {
        self.header.temperature
    }

    // ZA of the isotope
    #[inline]
    pub fn za(&self) -> usize {
        self.nxs_array.za
    }

    // Atomic number
    #[inline]
    pub fn z(&self) -> usize {
        self.nxs_array.z
    }

    // Mass number
    #[inline]
    pub fn a(&self) -> usize {
        self.nxs_array.a
    }

    // Isotope name
    #[inline]
    pub fn name(&self) -> String {
        helpers::isotope_name_from_Z_A(self.z(), self.a())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ace::utils::{get_parsed_test_file, local_get_parsed_test_file};

    #[tokio::test]
    async fn test_parse_test_file() {
        get_parsed_test_file().await;
    }

    // This test should only be run locally on a real ACE file
    #[tokio::test]
    async fn test_parse_local_test_file() {
        local_get_parsed_test_file().await;
    }

    #[tokio::test]
    async fn test_reject_ascii() {
        // We can just test this on the License file
        let result = AceIsotopeData::from_file("LICENSE").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_szaid_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.szaid(), Some(String::from("1100.800nc")));
    }

    #[tokio::test]
    async fn test_zaid_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.zaid(), String::from("1100.00c"));
    }

    #[tokio::test]
    async fn test_atomic_mass_fraction_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.atomic_mass_fraction(), 99.999);
    }

    #[tokio::test]
    async fn test_kT_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.kT(), 2.5301e-08);
    }

    #[tokio::test]
    async fn test_temperature_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.temperature(), 293.6059129982851);
    }

    #[tokio::test]
    async fn test_za_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.za(), 1100);
    }

    #[tokio::test]
    async fn test_z_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.z(), 1);
    }

    #[tokio::test]
    async fn test_a_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.a(), 100);
    }

    #[tokio::test]
    async fn test_name_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.name(), "H-100");
    }
}