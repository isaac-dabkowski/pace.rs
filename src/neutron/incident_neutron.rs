use std::collections::HashMap;
use std::path::Path;

use hdf5;

use crate::IncidentNeutronError;
use crate::utils::{constants, PERIODIC_TABLE};

use super::energy::NeutronEnergy;
use super::kts::KTs;

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
    // Shared energy grid across which cross sections are tabulated for different temperatures
    pub energy: NeutronEnergy,
    // Temperatures for which data is available, stored as kT in eV keyed by temperature in K
    pub kts: KTs,

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

        // Confirm that this is a neutron data file.
        let root = hdf5.group("/")?;
        // Filetype is stored as a fixed-length ASCII string ("data_neutron")
        // type FixedStr = hdf5::types::FixedAscii<constants::MAX_STR_LEN>;
        let filetype_val: hdf5::types::FixedAscii<{ constants::MAX_STR_LEN }> = root.attr("filetype")?.read_scalar()?;
        let filetype_str = filetype_val.as_str();
        if filetype_str != constants::NEUTRON_DATA_FILETYPE {
            return Err(IncidentNeutronError::InvalidH5Input(format!(
                "Expected 'data_neutron' filetype, received {}",
                filetype_str
            )));
        };

        // Get the isotope name
        let name = root.member_names()?.first().unwrap().to_string();

        // Get atributes from top level group
        let isotope_group = hdf5.group(&name)?;
        let atomic_number = isotope_group.attr("Z")?.read_scalar::<i64>()?;
        let mass_number = isotope_group.attr("A")?.read_scalar::<i64>()?;
        let atomic_weight_ratio = isotope_group.attr("atomic_weight_ratio")?.read_scalar::<f64>()?;
        let metastable_id = isotope_group.attr("metastable")?.read_scalar::<i64>()?;
        let metastable = match metastable_id {
            0 => false,
            1 => true,
            _ => { return Err(IncidentNeutronError::InvalidH5Input(format!("Metastable value {metastable_id} not recognized"))) }
        };

        // Get atomic symbol
        let element = *PERIODIC_TABLE
            .get(&(atomic_number as u8))
            .ok_or_else(|| {
                IncidentNeutronError::InvalidH5Input(format!("Element number {atomic_number} not recognized"))
            })?;
        let atomic_symbol = element.symbol.to_owned();

        // Energy vector
        let energy = NeutronEnergy::from_group(
            &isotope_group.group(constants::ENERGY_GROUP_NAME)?
        )?;

        // kTs values
        let kts = KTs::from_group(&isotope_group.group("kTs")?)?;

        // Placeholders
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
            kts,
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

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_name_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.name, "H100");
    }

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_Z_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.atomic_number, 1);
    }

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_A_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.mass_number, 100);
    }

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_symbol_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.atomic_symbol, "H");
    }

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_metastable_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.metastable, false);
    }

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_atomic_weight_ratio_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        assert_eq!(neutron.atomic_weight_ratio, 100.01);
    }
}
