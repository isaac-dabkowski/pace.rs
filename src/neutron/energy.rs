use std::collections::BTreeMap;

use hdf5;

use crate::utils::helpers::parse_temp_k;

use super::error::IncidentNeutronError;

#[derive(Debug)]
pub struct NeutronEnergy {
    pub by_temp_k: BTreeMap<u64, Vec<f64>>,
}

impl NeutronEnergy {
    pub fn from_group(group: &hdf5::Group) -> Result<Self, IncidentNeutronError> {
        let mut by_temp_k: BTreeMap<u64, Vec<f64>> = BTreeMap::new();

        for name in group.member_names()? {
            if let Some(tk) = parse_temp_k(&name) {
                let ds = group.dataset(&name)?;
                let dtype_str = format!("{:?}", ds.dtype()?);

                // Prefer f64, but fall back to f32 datasets and up-convert.
                let energies: Vec<f64> = match ds.read_raw::<f64>() {
                    Ok(v) => v,
                    Err(e_f64) => {
                        let tmp: Vec<f32> = ds.read_raw::<f32>().map_err(|e_f32| {
                            IncidentNeutronError::InvalidH5Input(format!(
                                "Error reading energy dataset '{name}' (dtype={dtype_str}) as f64 ({e_f64}) or f32 ({e_f32})"
                            ))
                        })?;
                        tmp.into_iter().map(|x| x as f64).collect()
                    }
                };

                by_temp_k.insert(tk, energies);
            } else {
                return Err(IncidentNeutronError::InvalidH5Input(format!(
                    "parsing of dataset {name} failed while parsing energies"
                )))
            }
        }

        Ok(Self { by_temp_k })
    }

    /// Return the index in the energy grid for `temperature` where `energy`
    /// would be inserted to preserve ordering (i.e. the slot index).
    /// Returns:
    /// - `MissingTemperature` if `temperature` is not present.
    /// - `NoXsAtEnergy` if `energy` is outside the available grid for that temperature.
    pub fn energy_index(&self, temperature: u64, energy: f64) -> Result<usize, IncidentNeutronError> {
        let grid = self
            .by_temp_k
            .get(&temperature)
            .ok_or_else(|| IncidentNeutronError::MissingTemperature(temperature, energy))?;

        if grid.is_empty() {
            return Err(IncidentNeutronError::NoXsAtEnergy(format!(
                "No cross sections available at {temperature} K (empty grid)"
            )));
        }

        let min_e = grid.first().copied().unwrap();
        let max_e = grid.last().copied().unwrap();
        if energy < min_e || energy > max_e {
            return Err(IncidentNeutronError::NoXsAtEnergy(format!(
                "Requested energy {energy} outside grid [{min_e}, {max_e}] at {temperature} K"
            )));
        }

        let idx = match grid.binary_search_by(|v| v
            .partial_cmp(&energy)
            .unwrap_or(std::cmp::Ordering::Less)) {
            Ok(i) | Err(i) => i,
        };
        Ok(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use tempfile::NamedTempFile;

    use crate::utils::get_parsed_neutron_file;

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_energy_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        let energy = &neutron.energy;
        assert_eq!(energy.by_temp_k.len(), 3);
        assert_eq!(energy.by_temp_k.get(&300u64).unwrap(), &vec![1.0e-5, 1.0e+0, 1.0e+7]);
        assert_eq!(energy.by_temp_k.get(&600u64).unwrap(), &vec![2.0e-5, 2.0e+0, 2.0e+7]);
        assert_eq!(energy.by_temp_k.get(&1200u64).unwrap(), &vec![3.0e-5, 3.0e+0, 3.0e+7]);
    }

    #[test]
    fn test_invalid_energy_input() {
        let tmp = NamedTempFile::new().expect("create temp file");
        let file = hdf5::File::create(tmp.path()).expect("create hdf5 file");
        let group = file
            .create_group("energy")
            .expect("create energy group");

        // Invalid data set
        group
            .new_dataset::<f64>()
            .shape(&[1])
            .create("invalid")
            .expect("create invalid dataset")
            .write_raw(&[9.99f64])
            .expect("write invalid data");

        assert!(NeutronEnergy::from_group(&group).is_err());
    }

    #[test]
    fn test_energy_index() {
        let ne = NeutronEnergy {
            by_temp_k: BTreeMap::from([(300u64, vec![1.0, 2.0, 3.0])]),
        };

        // Exact match
        assert_eq!(ne.energy_index(300, 2.0).unwrap(), 1);
        // Between points: should slot between 1.0 and 2.0
        assert_eq!(ne.energy_index(300, 1.5).unwrap(), 1);
        // Below first: should error with NoXsAtEnergy
        let err_low = ne.energy_index(300, 0.5).unwrap_err();
        let msg_low = format!("{err_low}");
        assert!(msg_low.contains("NoXsAtEnergy") || msg_low.contains("outside grid"));
        // Above last: should error with NoXsAtEnergy
        let err_high = ne.energy_index(300, 4.0).unwrap_err();
        let msg_high = format!("{err_high}");
        assert!(msg_high.contains("NoXsAtEnergy") || msg_high.contains("outside grid"));

        // Missing temperature should error
        let err = ne.energy_index(900, 2.0);
        assert!(err.is_err());
    }
}
