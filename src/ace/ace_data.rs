use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use crate::ace::utils::is_ascii_file;
use crate::ace::header::AceHeader;
use crate::ace::arrays::{IzawPair, IzawArray, JxsArray, NxsArray};
use crate::ace::blocks::DataBlocks;

#[derive(Clone)]
pub struct AceIsotopeData {
    header: AceHeader,
    izaw_array: IzawArray,
    nxs_array: NxsArray,
    jxs_array: JxsArray,
    data_blocks: DataBlocks
}

impl AceIsotopeData {
    pub async fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        let path = file_path.as_ref();

        // Invoke ASCII or binary parsing based on file type
        if is_ascii_file(path)? {
            // Parse ASCII file
            let ace_data = AceIsotopeData::from_ascii_file(path).await?;
            Ok(ace_data)
        } else {
            // Parse binary file
            todo!()
        }
    }

    // Create an AceIsotopeData object from an ASCII file
    pub async fn from_ascii_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path).map_err(|e| format!("Error opening ACE ASCII file: {}", e))?;
        let mut reader = BufReader::new(file);

        // Process the header
        let time = std::time::SystemTime::now();
        let header = AceHeader::from_ascii_file(&mut reader)?;
        println!(
            "⚛️  Time to parse header ⚛️ : {} μs",
            std::time::SystemTime::now().duration_since(time).unwrap().as_micros()
        );
        let time = std::time::SystemTime::now();

        // Process the IZAW array
        let izaw_array = IzawArray::from_ascii_file(&mut reader)?;
        println!(
            "⚛️  Time to parse IZAW ⚛️ : {} μs",
            std::time::SystemTime::now().duration_since(time).unwrap().as_micros()
        );
        let time = std::time::SystemTime::now();

        // Process the NXS array
        let nxs_array = NxsArray::from_ascii_file(&mut reader)?;
        println!(
            "⚛️  Time to parse NXS ⚛️ : {} μs",
            std::time::SystemTime::now().duration_since(time).unwrap().as_micros()
        );
        let time = std::time::SystemTime::now();

        // Process the JXS array
        let jxs_array = JxsArray::from_ascii_file(&mut reader)?;
        println!(
            "⚛️  Time to parse JXS ⚛️ : {} μs",
            std::time::SystemTime::now().duration_since(time).unwrap().as_micros()
        );
        let time = std::time::SystemTime::now();

        // Process the blocks out of the XXS array
        let data_blocks = DataBlocks::from_ascii_file(&mut reader, &nxs_array, &jxs_array).await?;
        println!(
            "⚛️  Time to parse data blocks ⚛️ : {} sec",
            std::time::SystemTime::now().duration_since(time).unwrap().as_secs_f32()
        );
        let time = std::time::SystemTime::now();

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

    // S alpha beta pairs of ZAIDs and atomic weight ratios
    #[inline]
    pub fn s_a_b_pairs(&self) -> Vec<IzawPair> {
        self.izaw_array.pairs.clone()
    }

    // Number of entries in the main data array
    #[inline]
    pub fn num_entries(&self) -> usize {
        self.nxs_array.xxs_len
    }

    // Number of energies
    #[inline]
    pub fn num_energies(&self) -> usize {
        self.nxs_array.nes
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
}

#[cfg(test)]
mod ascii_tests {
    use crate::ace::utils::get_parsed_ascii_for_testing;

    #[tokio::test]
    async fn test_parse_file() {
        get_parsed_ascii_for_testing().await;
    }

    #[tokio::test]
    async fn test_szaid_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.szaid(), Some(String::from("1001.800nc")));
    }

    #[tokio::test]
    async fn test_zaid_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.zaid(), String::from("1001.00c"));
    }

    #[tokio::test]
    async fn test_atomic_mass_fraction_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.atomic_mass_fraction(), 0.999167);
    }

    #[tokio::test]
    async fn test_kT_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.kT(), 2.5301e-08);
    }

    #[tokio::test]
    async fn test_temperature_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.temperature(), 293.6059129982851);
    }

    #[tokio::test]
    async fn test_izaw_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        for za_iz_pair in parsed_ace.s_a_b_pairs() {
            assert_eq!(za_iz_pair.za, 0);
            assert_eq!(za_iz_pair.iz, 0.0);
        }
        assert_eq!(parsed_ace.s_a_b_pairs().len(), 16)
    }

    #[tokio::test]
    async fn test_num_entries_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.num_entries(), 10257);
    }

    #[tokio::test]
    async fn test_num_energies_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.num_energies(), 631);
    }

    #[tokio::test]
    async fn test_za_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.za(), 1001);
    }

    #[tokio::test]
    async fn test_z_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.z(), 1);
    }

    #[tokio::test]
    async fn test_a_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing().await;
        assert_eq!(parsed_ace.a(), 1);
    }
}

#[cfg(test)]
mod binary_tests {
}