use strum_macros::{Display, EnumIter};

//=====================================================================
// Enum of all block types in continuous neutron ACE files.
//=====================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter, Display)]
pub enum BlockType {
    ESZ,    // Energy table
    NU,     // Fission nu data
    MTR,    // MT array
    LQR,    // Q-value array
    TYR,    // Reaction type array
    LSIG,   // Table of cross section locators
    SIG,    // Cross sections
    LAND,   // Table of angular distribution locators
    AND,    // Angular distributions
    LDLW,   // Table of energy distribution locators
    DLW,    // Energy distributions
    GPD,    // Photon production data
    MTRP,   // Photon production MT array
    LSIGP,  // Table of photon production cross section locators
    SIGP,   // Photon production cross sections
    LANDP,  // Table of photon production angular distribution locators
    ANDP,   // Photon production angular distributions
    LDLWP,  // Table of photon production energy distribution locators
    DLWP,   // Photon production energy distributions
    YP,     // Table of yield multipliers
    FIS,    // Total fission cross section
    END,    // Last word of the conventional table
    LUND,   // Probability tables
    DNU,    // Delayed nu-bar data
    BDD,    // Basic delayed neutron precursor data
    DNEDL,  // Table of delayed neutron energy distribution locators
    DNED,   // Delayed neutron energy distributions
    PTYPE,  // Particle type array
    NTRO,   // Array containing number of particle production reactions
    NEXT,   // Table of particle production locators
}


#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", BlockType::ESZ), "ESZ");
        assert_eq!(format!("{}", BlockType::NU), "NU");
        assert_eq!(format!("{}", BlockType::MTR), "MTR");
        assert_eq!(format!("{}", BlockType::LQR), "LQR");
        assert_eq!(format!("{}", BlockType::TYR), "TYR");
        assert_eq!(format!("{}", BlockType::LSIG), "LSIG");
        assert_eq!(format!("{}", BlockType::SIG), "SIG");
        assert_eq!(format!("{}", BlockType::LAND), "LAND");
        assert_eq!(format!("{}", BlockType::AND), "AND");
        assert_eq!(format!("{}", BlockType::LDLW), "LDLW");
        assert_eq!(format!("{}", BlockType::DLW), "DLW");
        assert_eq!(format!("{}", BlockType::GPD), "GPD");
        assert_eq!(format!("{}", BlockType::MTRP), "MTRP");
        assert_eq!(format!("{}", BlockType::LSIGP), "LSIGP");
        assert_eq!(format!("{}", BlockType::SIGP), "SIGP");
        assert_eq!(format!("{}", BlockType::LANDP), "LANDP");
        assert_eq!(format!("{}", BlockType::ANDP), "ANDP");
        assert_eq!(format!("{}", BlockType::LDLWP), "LDLWP");
        assert_eq!(format!("{}", BlockType::DLWP), "DLWP");
        assert_eq!(format!("{}", BlockType::YP), "YP");
        assert_eq!(format!("{}", BlockType::FIS), "FIS");
        assert_eq!(format!("{}", BlockType::END), "END");
        assert_eq!(format!("{}", BlockType::LUND), "LUND");
        assert_eq!(format!("{}", BlockType::DNU), "DNU");
        assert_eq!(format!("{}", BlockType::BDD), "BDD");
        assert_eq!(format!("{}", BlockType::DNEDL), "DNEDL");
        assert_eq!(format!("{}", BlockType::DNED), "DNED");
        assert_eq!(format!("{}", BlockType::PTYPE), "PTYPE");
        assert_eq!(format!("{}", BlockType::NTRO), "NTRO");
        assert_eq!(format!("{}", BlockType::NEXT), "NEXT");
    }

    #[test]
    fn test_equality() {
        assert_eq!(BlockType::ESZ, BlockType::ESZ);
        assert_ne!(BlockType::ESZ, BlockType::NU);
    }

    #[test]
    fn test_clone() {
        let original = BlockType::ESZ;
        let clone = original.clone();
        assert_eq!(original, clone);
    }

    #[test]
    fn test_hash() {

        let mut hasher = DefaultHasher::new();
        BlockType::ESZ.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = DefaultHasher::new();
        BlockType::ESZ.hash(&mut hasher);
        let hash2 = hasher.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_iter() {
        let mut count = 0;
        for _ in BlockType::iter() {
            count += 1;
        }
        assert_eq!(count, 30); // Ensure the count matches the number of enum variants
    }
}