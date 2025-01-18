use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use crate::ace::utils;

#[derive(Debug, Clone, PartialEq)]
pub struct IzawPair {
    pub za: usize,
    pub iz: f64, // Atomic weight ratio
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
    pub fn from_ascii_file(reader: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        let mut pairs = Vec::new();
        // An IZAW array consists of 4 lines, each with four pairs of values in format 4(I7,F11.0).
        let izaw_array_text = utils::read_lines(reader, 4)?;

        // Loop over each line
        for line in izaw_array_text {
            // Split each line into 4 pairs of values
            for pair_idx in 0..4 {
                let (za, iz) = Self::parse_za_iv_pair(&line, pair_idx)?;
                pairs.push(IzawPair::new(za, iz))
            }
        }
        Ok(Self { pairs })
    }

    // Parses a pair of ZA and IZ values from a line.
    fn parse_za_iv_pair(line: &str, pair_idx: usize) -> Result<(usize, f64), Box<dyn Error>> {
        let start_idx = pair_idx * 18;
        let za_str = line[start_idx..start_idx + 7].trim();
        let iz_str = line[start_idx + 7..start_idx + 18].trim();

        let za = za_str.parse::<usize>()?;
        let iz = iz_str.parse::<f64>()?;

        Ok((za, iz))
    }
}

#[cfg(test)]
mod ascii_tests {
    use super::*;

    #[test]
    fn test_iwaz_parsing() {
        // Simulate ACE IZAW array
        let izaw_line = "      0         0.      0         0.      0         0.      0         0.\n";
        let izaw_array =  format!("{}{}{}{}", izaw_line, izaw_line, izaw_line, izaw_line);
        let mut reader = utils::create_reader_from_string(&izaw_array);

        // Parse the header
        let izaw = IzawArray::from_ascii_file(&mut reader).expect("Failed to parse IZAW array");

        // Check fields
        for za_iz_pair in &izaw.pairs {
            assert_eq!(za_iz_pair.za, 0);
            assert_eq!(za_iz_pair.iz, 0.0);
            assert_eq!(*za_iz_pair, IzawPair::new(0, 0.0));
        }
        let izaw_len = izaw.pairs.len();
        assert_eq!(izaw_len, 16);
    }
}