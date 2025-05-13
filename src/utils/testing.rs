#![allow(clippy::await_holding_lock, unused)]

//=====================================================================
// Utility functions to aid in accelerating testing
//=====================================================================

use std::sync::Mutex;
use std::io::{self, Read, Write, Seek, BufReader, BufRead};
use std::path::Path;
use std::fs::File;
use std::error::Error;
use std::time::Instant;
use tempfile::tempfile;
use lazy_static::lazy_static;
use anyhow::{Context, Result};

use crate::api::{Isotope, PaceData};
use crate::utils::binary_format::convert_ACE_to_PACE;

// These variables are used to hold filepaths in a way where
// they are accesible to all tests in all files, and where
// they can be parsed once and reused in all tests.
lazy_static! {
    // For custom ACE file available to all tests
    pub static ref TEST_PACE_DATA: Mutex<Option<PaceData>> = Mutex::new(None);
    pub static ref TEST_ACE_COMMENTED: &'static str = "test_nuclear_data_files/test_ascii_ace";
    pub static ref TEST_ACE_UNCOMMENTED: &'static str = "test_nuclear_data_files/test_ascii_ace.no_comment";
    pub static ref TEST_PACE: &'static str = "test_nuclear_data_files/1100.800nc.pace";
    pub static ref ISOTOPE: Mutex<Option<Isotope>> = Mutex::new(None);

    // For local testing
    pub static ref LOCAL_TEST_PACE_DATA: Mutex<Option<PaceData>> = Mutex::new(None);
    pub static ref LOCAL_TEST_ACE: &'static str = "test_files/uranium_test_file";
    pub static ref LOCAL_TEST_PACE: &'static str = "test_files/92235.800nc.pace";
    // pub static ref LOCAL_TEST_ACE: &'static str = "test_files/hydrogen_test_file";
    // pub static ref LOCAL_TEST_PACE: &'static str = "test_files/1001.800nc.pace";
    pub static ref LOCAL_ISOTOPE: Mutex<Option<Isotope>> = Mutex::new(None);
}

// Checks if a file is ASCII by reading the first 1 kB of the file
pub fn is_ascii_file<P: AsRef<Path>>(path: P) -> Result<bool> {
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
fn uncomment_ace_test_file() -> Result<()> {
    let commented_filename: &Path = Path::new(*TEST_ACE_COMMENTED);
    let uncommented_filename: &Path = Path::new(*TEST_ACE_UNCOMMENTED);
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
            uncommented_file.write_all(line.into_bytes().as_slice())?;
            uncommented_file.flush()?;
        }
    }
    Ok(())
}

// The following code parses an example ACE file and saves it the resulting
// `PaceData` is globally accesible for testing.
pub async fn local_get_parsed_test_file() -> PaceData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ASCII ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<PaceData>> = LOCAL_TEST_PACE_DATA.lock().unwrap();

    // Only parse the ACE file if it is not already parsed
    if data.is_none() {
        // Convert the ACE file to PACE
        let mut start = Instant::now();
        let _ = convert_ACE_to_PACE(*LOCAL_TEST_ACE);
        println!(
            "⚛️  Time to convert local ACE file to PACE ⚛️ : {:?}",
            start.elapsed()
        );

        // Parse the PACE file
        start = Instant::now();
        let parsed_ace = PaceData::from_PACE(*LOCAL_TEST_PACE).await.unwrap();
        println!(
            "⚛️  Time to parse local PACE file ⚛️ : {:?}",
            start.elapsed()
        );
        *data = Some(parsed_ace);
    }
    // Otherwise, return the already parsed data
    data.as_ref().unwrap().clone()
}

pub async fn get_parsed_test_file() -> PaceData {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ACE file.
    let mut data: std::sync::MutexGuard<'_, Option<PaceData>> = TEST_PACE_DATA.lock().unwrap();

    // Only parse the ACE file if it is not already parsed
    if data.is_none() {
        // Convert the ACE file to PACE
        uncomment_ace_test_file();
        let mut start = Instant::now();
        let _ = convert_ACE_to_PACE(*TEST_ACE_UNCOMMENTED);
        println!(
            "⚛️  Time to convert ACE test file to PACE ⚛️ : {:?}",
            start.elapsed()
        );

        // Parse the PACE file
        start = Instant::now();
        let parsed_ace = PaceData::from_PACE(*TEST_PACE).await.unwrap();
        println!(
            "⚛️  Time to parse test PACE file ⚛️ : {:?}",
            start.elapsed()
        );
        *data = Some(parsed_ace);
    }
    // Otherwise, return the already parsed data
    data.as_ref().unwrap().clone()
}

// The following code parses a `PaceData` file and then builds its Isotope.
pub async fn local_get_isotope() -> Isotope {
    let mut isotope: std::sync::MutexGuard<'_, Option<Isotope>> = LOCAL_ISOTOPE.lock().unwrap();

    // Only make the Isotope if it is not already made
    if isotope.is_none() {
        // Get the parsed PACE data
        let mut pace_data = local_get_parsed_test_file().await;
        // Build the Isotope
        let mut start = Instant::now();
        let parsed_isotope = Isotope::from_PaceData(pace_data).await.unwrap();
        println!(
            "⚛️  Time to create Isotope from local PACE file ⚛️ : {:?}",
            start.elapsed()
        );
        *isotope = Some(parsed_isotope);
    }
    // Otherwise, return the Isotope
    isotope.as_ref().unwrap().clone()
}

pub async fn get_isotope() -> Isotope {
    let mut isotope: std::sync::MutexGuard<'_, Option<Isotope>> = ISOTOPE.lock().unwrap();

    // Only make the Isotope if it is not already made
    if isotope.is_none() {
        // Get the parsed PACE data
        let mut pace_data = get_parsed_test_file().await;
        // Build the Isotope
        let mut start = Instant::now();
        let parsed_isotope = Isotope::from_PaceData(pace_data).await.unwrap();
        println!(
            "⚛️  Time to create Isotope from custom PACE file ⚛️ : {:?}",
            start.elapsed()
        );
        *isotope = Some(parsed_isotope);
    }
    // Otherwise, return the Isotope
    isotope.as_ref().unwrap().clone()
}