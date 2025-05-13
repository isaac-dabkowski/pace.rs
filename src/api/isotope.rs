

//=====================================================================
// Helper struct to represent an isotope.
//=====================================================================

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;

use crate::api::{PaceData, CrossSection, Reaction};
use crate::helpers;

#[derive(Clone, Debug)]
pub struct Isotope {
    pub name: String,
    pub zaid: String,
    pub szaid: Option<String>,
    pub atomic_mass_fraction: f64,
    pub kT: f64,
    pub temperature: f64,
    pub z: usize,
    pub a: usize,
    pub za: usize,
    pub reactions: HashMap<usize, Reaction>,
}

impl Isotope {
    pub async fn from_PACE<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        // Parse the PACE file
        let pace_data = PaceData::from_PACE(file_path).await?;

        // Build the Isotope from the PACE data
        let isotope = Self::from_PaceData(pace_data).await?;
        Ok(isotope)
    }

    pub async fn from_PaceData(pace_data: PaceData) -> Result<Self> {
        // Parse the PACE file

        // Pull data from PACE header
        let zaid = pace_data.header.zaid.clone();
        let szaid = pace_data.header.szaid.clone();
        let atomic_mass_fraction = pace_data.header.atomic_mass_fraction;
        let kT = pace_data.header.kT;
        let temperature = pace_data.header.temperature;
        let z = pace_data.nxs_array.z;
        let a = pace_data.nxs_array.a;
        let za = pace_data.nxs_array.za;
        let name = helpers::isotope_name_from_Z_A(z, a);

        // Create the reactions
        let reactions = Isotope::make_reactions(pace_data);

        Ok(Self {
            name,
            zaid,
            szaid,
            atomic_mass_fraction,
            kT,
            temperature,
            z,
            a,
            za,
            reactions,
        })
    }

    fn make_reactions(pace_data: PaceData) -> HashMap<usize, Reaction> {
        let reactions = DashMap::new();
    
        // First, we will get the cross sections from ESZ (total, scattering, and disappearance)
        let esz = pace_data.data_blocks.ESZ.as_ref().unwrap();
        reactions.insert(
            1 as usize,
            Reaction {
                mt: 1,
                q: None,
                cross_section: CrossSection::from_e_and_sigma(esz.energy.clone(), esz.total_xs.clone()),
            },
        );
        reactions.insert(
            2 as usize,
            Reaction {
                mt: 2,
                q: None,
                cross_section: CrossSection::from_e_and_sigma(esz.energy.clone(), esz.elastic_xs.clone()),
            },
        );
        reactions.insert(
            101 as usize,
            Reaction {
                mt: 101,
                q: None,
                cross_section: CrossSection::from_e_and_sigma(esz.energy.clone(), esz.dissapearance_xs.clone()),
            },
        );
    
        // Now we will get the rest of the reactions from the data blocks
        if let Some(sig) = pace_data.data_blocks.SIG {
            sig.par_iter().for_each(|(mt, cross_section)| {
                // Get reaction Q value
                let q = pace_data.data_blocks.LQR.as_ref().unwrap().get(mt).unwrap();
                reactions.insert(
                    *mt,
                    Reaction {
                        mt: *mt,
                        q: Some(*q),
                        cross_section: CrossSection::from_e_and_sigma(
                            cross_section.energy.clone(),
                            cross_section.xs_val.clone(),
                        ),
                    },
                );
            });
        }
    
        // Convert DashMap back to a standard HashMap
        reactions.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{get_isotope, local_get_isotope};

    #[tokio::test]
    async fn test_make_isotope() {
        let _isotope = get_isotope().await;
    }

    #[cfg(feature = "local")]
    #[tokio::test]
    async fn test_make_local_isotope() {
        let _isotope = local_get_isotope().await;
    }

    // #[tokio::test]
    // async fn test_szaid_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.szaid(), Some(String::from("1100.800nc")));
    // }

    // #[tokio::test]
    // async fn test_zaid_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.zaid(), String::from("1100.00c"));
    // }

    // #[tokio::test]
    // async fn test_atomic_mass_fraction_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.atomic_mass_fraction(), 99.999);
    // }

    // #[tokio::test]
    // async fn test_kT_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.kT(), 2.5301e-08);
    // }

    // #[tokio::test]
    // async fn test_temperature_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.temperature(), 293.6059129982851);
    // }

    // #[tokio::test]
    // async fn test_za_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.za(), 1100);
    // }

    // #[tokio::test]
    // async fn test_z_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.z(), 1);
    // }

    // #[tokio::test]
    // async fn test_a_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.a(), 100);
    // }

    // #[tokio::test]
    // async fn test_name_parsing() {
    //     let parsed_pace = get_parsed_test_file().await;
    //     assert_eq!(parsed_pace.name(), "H100");
    // }
}