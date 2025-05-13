use std::fs::File;
use std::io::BufReader;

use anyhow::Result;

use crate::utils;

//=====================================================================
// Support for the headers of ACE files. These contain high-level
// information on the isotope including the ZAID, SZAID,
// atomic mass fraction, and temperature.
//
// See the ACE Format specification for a description of the possible
// ACE header formats (>2.0.0 and legacy).
//=====================================================================

#[derive(Clone, Debug)]
pub struct Header {
    pub zaid: String,
    pub szaid: Option<String>,
    pub atomic_mass_fraction: f64,
    pub kT: f64,
    pub temperature: f64,
}

impl Header {
    // Because the header in ACE files is not a fixed size and is very small,
    // we implement a helper function here to parse the header from an ACE file.
    pub fn from_ACE(reader: &mut BufReader<File>) -> Result<Self> {
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

    pub fn from_PACE(mmap: &utils::PaceMmap) -> Result<Self> {
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

        // Read ZAID (next 16 bytes), cast to String
        let zaid = String::from_utf8(header_bytes[offset..offset + 16].trim_ascii_end().to_vec()).unwrap();
        offset += 16;

        // Read atomic mass fraction, cast to f64
        let atomic_mass_fraction = f64::from_ne_bytes(header_bytes[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // Read kT, cast to f64
        let kT = f64::from_ne_bytes(header_bytes[offset..offset + 8].try_into().unwrap());

        // Calculate temperature in Kelvin from kT
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
    use super::*;
    use tempfile::tempfile;
    use std::io::{Write, Seek, BufReader};

    use approx::assert_abs_diff_eq;

    #[tokio::test]
    async fn test_2_0_1_header_parsing() {
        // The custom test ACE file is of version 2.0.1.
        let parsed_pace = utils::get_parsed_test_file().await;
        assert_eq!(parsed_pace.header.szaid, Some(String::from("1100.800nc")));
        assert_eq!(parsed_pace.header.zaid, String::from("1100.00c"));
        assert_eq!(parsed_pace.header.atomic_mass_fraction, 99.999);
        assert_eq!(parsed_pace.header.kT, 2.5301e-08);
        assert_eq!(parsed_pace.header.temperature, 293.6059129982851);
    }

    #[test]
    fn test_legacy_header_parsing() {
        // Simulate ACE file content for the legacy header format
        let legacy_header = concat!(
            "  1100.00c    99.999  2.5301E-08   05/02/18\n",
            "H100 TEST (author)  Reference some_report by Author, A.B, et al.    mat 123\n"
        );

        // Create a temporary file and write the legacy header to it - workaround for testing.
        let mut test_file = tempfile().unwrap();
        writeln!(&mut test_file, "{}", legacy_header).unwrap();
        test_file.seek(std::io::SeekFrom::Start(0)).unwrap();
        let mut reader = BufReader::new(test_file);

        // Parse the header
        let header = Header::from_ACE(&mut reader).expect("Failed to parse legacy header");

        // Check fields
        assert_eq!(header.zaid, "1100.00c");
        assert_eq!(header.szaid, None);
        assert_abs_diff_eq!(header.atomic_mass_fraction, 99.999, epsilon=1e-5);
        assert_abs_diff_eq!(header.kT, 2.5301e-08, epsilon=1e-5);
        assert_abs_diff_eq!(header.temperature, 293.605912998, epsilon=1e-5);
    }
}
