use std::error::Error;

use crate::ace::binary_format::AceBinaryMmap;

// Pair of values used in S alpha beta calculations
#[derive(Debug, Clone, PartialEq)]
pub struct IzawPair {
    pub za: usize,  // ZA of isotope
    pub iz: f64,    // Atomic weight ratio
}

impl IzawPair {
    pub fn new(za: usize, iz: f64) -> Self {
        Self { za, iz }
    }
}

// Represents the IZAW array from an ACE file. See page 4 of the ACE format spec for a description.
#[derive(Clone, Debug)]
pub struct IzawArray {
    pub pairs: Vec<IzawPair>
}

impl IzawArray {
    pub fn from_file(mmap: &AceBinaryMmap) -> Result<Self, Box<dyn Error>> {
        let pairs = mmap.izaw_bytes().chunks_exact(16)
            .map(
                |chunk| {
                    IzawPair::new(
                        usize::from_ne_bytes(chunk[0..8].try_into().unwrap()),
                        f64::from_ne_bytes(chunk[8..16].try_into().unwrap())
                    )
                }
            )
            .collect::<Vec<_>>();
        Ok(Self { pairs })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_iwaz_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        // Check contents
        for za_iz_pair in &parsed_ace.izaw_array.pairs {
            assert_eq!(za_iz_pair.za, 0);
            assert_eq!(za_iz_pair.iz, 0.0);
            assert_eq!(*za_iz_pair, IzawPair::new(0, 0.0));
        }
        let izaw_len = parsed_ace.izaw_array.pairs.len();
        assert_eq!(izaw_len, 16);
    }
}