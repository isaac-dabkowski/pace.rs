use std::collections::BTreeMap;

use hdf5;

use crate::utils::helpers::parse_temp_k;

use super::error::IncidentNeutronError;

#[derive(Clone, Debug)]
pub struct KT {
    pub temp_k: u64,
    pub kt_ev: f64,
}

#[derive(Debug)]
pub struct KTs {
    pub by_temp_k: BTreeMap<u64, f64>,
}

impl KTs {
    pub fn from_group(group: &hdf5::Group) -> Result<Self, IncidentNeutronError> {
        let mut by_temp_k: BTreeMap<u64, f64> = BTreeMap::new();

        for name in group.member_names()? {
            if let Some(tk) = parse_temp_k(&name) {
                let ds = group.dataset(&name)?;
                let value: f64 = ds.read_scalar()?;
                by_temp_k.insert(tk, value);
            } else {
                return Err(IncidentNeutronError::InvalidH5Input(format!(
                    "parsing of dataset {name} failed while parsing kTs"
                )));
            }
        }

        Ok(Self { by_temp_k })
    }

    pub fn as_vec(&self) -> Vec<KT> {
        self.by_temp_k
            .iter()
            .map(|(&temp_k, &kt_ev)| KT { temp_k, kt_ev })
            .collect()
    }

    pub fn temperatures_k(&self) -> Vec<u64> {
        self.by_temp_k.keys().copied().collect()
    }

    pub fn temperatures_eV(&self) -> Vec<f64> {
        self.by_temp_k.values().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::get_parsed_neutron_file;

    #[cfg(not(feature = "local"))]
    #[tokio::test]
    async fn test_kts_parsing() {
        let neutron = get_parsed_neutron_file().await.as_ref().unwrap();
        let kts = &neutron.kts;

        assert_eq!(kts.by_temp_k.len(), 3);
        assert_eq!(kts.by_temp_k.get(&250u64).unwrap(), &0.021543);
        assert_eq!(kts.by_temp_k.get(&600u64).unwrap(), &0.051704);
        assert_eq!(kts.by_temp_k.get(&1200u64).unwrap(), &0.10341);

        assert_eq!(kts.temperatures_k(), vec![250, 600, 1200]);
        assert_eq!(kts.temperatures_eV(), vec![0.021543, 0.051704, 0.10341]);
    }
}
