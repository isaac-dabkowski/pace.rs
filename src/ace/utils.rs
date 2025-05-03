#![allow(clippy::await_holding_lock, dead_code)]

use std::sync::Mutex;
use std::io::{self, Read, Write, Seek, BufReader, BufRead};
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::time::Instant;
use tempfile::tempfile;
use lazy_static::lazy_static;

use crate::ace::ace_data::AceIsotopeData;
use crate::ace::binary_format::convert_ascii_to_binary;

// These variables are used to hold filepaths in a way where
// they are accesible to all tests in all files, and where
// they can be parsed once and reused in all tests.
lazy_static! {
    // For custom ACE file available to all tests
    pub static ref TEST_ACE_ISOTOPE_DATA: Mutex<Option<AceIsotopeData>> = Mutex::new(None);
    pub static ref TEST_ACE_ASCII_COMMENTED: &'static str = "test_nuclear_data_files/test_ascii_ace";
    pub static ref TEST_ACE_ASCII_UNCOMMENTED: &'static str = "test_nuclear_data_files/test_ascii_ace.no_comment";
    pub static ref TEST_ACE_BINARY: &'static str = "test_nuclear_data_files/binary_1100.800nc";

    // For local testing
    pub static ref LOCAL_TEST_ACE_ISOTOPE_DATA: Mutex<Option<AceIsotopeData>> = Mutex::new(None);
    pub static ref LOCAL_TEST_ACE_ASCII: &'static str = "test_files/uranium_test_file";
    pub static ref LOCAL_TEST_BINARY_FILENAME: &'static str = "test_files/binary_92235.800nc";
    // pub static ref LOCAL_TEST_ACE_ASCII: &'static str = "test_files/hydrogen_test_file";
    // pub static ref LOCAL_TEST_BINARY_FILENAME: &'static str = "test_files/binary_1001.800nc";
}

// Checks if a file is ASCII by reading the first 1 kB of the file
pub fn is_ascii_file<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; 1024];

    match reader.read(&mut buffer)? {
        0 => Ok(true),
        n => Ok(!buffer[..n].iter().any(|&byte| 
            byte >= 128 || (byte < 32 && !matches!(byte, 9 | 10 | 13))
        ))
    }
}

// This function simply removes comments from the specially-constructed ASCII ACE test file
pub fn uncomment_ace_test_file() {
    let commented_filename: &Path = Path::new(*TEST_ACE_ASCII_COMMENTED);
    let uncommented_filename: &Path = Path::new(*TEST_ACE_ASCII_UNCOMMENTED);
    // Open test ASCII ACE
    let commented_file = File::open(commented_filename).unwrap();
    let reader = BufReader::new(commented_file);
    let uncommented_file = File::create(uncommented_filename).unwrap();
    let uncommented_file = Mutex::new(uncommented_file);

    // Rewrite the file without any of the comment lines
    for _line in reader.lines() {
        let line = _line.unwrap() + &String::from("\n");
        if !line.starts_with("//") {
            let mut uncommented_file = uncommented_file.lock().unwrap();
            if let Err(e) = uncommented_file.write_all(line.into_bytes().as_slice()) {
                eprintln!("Error writing to file: {}", e);
            }
            if let Err(e) = uncommented_file.flush() {
                eprintln!("Error flushing file: {}", e);
            }
        }
    }
}

// This function simply removes comments from the specially-constructed ASCII ACE test file
// It also parses the test file to binary
pub fn convert_ace_test_file_to_binary() {
    uncomment_ace_test_file();
    let uncommented_filename: &Path = Path::new(*TEST_ACE_ASCII_UNCOMMENTED);
    // Process to a binary file as well
    convert_ascii_to_binary(uncommented_filename).expect("Error processing ASCII ACE file to binary");
}

// Read a specified number of lines into a BufReader
#[inline]
pub fn read_lines(reader: &mut BufReader<File>, num_lines: usize) -> Result<Vec<String>, Box<dyn Error>> {
    reader.lines()
        .take(num_lines)
        .collect::<Result<Vec<_>, _>>()
        .map_err(
            |e| Box::new(e) as Box<dyn Error>
        )
}

// Provided a temperature in MeV, convert to K
#[inline]
pub fn compute_temperature_from_kT(kT: f64) -> f64 {
    kT * 1e6 / 8.617333262e-5
}

// Create a reader from a string to aid in testing
#[inline]
pub fn create_reader_from_string(content: &str) -> BufReader<File> {
    let mut test_file = tempfile().unwrap();
    writeln!(&mut test_file, "{}", content).unwrap();
    test_file.seek(std::io::SeekFrom::Start(0)).unwrap();
    BufReader::new(test_file)
}

// The following code parses an example ACE file and saves it the resulting
// `AceIsotopeData` is globally accesible for testing.
pub async fn local_get_parsed_test_file() -> AceIsotopeData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<AceIsotopeData>> = LOCAL_TEST_ACE_ISOTOPE_DATA.lock().unwrap();

    // Only parse the ACE file if it is not already parsed
    if data.is_none() {
        // Convert the ACE file to binary
        let mut start = Instant::now();
        let _ = convert_ascii_to_binary(*LOCAL_TEST_ACE_ASCII);
        println!(
            "⚛️  Time to convert local ACE file from ASCII to binary ⚛️ : {} sec",
            start.elapsed().as_secs_f32()
        );

        // Parse the ACE file
        start = Instant::now();
        let parsed_ace = AceIsotopeData::from_file(*LOCAL_TEST_BINARY_FILENAME).await.unwrap();
        println!(
            "⚛️  Time to parse local ACE file ⚛️ : {} sec",
            start.elapsed().as_secs_f32()
        );
        *data = Some(parsed_ace);
    }
    data.as_ref().unwrap().clone()
}

pub async fn get_parsed_test_file() -> AceIsotopeData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<AceIsotopeData>> = TEST_ACE_ISOTOPE_DATA.lock().unwrap();

    // Only parse the ACE file if it is not already parsed
    if data.is_none() {
        // Make sure that our special ACE test file binary is always up to date
        let mut start = Instant::now();
        convert_ace_test_file_to_binary();
        println!(
            "⚛️  Time to convert custom ACE test file from ASCII to binary ⚛️ : {} sec",
            start.elapsed().as_secs_f32()
        );

        start = Instant::now();
        let parsed_ace = AceIsotopeData::from_file(*TEST_ACE_BINARY).await.unwrap();
        println!(
            "⚛️  Time to parse custom ACE test file ⚛️ : {} sec",
            start.elapsed().as_secs_f32()
        );
        *data = Some(parsed_ace);
    }
    data.as_ref().unwrap().clone()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ascii_file() {
        uncomment_ace_test_file();
        assert!(is_ascii_file(*TEST_ACE_ASCII_UNCOMMENTED).unwrap());
    }

    #[test]
    fn test_compute_temperature_from_kT() {
        let kT = 8.617333262e-8;
        let expected_temperature = 1000.0; // Kelvin
        assert!((compute_temperature_from_kT(kT) - expected_temperature).abs() < 1e-9);
    }
}