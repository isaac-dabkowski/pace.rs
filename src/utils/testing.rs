#![allow(unused)]

//=====================================================================
// Utility functions to aid in testing
//=====================================================================

use anyhow::{Context, Ok, Result};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::path::Path;
use std::str::FromStr;
use std::time::Instant;
use tempfile::NamedTempFile;
use tokio::sync::OnceCell;

use hdf5;

use crate::{IncidentNeutron, time_it};

use super::yaml_to_hdf5::yaml_to_hdf5_file;

// These constants and cells hold test file paths and parsed `PaceData`
// so that they are available to all tests and parsed only once.

static TEST_INCIDENTNEUTRON: OnceCell<IncidentNeutron> = OnceCell::const_new();

#[cfg(not(feature = "local"))]
pub const TEST_HDF5: &str = "test_files/H100.h5";

// For local testing (enabled with the `local` feature)
#[cfg(feature = "local")]
pub const TEST_HDF5: &str = "test_files_local/U235.h5";

// Default test data file for fictional hydrogen-100 isotope.
pub fn create_test_hdf5_file() -> Result<()> {
    let test_files_dir = Path::new("test_files");
    let spec_path = test_files_dir.join("test_file.yaml");
    let out_path = test_files_dir.join("H100.h5");

    // Write an HDF5 file from the provided YAML
    yaml_to_hdf5_file(&spec_path, &out_path)
        .with_context(|| "While converting YAML test file to HDF5")?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdf5_to_yaml() {
        create_test_hdf5_file().unwrap();
    }
}
