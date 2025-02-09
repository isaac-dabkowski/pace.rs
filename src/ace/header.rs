use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use crate::ace::utils;
use crate::ace::binary_format::AceBinaryMmap;

#[derive(Clone, Debug)]
pub struct AceHeader {
    pub zaid: String,
    pub szaid: Option<String>,
    pub atomic_mass_fraction: f64,
    pub kT: f64,
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
        let kT: f64 = split_legacy_header[2].parse()?;
        let temperature = utils::compute_temperature_from_kT(kT);

        Ok(Self { zaid, szaid, atomic_mass_fraction, kT, temperature })
    }

    pub fn from_file(mmap: &AceBinaryMmap) -> Result<Self, Box<dyn Error>> {
        let header_bytes = mmap.header_bytes();
        let mut offset = 0;
        // Read SZAID (first 16 bytes)
        let szaid_str = String::from_utf8(header_bytes[offset..offset + 16].trim_ascii_end().to_vec()).unwrap();
        offset += 16;

        let szaid = {
            if szaid_str.is_empty() {
                None
            } else {
                Some(szaid_str)
            }
        };

        // Read ZAID (next 16 bytes)
        let zaid = String::from_utf8(header_bytes[offset..offset + 16].trim_ascii_end().to_vec()).unwrap();
        offset += 16;

        // Read atomic mass fraction
        let atomic_mass_fraction = f64::from_ne_bytes(header_bytes[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // Read kT
        let kT = f64::from_ne_bytes(header_bytes[offset..offset + 8].try_into().unwrap());

        // Calculate temperature in Kelvin
        let temperature = utils::compute_temperature_from_kT(kT);

        Ok(Self {
            zaid,
            szaid,
            atomic_mass_fraction,
            kT,
            temperature,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_header_parsing() {
        let parsed_ace = get_parsed_test_file().await;
        assert_eq!(parsed_ace.header.szaid, Some(String::from("1100.800nc")));
        assert_eq!(parsed_ace.header.zaid, String::from("1100.00c"));
        assert_eq!(parsed_ace.header.atomic_mass_fraction, 99.999);
        assert_eq!(parsed_ace.header.kT, 2.5301e-08);
        assert_eq!(parsed_ace.header.temperature, 293.6059129982851);
    }
}

#[cfg(test)]
mod ascii_tests {
    use super::*;

    #[test]
    fn test_2_0_1_header_parsing() {
        // Simulate ACE file content for the ">2.0.0" header format
        let header_2_0_1 = concat!(
            "2.0.1                    1100.800nc         ENDF/B-VIII.0\n",
            "    99.999   2.5301e-08 2025-02-05    2\n",
            "  1100.00c    99.999  2.5301E-08   05/02/18\n",
            "H100 TEST (author)  Reference some_report by Author, A.B, et al.    mat 123\n"
        );
        let mut reader = utils::create_reader_from_string(header_2_0_1);

        // Parse the header
        let header = AceHeader::from_ascii_file(&mut reader).expect("Failed to parse version >2.0.0 header");

        // Check fields
        assert_eq!(header.zaid, "1100.00c");
        assert_eq!(header.szaid, Some("1100.800nc".to_string()));
        assert!((header.atomic_mass_fraction - 99.999).abs() < 1e-6);
        assert!((header.kT - 2.5301e-08).abs() < 1e-6);
        assert!((header.temperature - 293.605912998).abs() < 1e-6);
    }

    #[test]
    fn test_legacy_header_parsing() {
        // Simulate ACE file content for the legacy header format
        let legacy_header = concat!(
            "  1100.00c    99.999  2.5301E-08   05/02/18\n",
            "H100 TEST (author)  Reference some_report by Author, A.B, et al.    mat 123\n"
        );
        let mut reader = utils::create_reader_from_string(legacy_header);

        // Parse the header
        let header = AceHeader::from_ascii_file(&mut reader).expect("Failed to parse legacy header");

        // Check fields
        assert_eq!(header.zaid, "1100.00c");
        assert_eq!(header.szaid, None);
        assert!((header.atomic_mass_fraction - 99.999).abs() < 1e-6);
        assert!((header.kT - 2.5301e-08).abs() < 1e-6);
        assert!((header.temperature - 293.605912998).abs() < 1e-6);
    }
}
