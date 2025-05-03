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
use crate::ace::blocks::block_traits::Parse;
use crate::ace::arrays::{Arrays, JxsArray, NxsArray, XxsArray};

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
        let xxs_array: &XxsArray = mmap.xxs_array();

        // Construct our Arrays struct
        let arrays = Arrays {
            nxs: nxs_array,
            jxs: jxs_array,
            xxs: xxs_array,
        };

        // Process the data blocks from the binary ACE file
        let always_expected = true;
        let has_xs_other_than_elastic = arrays.nxs.ntr != 0;
        let is_fissile = arrays.jxs.get(&DataBlockType::NU) != 0;
        // -------------------------------
        // Blocks which are always present
        // -------------------------------
        // Energy grid
        let esz = ESZ::parse(always_expected, &arrays, ());

        // -------------------------------------------
        // Blocks present if isotope has reactions
        // other than elastic scattering (NXS(4) != 0)
        // -------------------------------------------
        // Reaction MT values
        let mtr = MTR::parse(has_xs_other_than_elastic, &arrays, ());
        // Q values
        let lqr = LQR::parse(has_xs_other_than_elastic, &arrays, &mtr);
        // Cross section locations
        let lsig = LSIG::parse(has_xs_other_than_elastic, &arrays, ());
        // Cross section values
        let sig = SIG::parse(has_xs_other_than_elastic, &arrays, (&mtr, &lsig, &esz));


        // -------------------------------------------
        // Blocks present if fission nu data is
        // available (JXS(2) != 0)
        // -------------------------------------------
        // Fission nu values
        let nu = NU::parse(is_fissile, &arrays, ());
        // Fission dnu values
        let dnu = DNU::parse(is_fissile, &arrays, ());
        // Fission bdd values
        let bdd = BDD::parse(is_fissile, &arrays, ());

        Ok(
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
        )
    }
}

impl std::fmt::Display for DataBlocks {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
