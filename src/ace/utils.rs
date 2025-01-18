#![allow(non_snake_case)]

use std::sync::Mutex;
use std::io::{self, Read, Write, Seek, BufReader, BufRead};
use std::path::Path;
use std::fs::File;
use std::error::Error;
use tempfile::tempfile;
use lazy_static::lazy_static;
use crate::AceIsotopeData;

// Checks if a file is ASCII by reading the first 100 kB of the file
pub fn is_ascii_file<P: AsRef<Path>>(path: P) -> io::Result<bool> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; 102400];

    match reader.read(&mut buffer)? {
        0 => Ok(true),
        n => Ok(!buffer[..n].iter().any(|&byte| 
            byte >= 128 || (byte < 32 && !matches!(byte, 9 | 10 | 13))
        ))
    }
}

#[inline]
// Read a specified number of lines into a vector of strings
pub fn read_lines(reader: &mut BufReader<File>, num_lines: usize) -> Result<Vec<String>, Box<dyn Error>> {
    reader.lines()
        .take(num_lines)
        .collect::<Result<Vec<_>, _>>()
        .map_err(
            |e| Box::new(e) as Box<dyn Error>
        )
}

#[inline]
// Provided a temperature in MeV, convert to K
pub fn compute_temperature_from_kT(kT: f64) -> f64 {
    kT * 1e6 / 8.617333262e-5
}

// Create a reader from a string to aid in testing
#[allow(dead_code)]
pub fn create_reader_from_string(content: &str) -> BufReader<File> {
    let mut test_file = tempfile().unwrap();
    writeln!(&mut test_file, "{}", content).unwrap();
    test_file.seek(std::io::SeekFrom::Start(0)).unwrap();
    BufReader::new(test_file)
}

// The following code parses an example ACE file and saves it the resulting
// `AceIsotopeData` is globally accesible for testing.
lazy_static! {
    pub static ref PARSED_ACE_FILE: Mutex<Option<AceIsotopeData>> = Mutex::new(None);
}

#[allow(dead_code)]
pub fn get_parsed_ascii_for_testing() -> AceIsotopeData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<AceIsotopeData>> = PARSED_ACE_FILE.lock().unwrap();
    if data.is_none() {
        let start = std::time::SystemTime::now();
        let parsed_ace = AceIsotopeData::from_file("test_files/hydrogen_test_file").unwrap();
        println!(
            "Time to parse ACE file: {} sec",
            std::time::SystemTime::now().duration_since(start).unwrap().as_secs_f32()
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
        let path = "test_files/hydrogen_test_file";
        assert!(is_ascii_file(path).unwrap());
    }

    #[test]
    fn test_compute_temperature_from_kT() {
        let kT = 8.617333262e-8;
        let expected_temperature = 1000.0; // Kelvin
        assert!((compute_temperature_from_kT(kT) - expected_temperature).abs() < 1e-9);
    }
}