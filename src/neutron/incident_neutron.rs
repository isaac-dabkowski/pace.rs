use std::collections::HashMap;
use std::path::Path;

use hdf5::{self, file};

use crate::IncidentNeutronError;
use crate::utils::PERIODIC_TABLE;

const EXPECTED_FILETYPE: &str = "data_neutron";

#[derive(Clone, Debug)]
pub struct Reaction {
    // placeholder
}

#[derive(Clone, Debug)]
pub struct FissionEnergyRelease {
    // placeholder
}
#[derive(Clone, Debug)]
pub struct Resonances {
    // placeholder
}
#[derive(Clone, Debug)]
pub struct ResonanceCovariances {
    // placeholder
}

#[derive(Clone, Debug)]
pub struct ProbabiltyTable {
    // placeholder
}

pub struct IncidentNeutron {
    // NAme of the particle using GNDS naming convention
    pub name: String,
    // Number of protons in nucleus
    pub atomic_number: i64,
    // Number of nucleons in nucleus
    pub mass_number: i64,
    // Atomic symbol of the nuclide, e.g., 'Zr'
    pub atomic_symbol: String,
    // Metastable state of the nucleus
    pub metastable: bool,
    // Atomic weight ratio of nucleus
    pub atomic_weight_ratio: f64,
    // List of temperatures for nucleus for which data is available, units are eV
    pub kts_ev: Vec<f64>,
    // Shared energy grid across which cross sections are tabulated, units are eV
    pub energy: Vec<f64>,

    // Reactions contain cross sections, secondary and and energy distributions,
    // and metadata for different reaction types. Keys are reaction MT values.
    pub reactions: HashMap<u64, Reaction>,

    // Energy-dependent energy release from fission tabulated by component (e.g. prompt
    // neutrons, beta particles, neutronics, etc.)
    pub fission_energy: Option<FissionEnergyRelease>,
    // Resonance parameters, if available
    pub resonances: Option<Resonances>,
    // Covariance for resonance parameters, if available
    pub resonance_covariances: Option<ResonanceCovariances>,
    // Unresolved resonance probability tables, if available
    pub urr: Option<HashMap<String, ProbabiltyTable>>,
}


impl IncidentNeutron {
    pub async fn from_hdf5(path: &Path) -> Result<Self, IncidentNeutronError> {
        // Open HDF5 file
        let hdf5 = hdf5::File::open(path)?;

        // Confirm that this is a neutron data file
        let filetype: hdf5::types::VarLenUnicode = hdf5
            .group("/")?
            .attr("filetype")?
            .read_scalar()?;
        if filetype.as_str() != EXPECTED_FILETYPE {
            return Err(
                IncidentNeutronError::Neutron(
                    format!(
                        "Expected 'data_neutron' filetype, received {}",
                        filetype.as_str()
                    )
                )
            )
        };

        // Get the isotope name
        let name = String::from(path.file_stem().unwrap().to_string_lossy());

        // Get atributes from top level group
        let top_level_group = hdf5.group(&name)?;
        let atomic_number: i64 = top_level_group.attr("Z")?.read_scalar()?;
        let mass_number: i64 = top_level_group.attr("A")?.read_scalar()?;
        let atomic_weight_ratio: f64 = top_level_group.attr("atomic_weight_ratio")?.read_scalar()?;
        let metastable_id: i64 = top_level_group.attr("metastable")?.read_scalar()?;
        let metastable = match metastable_id {
            0 => false,
            1 => true,
            _ => { return Err(IncidentNeutronError::Neutron(format!("Metastable value {metastable_id} not recognized"))) }
        };

        // Get atomic symbol
        let element = *PERIODIC_TABLE
            .get(&(atomic_number as u8))
            .ok_or_else(|| {
                IncidentNeutronError::Neutron(format!("Element number {atomic_number} not recognized"))
            })?;
        let atomic_symbol = element.symbol.to_owned();

        // TEMP VALUES
        let kts_ev = vec![1.0, 2.0, 3.0];
        let energy = vec![1.0, 2.0, 3.0];
        let mut reactions = HashMap::new();
        reactions.insert(1, Reaction {});
        let fission_energy = Some(FissionEnergyRelease {});
        let resonances = Some(Resonances {});
        let resonance_covariances = Some(ResonanceCovariances {});
        let urr = None;

        Ok(Self {
            name,
            atomic_number,
            mass_number,
            atomic_symbol,
            metastable,
            atomic_weight_ratio,
            kts_ev,
            energy,
            reactions,
            fission_energy,
            resonances,
            resonance_covariances,
            urr
        })
    }
}


#[cfg(test)]
mod tests {
    use crate::utils::get_parsed_neutron_file;

    #[tokio::test]
    async fn test_parse_neutron_file() {
        let res = get_parsed_neutron_file().await;
        if let Err(e) = res.as_ref() {
            // `e` is IncidentNeutronError, formatted by `thiserror`
            panic!("failed to parse test neutron file: {e}");
        }
    }

    #[tokio::test]
    async fn test_name_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.name, "H100");
    }

    #[tokio::test]
    async fn test_Z_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.atomic_number, 1);
    }

    #[tokio::test]
    async fn test_A_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.mass_number, 100);
    }

    #[tokio::test]
    async fn test_symbol_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.atomic_symbol, "H");
    }

    #[tokio::test]
    async fn test_metastable_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.metastable, false);
    }

    #[tokio::test]
    async fn test_atomic_weight_ratio_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.atomic_weight_ratio, 100.01);
    }
}
