use std::collections::HashMap;
use std::path::Path;

use hdf5;

use crate::IncidentNeutronError;
use crate::utils::PERIODIC_TABLE;

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

        // Get the isotope name
        let name = String::from(path.file_stem().unwrap().to_string_lossy());

        // Get atributes from top level group
        let top_level_group = hdf5.group(&name)?;
        let atomic_number: i64 = top_level_group.attr("Z")?.read_scalar()?;
        let mass_number: i64 = top_level_group.attr("A")?.read_scalar()?;
        let atomic_weight_ratio: f64 = top_level_group.attr("atomic_weight_ratio")?.read_scalar()?;

        // Get atomic symbol
        let element = PERIODIC_TABLE.get(atomic_number as usize).copied()
            .unwrap_or(Err(IncidentNeutronError::Neutron(format!("Element number {atomic_number} not recognized")))?);
        let atomic_symbol = element.symbol.to_owned();

        // TEMP VALUES
        let metastable = true;
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
    async fn test_parse_test_file() {
        get_parsed_neutron_file().await;
    }
}
