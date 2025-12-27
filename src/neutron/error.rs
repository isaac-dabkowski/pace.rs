use thiserror::Error;

// Public error type
#[derive(Debug, Error)]
pub enum IncidentNeutronError {
    #[error("{0}")]
    InvalidH5Input(String),
    
    #[error("I/O error while reading file '{path}': {source}")]
    Io {#[source] source: std::io::Error, path: std::path::PathBuf,},
    
    #[error("HDF5 error: {0}")]
    Hdf5(#[from] hdf5::Error),

    #[error("{0}")]
    NoXsAtEnergy(String),

    // TODO: Add functionality to remove when these errors are called.
    #[error("Missing temperature {0} K for energy {1}")]
    MissingTemperature(u64, f64),
}