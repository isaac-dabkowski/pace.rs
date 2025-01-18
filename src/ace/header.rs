use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use crate::ace::utils;

#[derive(Clone)]
pub struct AceHeader {
    pub zaid: String,
    pub szaid: Option<String>,
    pub atomic_mass_fraction: f64,
    pub temperature: f64,
}

// See page 3 of the ACE Format specification \[1\] for a description of the possible
// ACE header formats (>2.0.0 and legacy)
impl AceHeader {
    pub fn from_ascii_file(reader: &mut BufReader<File>) -> Result<Self, Box<dyn Error>> {
        // Pull first two lines
        let header = utils::read_lines(reader, 2)?;

        // Logic for pulling SZAID if the ASCII file has a version >2.0.0 header
        let (szaid, legacy_header) = if header[0].contains("2.0.") {
            let szaid = header[0]
                .split_whitespace()
                .nth(1)
                .map(|s| s.to_string());
            // We will get the remaining data from the legacy header
            let legacy_header = utils::read_lines(reader, 2)?;
            (szaid, legacy_header)
        } else {
            (None, header)
        };

        // Pull remaining data from legacy header
        let split_legacy_header: Vec<String> = legacy_header[0]
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();
        let zaid = split_legacy_header[0].clone();
        let atomic_mass_fraction: f64 = split_legacy_header[1].parse()?;
        let temperature = utils::compute_temperature_from_kT(split_legacy_header[2].parse()?);

        Ok(Self { zaid, szaid, atomic_mass_fraction, temperature })
    }
}

#[cfg(test)]
mod ascii_tests {
    use super::*;

    #[test]
    fn test_2_0_1_header_parsing() {
        // Simulate ACE file content for the ">2.0.0" header format
        let header_2_0_1 = concat!(
            "2.0.1                    6012.800nc         ENDF/B-VIII.0\n",
            "   11.893650   2.5301e-08 2018-05-02    2\n",
            "  6012.00c   11.893650  2.5301E-08   05/02/18\n",
            "C12 Lib80x (jlconlin)  Reference LA-UR-18-24034 by Conlin, J.L., et al.  mat 625\n",
        );
        let mut reader = utils::create_reader_from_string(header_2_0_1);

        // Parse the header
        let header = AceHeader::from_ascii_file(&mut reader).expect("Failed to parse version >2.0.0 header");

        // Check fields
        assert_eq!(header.zaid, "6012.00c");
        assert_eq!(header.szaid, Some("6012.800nc".to_string()));
        assert!((header.atomic_mass_fraction - 11.893650).abs() < 1e-6);
        assert!((header.temperature - utils::compute_temperature_from_kT(2.5301e-08)).abs() < 1e-6);
    }

    #[test]
    fn test_legacy_header_parsing() {
        // Simulate ACE file content for the legacy header format
        let legacy_header = concat!(
            " 26054.00c   53.476240  2.5301E-08   05/01/18\n",
            "Fe54 Lib80x (jlconlin)  Reference LA-UR-18-24034 by Conlin, J.L., et al. mat2625\n",
        );
        let mut reader = utils::create_reader_from_string(legacy_header);

        // Parse the header
        let header = AceHeader::from_ascii_file(&mut reader).expect("Failed to parse legacy header");

        // Check fields
        assert_eq!(header.zaid, "26054.00c");
        assert_eq!(header.szaid, None);
        assert!((header.atomic_mass_fraction - 53.476240).abs() < 1e-6);
        assert!((header.temperature - utils::compute_temperature_from_kT(2.5301e-08)).abs() < 1e-6);
    }
}