#![allow(clippy::await_holding_lock, dead_code)]

use std::mem;
use std::sync::Mutex;
use std::io::{self, Read, Write, Seek, BufReader, BufRead};
use std::path::Path;
use std::fs::File;
use std::error::Error;
use tempfile::tempfile;
use lazy_static::lazy_static;

use crate::AceIsotopeData;
use crate::ace::header::AceHeader;

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

// Read a specified number of lines into a vector of strings
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

// This function converts an ASCII ACE file into a binary format
pub fn convert_ascii_to_binary<P: AsRef<Path>>(input_path: P) -> Result<String, Box<dyn std::error::Error>> {
    // Open input file for reading
    let input_file = File::open(input_path.as_ref())?;
    let mut reader = BufReader::new(input_file);

    // File headers are inconsistent across ACE files, so we will invoke the ascii header parsing
    // we implemented earlier to pull the relavent data.
    let header = AceHeader::from_ascii_file(&mut reader)?;

    // Set the binary file name to the SZAID if it is available. Otherwise, set it to the ZAID.
    let output_filename = if let Some(ref val) = header.szaid {
        Path::new(val)
    } else {
        Path::new(&header.zaid)
    };
    let output_path = input_path.as_ref()
        .parent()
        .unwrap()
        .join(output_filename);

    // Create output file for writing
    let mut output_file = File::create(output_path.clone())?;

    // Write the header information
    // The first byte represents if we have a SZAID or not
    match header.szaid {
        Some(val) => {
            output_file.write_all(&i64::from(1).to_ne_bytes())?;
            output_file.write_all(&(val.len() as i64).to_ne_bytes())?;
            output_file.write_all(val.into_bytes().as_slice())?;
        },
        None => {
            output_file.write_all(&i64::from(0).to_ne_bytes())?;
        }
    }
    output_file.write_all(&(header.zaid.len() as i64).to_ne_bytes())?;
    output_file.write_all(header.zaid.into_bytes().as_slice())?;
    output_file.write_all(&header.atomic_mass_fraction.to_ne_bytes())?;
    output_file.write_all(&header.kT.to_ne_bytes())?;

    // Process each line into binary
    for line_result in reader.lines() {
        let line = line_result?;
        // Split line into whitespace-separated tokens
        for token in line.split_whitespace() {
            // Try parsing as integer first
            if let Ok(integer) = token.parse::<i64>() {
                output_file.write_all(&integer.to_ne_bytes())?;
            }
            // Then try parsing as float
            else if let Ok(float) = token.parse::<f64>() {
                output_file.write_all(&float.to_ne_bytes())?;
            } else {
                return Err(format!("Invalid number format: '{}'", token).into());
            }
        }
    }

    Ok(output_path.to_string_lossy().into_owned())
}

// Read an int from ACE binary files
pub fn read_int(reader: &mut BufReader<File>) -> i64 {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer).expect("Failed to a read an int where expected");
    i64::from_ne_bytes(buffer)
}

// Read an unsigned int from ACE binary files
pub fn read_usize(reader: &mut BufReader<File>) -> usize {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer).expect("Failed to a read a usize where expected");
    usize::from_ne_bytes(buffer)
}

// Read a float from ACE binary files
pub fn read_float(reader: &mut BufReader<File>) -> f64 {
    let mut buffer = [0u8; 8];
    reader.read_exact(&mut buffer).expect("Failed to a read a float where expected");
    f64::from_ne_bytes(buffer)
}

// Read a series of ints from ACE binary files
pub fn read_ints(N: usize, reader: &mut BufReader<File>) -> Vec<i64> {
    let mut buffer = vec![0i64; N];
    let byte_buffer = unsafe {
        std::slice::from_raw_parts_mut(
            buffer.as_mut_ptr() as *mut u8,
            N * mem::size_of::<i64>(),
        )
    };
    reader.read_exact(byte_buffer).expect("Failed to read a usize where expected");
    buffer
}

// Read a series of floats from ACE binary files
pub fn read_floats(N: usize, reader: &mut BufReader<File>) -> Vec<f64> {
    let mut buffer = vec![0f64; N];
    let byte_buffer = unsafe {
        std::slice::from_raw_parts_mut(
            buffer.as_mut_ptr() as *mut u8,
            N * mem::size_of::<f64>(),
        )
    };
    reader.read_exact(byte_buffer).expect("Failed to read a usize where expected");
    buffer
}

// Read a series of unsigned ints from ACE binary files
pub fn read_usizes(N: usize, reader: &mut BufReader<File>) -> Vec<usize> {
    let mut buffer = vec![0usize; N];
    let byte_buffer = unsafe {
        std::slice::from_raw_parts_mut(
            buffer.as_mut_ptr() as *mut u8,
            N * mem::size_of::<usize>(),
        )
    };
    reader.read_exact(byte_buffer).expect("Failed to read a usize where expected");
    buffer
}

// The following code parses an example ACE file and saves it the resulting
// `AceIsotopeData` is globally accesible for testing.
lazy_static! {
    pub static ref PARSED_ACE_ASCII_FILE: Mutex<Option<AceIsotopeData>> = Mutex::new(None);
    pub static ref PARSED_ACE_BINARY_FILE: Mutex<Option<AceIsotopeData>> = Mutex::new(None);
    // pub static ref TEST_ASCII_FILENAME: &'static str = "test_files/hydrogen_test_file";
    pub static ref TEST_ASCII_FILENAME: &'static str = "test_files/uranium_test_file";
    // pub static ref TEST_BINARY_FILENAME: &'static str = "test_files/1001.800nc";
    pub static ref TEST_BINARY_FILENAME: &'static str = "test_files/92235.800nc";
}

pub async fn get_parsed_ascii_for_testing() -> AceIsotopeData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<AceIsotopeData>> = PARSED_ACE_ASCII_FILE.lock().unwrap();
    if data.is_none() {
        let start = std::time::SystemTime::now();
        let parsed_ace = AceIsotopeData::from_file(*TEST_ASCII_FILENAME).await.unwrap();
        println!(
            "⚛️  Time to parse ACE file ⚛️ : {} sec",
            std::time::SystemTime::now().duration_since(start).unwrap().as_secs_f32()
        );
        *data = Some(parsed_ace);
    }
    data.as_ref().unwrap().clone()
}

pub async fn get_parsed_binary_for_testing() -> AceIsotopeData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<AceIsotopeData>> = PARSED_ACE_BINARY_FILE.lock().unwrap();
    if data.is_none() {
        let start = std::time::SystemTime::now();
        let parsed_ace = AceIsotopeData::from_file(*TEST_BINARY_FILENAME).await.unwrap();
        println!(
            "⚛️  Time to parse ACE file ⚛️ : {} sec",
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

    #[test]
    fn test_ascii_to_binary() {
        let path = "test_files/hydrogen_test_file";
        let _ = convert_ascii_to_binary(path);
        assert!(convert_ascii_to_binary(path).is_ok());
    }
}