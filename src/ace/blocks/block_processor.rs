use std::error::Error;
use std::collections::HashMap;
use strum::IntoEnumIterator;

use crate::ace::binary_format::AceBinaryMmap;
use crate::ace::blocks::{
    DataBlockType,
    ESZ,
    MTR,
    LSIG,
    SIG,
    LQR,
    NU,
    DNU,
    BDD,
};
use crate::ace::blocks::block_traits::{PullFromXXS, Process};
use crate::ace::arrays::{JxsArray, NxsArray};

#[derive(Clone, Debug, Default)]
pub struct DataBlocks {
    pub ESZ: Option<ESZ>,
    pub MTR: Option<MTR>,
    pub LSIG: Option<LSIG>,
    pub SIG: Option<SIG>,
    pub LQR: Option<LQR>,
    pub NU: Option<NU>,
    pub DNU: Option<DNU>,
    pub BDD: Option<BDD>,
}

impl DataBlocks {
    // Create a new BlockProcessor from a binary XXS array, the NXS and JXS array are used to
    // determine the start and end locations of each block
    pub fn from_file(mmap: &AceBinaryMmap, nxs_array: &NxsArray, jxs_array: &JxsArray) -> Result<Self, Box<dyn Error>> {
        // Recall that this array is returned as f64's, we will parse these values back to
        // integers where appropriate later
        let xxs_array: &[f64] = mmap.xxs_array();

        // Split XXS array into raw text correspoding to each block
        let block_map = DataBlocks::split_xxs_into_blocks(nxs_array, jxs_array, xxs_array);

        // Process the data blocks
        let data_blocks = DataBlocks::process_data_blocks(block_map, nxs_array, jxs_array);

        Ok( data_blocks )
    }

    fn split_xxs_into_blocks<'a>(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> HashMap<DataBlockType, &'a [f64]> {
        let mut block_map: HashMap<DataBlockType, &'a [f64]> = HashMap::default();
        // Loop over all possible DataBlockTypes
        for block_type in DataBlockType::iter() {
            // If the block type's start index is non-zero, the block is present in the XXS array
            let start_index = jxs_array.get(&block_type);
            if start_index != 0 {
                // Pull the block from the XXS array (if procedure to do so has been implemented)
                if let Some(block_data) = DataBlocks::pull_block_from_xxs_array(&block_type, nxs_array, jxs_array, xxs_array) {
                    block_map.insert(block_type, block_data);
                }
            }
        }
        block_map
    }

    fn pull_block_from_xxs_array<'a>(block_type: &DataBlockType, nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> Option<&'a [f64]> {
        match block_type {
            DataBlockType::ESZ => Some(ESZ::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            DataBlockType::MTR => Some(MTR::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            DataBlockType::LSIG => Some(LSIG::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            DataBlockType::SIG => Some(SIG::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            DataBlockType::LQR => Some(LQR::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            DataBlockType::NU => Some(NU::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            DataBlockType::DNU => Some(DNU::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            DataBlockType::BDD => Some(BDD::pull_from_xxs_array(nxs_array, jxs_array, xxs_array)),
            _ => {
                // println!("DataBlockType {} was found in XXS array, but its parsing has not been implemented yet!", block_type);
                None
            }
        }
    }

    // Process data blocks from a binary ACE file
    fn process_data_blocks(block_map: HashMap<DataBlockType, &[f64]>, nxs_array: &NxsArray, jxs_array: &JxsArray) -> Self {
        // -------------------------------
        // Blocks which are always present
        // -------------------------------
        // Energy grid
        let esz_data = block_map.get(&DataBlockType::ESZ).unwrap();
        let esz = Some(ESZ::process(esz_data, nxs_array));

        // -------------------------------------------
        // Blocks present if isotope has reactions
        // other than elastic scattering (NXS(4) != 0)
        // -------------------------------------------
        let (mtr, lqr, lsig, sig) = if nxs_array.ntr != 0 {
            // Reaction MT values
            let mtr_data = block_map.get(&DataBlockType::MTR).unwrap();
            let mtr = MTR::process(mtr_data, ());
            // Q values
            let lqr_data = block_map.get(&DataBlockType::LQR).unwrap();
            let lqr = LQR::process(lqr_data, &mtr);
            // Cross section locations
            let lsig_data = block_map.get(&DataBlockType::LSIG).unwrap();
            let lsig = LSIG::process(lsig_data, ());
            // Cross section values
            let sig_data = block_map.get(&DataBlockType::SIG).unwrap();
            let sig = SIG::process(sig_data, (&mtr, &lsig, esz.as_ref().unwrap()));
            (
                Some(mtr),
                Some(lqr),
                Some(lsig),
                Some(sig),
            )
        } else {
            (None, None, None, None)
        };

        // -------------------------------------------
        // Blocks present if fission nu data is
        // available (JXS(2) != 0)
        // -------------------------------------------
        let (nu, dnu, bdd) = if jxs_array.get(&DataBlockType::NU) != 0 {
            // Fission nu values
            let nu_data = block_map.get(&DataBlockType::NU).unwrap();
            let nu = NU::process(nu_data, jxs_array);
            // Fission dnu values
            let dnu_data = block_map.get(&DataBlockType::DNU).unwrap();
            let dnu = DNU::process(dnu_data, ());
            // Fission bdd values
            let bdd_data = block_map.get(&DataBlockType::BDD).unwrap();
            let bdd = BDD::process(bdd_data, nxs_array);
            (
                Some(nu),
                Some(dnu),
                Some(bdd),
            )
        } else {
            (None, None, None)
        };

        Self {
            ESZ: esz,
            MTR: mtr,
            LSIG: lsig,
            SIG: sig,
            LQR: lqr,
            DNU: dnu,
            NU: nu,
            BDD: bdd,
        }
    }
}

impl std::fmt::Display for DataBlocks {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
