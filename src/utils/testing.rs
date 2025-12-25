#![allow(unused)]

//=====================================================================
// Utility functions to aid in testing
//=====================================================================

use anyhow::{Context, Ok, Result};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;
use tempfile::NamedTempFile;
use tokio::sync::OnceCell;

use hdf5;

// Helper macro: in tests, time a parse and print; in other builds, just
// evaluate the expression without any timing or println noise.
#[cfg(test)]
#[macro_export]
macro_rules! time_it {
    ($label:expr, $expr:expr) => {{
        let start = Instant::now();
        let result = $expr;
        println!("⚛️  {}  ⚛️ : {} μs", $label, start.elapsed().as_micros());
        result
    }};
}

#[cfg(not(test))]
#[macro_export]
macro_rules! time_it {
    ($label:expr, $expr:expr) => {{ $expr }};
}

// These constants and cells hold test file paths and parsed `PaceData`
// so that they are available to all tests and parsed only once.

static TEST_PACE_DATA: OnceCell<i32> = OnceCell::const_new();

#[cfg(not(feature = "local"))]
pub const TEST_ACE: &str = "test_files/test.h5";

// For local testing (enabled with the `local` feature)
#[cfg(feature = "local")]
pub const TEST_ACE: &str = "test_files_local/U235.h5";


pub fn write_scalar_attribute<V: hdf5::H5Type>(
    group: &hdf5::Group,
    name: &str,
    value: V,
) -> Result<()> {
    let attr = group
        .new_attr::<V>()
        .create(name)
        .with_context(|| {format!("Failed to create attribute {}", name)})?;
    attr.write_scalar(&value)
        .with_context(|| {format!("Failed to write attribute {}", name)})?;
    Ok(())
}

// Default test data file for fictional hydrogen-100 isotope.
pub fn create_test_hdf5_file() -> Result<()> {
    let dir = Path::new("test_files");
    std::fs::create_dir_all(dir).with_context(|| {
        format!("Failed to create test_files directory at {}", dir.display())
    })?;

    let file_path = dir.join("H100.h5");

    // Create (or truncate) the HDF5 file.
    let file = hdf5::File::create(&file_path).with_context(|| {
        format!(
            "Failed to create HDF5 test file at {}",
            file_path.display()
        )
    })?;

    // Create the top-level group.
    let top_level_group = file.create_group("H100")?;
    write_scalar_attribute(
        &top_level_group,
        "filetype",
        hdf5::types::VarLenAscii::from_ascii("data_neutron")?,
    )?;
    write_scalar_attribute(&top_level_group, "version", [3, 0])?;

    Ok(())
}

// The following code parses example ACE files and caches the resulting
// `PaceData` so it is globally accessible for testing.
pub async fn get_parsed_test_file() -> i32 {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual ACE file.
    let data = TEST_PACE_DATA
        .get_or_init(|| async {
            create_test_hdf5_file();
            // Path to ACE test file

            // let ace_path = Path::new(TEST_ACE);

            // // Also copy the generated PACE file into the same directory as the
            // // original ACE test file so it can be inspected later if desired.
            // if let Some(dir) = ace_path.parent() {
            //     if let Some(fname) = tmp_pace_path.file_name() {
            //         let saved_pace_path = dir.join(fname);
            //         // Ignore copy errors here; the temp file is still usable
            //         let _ = std::fs::copy(&tmp_pace_path, &saved_pace_path);
            //     }
            // }

            // // Parse into PaceData
            // let parsed_ace = time_it!(
            //     format!("Time to parse test PACE file {}", tmp_pace_path.display()),
            //     PaceData::from_file(&tmp_pace_path).await.unwrap()
            // );
            1
        })
        .await;

    data.clone()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_test_file() {
        get_parsed_test_file().await;
    }
}
