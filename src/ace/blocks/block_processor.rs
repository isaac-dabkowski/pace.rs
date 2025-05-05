use std::error::Error;
use std::time::Instant;

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
    TYR,
    LAND,
    AND, // Ensure AND implements a trait for dynamic dispatch
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
    pub TYR: Option<TYR>,
    pub LAND: Option<LAND>,
    pub AND: Option<AND>,
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
        let mut start = Instant::now();
        let esz = ESZ::parse(always_expected, &arrays, ());
        println!(
            "⚛️  ESZ time ⚛️ : {} us",
            start.elapsed().as_micros()
        );

        // -------------------------------------------
        // Blocks present if isotope has reactions
        // other than elastic scattering (NXS(4) != 0)
        // -------------------------------------------
        // Reaction MT values
        start = Instant::now();
        let mtr = MTR::parse(has_xs_other_than_elastic, &arrays, ());
        println!(
            "⚛️  MTR time ⚛️ : {} us",
            start.elapsed().as_micros()
        );
        // Q values
        start = Instant::now();
        let lqr = LQR::parse(has_xs_other_than_elastic, &arrays, &mtr);
        println!(
            "⚛️  LQR time ⚛️ : {} us",
            start.elapsed().as_micros()
        );
        // Cross section locations
        start = Instant::now();
        let lsig = LSIG::parse(has_xs_other_than_elastic, &arrays, ());
        println!(
            "⚛️  LSIG time ⚛️ : {} us",
            start.elapsed().as_micros()
        );
        // Cross section values
        start = Instant::now();
        let sig = SIG::parse(has_xs_other_than_elastic, &arrays, (&mtr, &lsig, &esz));
        println!(
            "⚛️  SIG time ⚛️ : {} us",
            start.elapsed().as_micros()
        );
        // Secondary neutron information
        start = Instant::now();
        let tyr = TYR::parse(has_xs_other_than_elastic, &arrays, &mtr);
        println!(
            "⚛️  TYR time ⚛️ : {} us",
            start.elapsed().as_micros()
        );

        // -------------------------------------------
        // Blocks present if fission nu data is
        // available (JXS(2) != 0)
        // -------------------------------------------
        // Fission nu values
        start = Instant::now();
        let nu = NU::parse(is_fissile, &arrays, ());
        println!(
            "⚛️  NU time ⚛️ : {} us",
            start.elapsed().as_micros()
        );
        // Fission dnu values
        start = Instant::now();
        let dnu = DNU::parse(is_fissile, &arrays, ());
        println!(
            "⚛️  DNU time ⚛️ : {} us",
            start.elapsed().as_micros()
        );
        // Fission precursor data values
        start = Instant::now();
        let bdd = BDD::parse(is_fissile, &arrays, ());
        println!(
            "⚛️  BDD time ⚛️ : {} us",
            start.elapsed().as_micros()
        );

        // --------------------------------------------------------------------------------
        // Blocks which are always present, but where having MTR makes them easier to parse
        // --------------------------------------------------------------------------------
        // Secondary neutron angular distribution locations
        start = Instant::now();
        let land = LAND::parse(always_expected, &arrays, &mtr);
        println!(
            "⚛️  LAND time ⚛️ : {} us",
            start.elapsed().as_micros()
        );
        // Secondary neutron angular distributions
        start = Instant::now();
        let and = AND::parse(always_expected, &arrays, (&tyr, &land));
        println!(
            "⚛️  AND time ⚛️ : {} us",
            start.elapsed().as_micros()
        );

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
                TYR: tyr,
                LAND: land,
                AND: and,
            }
        )
    }
}

impl std::fmt::Display for DataBlocks {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
