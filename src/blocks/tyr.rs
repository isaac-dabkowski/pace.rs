use std::collections::HashMap;
use std::ops::Deref;

use crate::arrays::Arrays;
use crate::blocks::{BlockType, MTR};
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// TYR data block
//
// This information on neutron release for different reactions, as well
// as the frame of reference (center of mass vs. laboratory) for the
// reactions.
//=====================================================================
#[derive(Debug, Clone, PartialEq)]
pub struct TYR ( pub HashMap<usize, ExitingNeutronData> );

impl Deref for TYR {
    type Target = HashMap<usize, ExitingNeutronData>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> PullFromXXS<'a> for TYR {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We expect TYR if NXS(4) (NTR) != 0
        let has_xs_other_than_elastic = arrays.nxs.ntr != 0;

        // Get the starting index of the block in the XXS array
        let block_start = get_block_start(
            &BlockType::TYR,
            arrays,
            has_xs_other_than_elastic,
            "TYR is expected if NXS(4) (NTR) != 0, but TYR was not found.".to_string(),
        )?;

        // Calculate the block end index, see the TYR description in the ACE spec
        let num_reactions = arrays.nxs.ntr;
        let block_length = num_reactions;

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for TYR {
    type Dependencies = &'a Option<MTR>;

    fn process(data: &[f64], _arrays: &Arrays, mtr: &Option<MTR>) -> Self {
        let neutron_release: HashMap<usize, ExitingNeutronData> = data
            .iter()
            .enumerate()
            .map(|(i, &val)| (
                mtr.as_ref().unwrap()[i],
                ExitingNeutronData {
                    neutron_release: NumberOfExitingNeutrons::from(val.to_bits() as isize),
                    frame_of_reference: ExitingNeutronFrameOfReference::from(val.to_bits() as isize),
                }
            ))
            .collect();

        Self(neutron_release)
    }
}

impl<'a> TYR {
    pub fn mt_values_with_neutron_release(&self) -> Vec<usize> {
        self.iter()
            .filter(|(_, exit_neutron_data)| exit_neutron_data.neutron_release != NumberOfExitingNeutrons::Absorption)
            .map(|(mt, _)| *mt)
            .collect()
    }
}

impl std::fmt::Display for TYR {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TYR({} reactions)", self.len())
    }
}

//=====================================================================
// Helper structs and enums to describe neutron release and frame of
// reference.
//=====================================================================
// Types of neutron release
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumberOfExitingNeutrons {
    Discrete(usize),
    EnergyDependent,
    Absorption
}
// Produces a NumberOfExitingNeutrons from an isize value
// 0 = Absorption, +/- (1-4 = Discrete, 19 = EnergyDependent, > 100 = EnergyDependent)
impl From<isize> for NumberOfExitingNeutrons {
    fn from(value: isize) -> Self {
        match value.abs() {
            0 => NumberOfExitingNeutrons::Absorption,
            1 => NumberOfExitingNeutrons::Discrete(1),
            2 => NumberOfExitingNeutrons::Discrete(2),
            3 => NumberOfExitingNeutrons::Discrete(3),
            4 => NumberOfExitingNeutrons::Discrete(4),
            n if n > 100 || n == 19 => NumberOfExitingNeutrons::EnergyDependent,
            _ => {
                panic!("Invalid value in TYR describing neutron release, allowable values are 0, +/- 1-4, 19, and > 100, found: {}", value)
            }
        }
    }
}

// Scattering system type which describes the cross section tables used to determine the exiting neutrons’ angles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExitingNeutronFrameOfReference {
    CenterOfMass,
    Laboratory,
    NoRelease
}
// Produces a ExitingNeutronFrameOfReference from an isize value
impl From<isize> for ExitingNeutronFrameOfReference {
    fn from(value: isize) -> Self {
        match value {
            0 => ExitingNeutronFrameOfReference::NoRelease,
            n if n > 0 => ExitingNeutronFrameOfReference::Laboratory,
            n if n < 0 => ExitingNeutronFrameOfReference::CenterOfMass,
            _ => panic!("Unexpected value for ExitingNeutronFrameOfReference: {}", value),
        }
    }
}

// Data structure for the TYR block, which contains information on neutron release and frame of reference
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExitingNeutronData {
    pub neutron_release: NumberOfExitingNeutrons,
    pub frame_of_reference: ExitingNeutronFrameOfReference,
}
// Produces a ExitingNeutronData from an isize value
impl From<isize> for ExitingNeutronData {
    fn from(value: isize) -> Self {
        Self {
            neutron_release: NumberOfExitingNeutrons::from(value),
            frame_of_reference: ExitingNeutronFrameOfReference::from(value),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{utils::get_parsed_test_file, helpers::MTNumber};

    #[tokio::test]
    async fn test_tyr_parsing() {
        let parsed_pace = get_parsed_test_file().await;

        // Check contents
        let tyr = parsed_pace.data_blocks.TYR.unwrap();
        assert_eq!(
            tyr.get(&(MTNumber::Fission as usize)),
            Some(&ExitingNeutronData {
                neutron_release: NumberOfExitingNeutrons::EnergyDependent,
                frame_of_reference: ExitingNeutronFrameOfReference::Laboratory,
            })
        );
    }
}