#![allow(unused)]

//=====================================================================
// Utility functions to aid in testing
//=====================================================================

use anyhow::{Context, Ok, Result};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
use std::time::Instant;
use tempfile::NamedTempFile;
use tokio::sync::OnceCell;

use hdf5;

use crate::utils::yaml_to_hdf5::YamlHdf5Error;
use crate::{IncidentNeutron, IncidentNeutronError, time_it};

use super::yaml_to_hdf5::yaml_to_hdf5_file;

// OnceLocks are used to initialize data (parased HDF5s, IncidentNeutrons, etc.) for testing.
static GENERATED_HDF5: OnceLock<Result<(), YamlHdf5Error>> = OnceLock::new();
static TEST_INCIDENTNEUTRON: OnceCell<Result<IncidentNeutron, IncidentNeutronError>> = OnceCell::const_new();

#[cfg(not(feature = "local"))]
pub const TEST_HDF5: &str = "test_files/H100.h5";

// For local testing (enabled with the `local` feature)
#[cfg(feature = "local")]
pub const TEST_HDF5: &str = "test_files_local/U235.h5";

// Default test data file for fictional hydrogen-100 isotope.
pub fn create_test_hdf5_file() -> &'static Result<(), YamlHdf5Error> {
    let test_files_dir = Path::new("test_files");
    let spec_path = test_files_dir.join("test_file.yaml");
    let out_path = test_files_dir.join("H100.h5");
    // Write an HDF5 file from the provided YAML
    let test_hdf5 = GENERATED_HDF5.get_or_init(|| {
        yaml_to_hdf5_file(&spec_path, &out_path)
    });

    test_hdf5
}


// The following code parses example files and caches the resulting
// structs so that they are globally accessible for testing.

// TODO: Update this to use the PACE file format instead of HDF5.
// Default test data based on the canonical hydrogen test file.
pub async fn get_parsed_neutron_file() -> &'static Result<IncidentNeutron, IncidentNeutronError> {
    // In effect, this acts as a sloppy integration test as it involves
    // the parsing of an actual HDF5 file.
    let incident_neutron = TEST_INCIDENTNEUTRON
        .get_or_init(|| async {
            // Path to HDF5 test file
            let hdf5_path = Path::new(TEST_HDF5);

            // Parse into IncidentNeutron
            time_it!(
                format!("Time to parse test neutron HDF5 file {}", hdf5_path.display()),
                IncidentNeutron::from_hdf5(&hdf5_path).await
            )
        })
        .await;
    incident_neutron
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hdf5_to_yaml() {
        create_test_hdf5_file();
    }
}
