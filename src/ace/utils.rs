#![allow(clippy::await_holding_lock, dead_code)]

use std::sync::Mutex;
use std::io::{self, Read, Write, Seek, BufReader, BufRead};
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::time::Instant;
use tempfile::tempfile;
use lazy_static::lazy_static;

use crate::AceIsotopeData;
use crate::ace::binary_format::convert_ascii_to_binary;

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
// It also parses the test file to binary
pub fn update_ace_test_files() {
    let commented_filename: &Path = Path::new(*TEST_ASCII_FILE_COMMENTED);
    let uncommented_filename: &Path = Path::new(*TEST_ASCII_FILE_UNCOMMENTED);
    // Open test ASCII ACE
    let commented_file = File::open(commented_filename).unwrap();
    let mut uncommented_file = File::create(uncommented_filename).unwrap();
    let reader = BufReader::new(commented_file);

    // Rewrite the file without any of the comment lines
    for _line in reader.lines() {
        let line = _line.unwrap() + &String::from("\n");
        if !line.starts_with("//") {
            if let Err(e) = uncommented_file.write_all(line.into_bytes().as_slice()) {
                eprintln!("Error writing to file: {}", e);
            }
            if let Err(e) = uncommented_file.flush() {
                eprintln!("Error flushing file: {}", e);
            }
        }
    }

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
lazy_static! {
    // Holds parsed test ACE file
    pub static ref TEST_ACEISOTOPEDATA: Mutex<Option<AceIsotopeData>> = Mutex::new(None);
    pub static ref TEST_ASCII_FILE_COMMENTED: &'static str = "test_nuclear_data_files/test_ascii_ace";
    pub static ref TEST_ASCII_FILE_UNCOMMENTED: &'static str = "test_nuclear_data_files/test_ascii_ace.no_comment";
    pub static ref TEST_BINARY_FILE: &'static str = "test_nuclear_data_files/binary_1100.800nc";

    // For local testing
    pub static ref LOCAL_TEST_ACEISOTOPEDATA: Mutex<Option<AceIsotopeData>> = Mutex::new(None);
    // pub static ref LOCAL_TEST_BINARY_FILENAME: &'static str = "test_files/binary_1001.800nc";
    pub static ref LOCAL_TEST_BINARY_FILENAME: &'static str = "test_files/binary_92235.800nc";
}

pub async fn local_get_parsed_test_file() -> AceIsotopeData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<AceIsotopeData>> = LOCAL_TEST_ACEISOTOPEDATA.lock().unwrap();

    if data.is_none() {
        // Make sure that our special ACE test file binary is always up to date 
        update_ace_test_files();

        let start = std::time::SystemTime::now();
        let parsed_ace = AceIsotopeData::from_file(*LOCAL_TEST_BINARY_FILENAME).await.unwrap();
        println!(
            "⚛️  Time to parse local ACE file ⚛️ : {} sec",
            std::time::SystemTime::now().duration_since(start).unwrap().as_secs_f32()
        );
        *data = Some(parsed_ace);
    }
    data.as_ref().unwrap().clone()
}

pub async fn get_parsed_test_file() -> AceIsotopeData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<AceIsotopeData>> = TEST_ACEISOTOPEDATA.lock().unwrap();

    if data.is_none() {
        // Make sure that our special ACE test file binary is always up to date 
        update_ace_test_files();

        let start = Instant::now();
        let parsed_ace = AceIsotopeData::from_file(*TEST_BINARY_FILE).await.unwrap();
        println!(
            "⚛️  Time to parse example ACE file ⚛️ : {} sec",
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
        update_ace_test_files();
        assert!(is_ascii_file(*TEST_ASCII_FILE_UNCOMMENTED).unwrap());
    }

    #[test]
    fn test_compute_temperature_from_kT() {
        let kT = 8.617333262e-8;
        let expected_temperature = 1000.0; // Kelvin
        assert!((compute_temperature_from_kT(kT) - expected_temperature).abs() < 1e-9);
    }
}