use strum_macros::EnumIter;

// The MT number is a unique identifier for each reaction type in the ACE file
pub type MT = usize;

// Enum of all block types in continuous neutron ACE file
#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum DataBlockType {
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

impl std::fmt::Display for DataBlockType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", DataBlockType::ESZ), "ESZ");
        assert_eq!(format!("{}", DataBlockType::NU), "NU");
        assert_eq!(format!("{}", DataBlockType::MTR), "MTR");
        assert_eq!(format!("{}", DataBlockType::LQR), "LQR");
        assert_eq!(format!("{}", DataBlockType::TYR), "TYR");
        assert_eq!(format!("{}", DataBlockType::LSIG), "LSIG");
        assert_eq!(format!("{}", DataBlockType::SIG), "SIG");
        assert_eq!(format!("{}", DataBlockType::LAND), "LAND");
        assert_eq!(format!("{}", DataBlockType::AND), "AND");
        assert_eq!(format!("{}", DataBlockType::LDLW), "LDLW");
        assert_eq!(format!("{}", DataBlockType::DLW), "DLW");
        assert_eq!(format!("{}", DataBlockType::GPD), "GPD");
        assert_eq!(format!("{}", DataBlockType::MTRP), "MTRP");
        assert_eq!(format!("{}", DataBlockType::LSIGP), "LSIGP");
        assert_eq!(format!("{}", DataBlockType::SIGP), "SIGP");
        assert_eq!(format!("{}", DataBlockType::LANDP), "LANDP");
        assert_eq!(format!("{}", DataBlockType::ANDP), "ANDP");
        assert_eq!(format!("{}", DataBlockType::LDLWP), "LDLWP");
        assert_eq!(format!("{}", DataBlockType::DLWP), "DLWP");
        assert_eq!(format!("{}", DataBlockType::YP), "YP");
        assert_eq!(format!("{}", DataBlockType::FIS), "FIS");
        assert_eq!(format!("{}", DataBlockType::END), "END");
        assert_eq!(format!("{}", DataBlockType::LUND), "LUND");
        assert_eq!(format!("{}", DataBlockType::DNU), "DNU");
        assert_eq!(format!("{}", DataBlockType::BDD), "BDD");
        assert_eq!(format!("{}", DataBlockType::DNEDL), "DNEDL");
        assert_eq!(format!("{}", DataBlockType::DNED), "DNED");
        assert_eq!(format!("{}", DataBlockType::PTYPE), "PTYPE");
        assert_eq!(format!("{}", DataBlockType::NTRO), "NTRO");
        assert_eq!(format!("{}", DataBlockType::NEXT), "NEXT");
    }

    #[test]
    fn test_equality() {
        assert_eq!(DataBlockType::ESZ, DataBlockType::ESZ);
        assert_ne!(DataBlockType::ESZ, DataBlockType::NU);
    }

    #[test]
    fn test_clone() {
        let original = DataBlockType::ESZ;
        let clone = original.clone();
        assert_eq!(original, clone);
    }

    #[test]
    fn test_hash() {

        let mut hasher = DefaultHasher::new();
        DataBlockType::ESZ.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut hasher = DefaultHasher::new();
        DataBlockType::ESZ.hash(&mut hasher);
        let hash2 = hasher.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_iter() {
        let mut count = 0;
        for _ in DataBlockType::iter() {
            count += 1;
        }
        assert_eq!(count, 30); // Ensure the count matches the number of enum variants
    }
}