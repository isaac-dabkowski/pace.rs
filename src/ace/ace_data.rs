use std::path::Path;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use crate::ace::header::AceHeader;
use crate::ace::utils::is_ascii_file;

#[derive(Clone)]
pub struct AceData {
    header: AceHeader
}

impl AceData {
    pub fn from_file<P: AsRef<Path>>(file_path: P) -> Result<Self, Box<dyn Error>> {
        let path = file_path.as_ref();

        // Invoke ASCII or binary parsing based on file type
        if is_ascii_file(path)? {
            // Parse ASCII file
            let ace_data = AceData::from_ascii_file(path)?;
            Ok(ace_data)
        } else {
            // Parse binary file
            todo!()
        }
    }

    // Create an AceData object from an ASCII file
    pub fn from_ascii_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path).map_err(|e| format!("Error opening ACE ASCII file: {}", e))?;
        let mut reader = BufReader::new(file);

        // Process the header
        let header = AceHeader::from_ascii_file(&mut reader)?;

        Ok(Self { header })
    }

    // Expose AceHeader fields via methods
    pub fn zaid(&self) -> String {
        self.header.zaid.clone()
    }

    pub fn szaid(&self) -> Option<String> {
        self.header.szaid.clone()
    }

    pub fn atomic_mass_fraction(&self) -> f64 {
        self.header.atomic_mass_fraction
    }

    pub fn temperature(&self) -> f64 {
        self.header.temperature
    }

    // pub fn izaw(&self) -> Vec<IzawEntry> {
    //     self.izaw.entries.clone()
    // }
}

#[cfg(test)]
mod ascii_tests {
    use crate::ace::utils::get_parsed_ascii_for_testing;

    #[test]
    fn test_szaid_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing();
        assert_eq!(parsed_ace.szaid(), Some(String::from("1001.800nc")));
    }

    #[test]
    fn test_zaid_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing();
        assert_eq!(parsed_ace.zaid(), String::from("1001.00c"));
    }

    #[test]
    fn test_atomic_mass_fraction_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing();
        assert_eq!(parsed_ace.atomic_mass_fraction(), 0.999167);
    }

    #[test]
    fn test_temperature_parsing() {
        let parsed_ace = get_parsed_ascii_for_testing();
        assert_eq!(parsed_ace.temperature(), 293.6059129982851);
    }

    // #[test]
    // fn test_izaw_parsing() {
    //     let correct_za_iz_pair =  IzawEntry::new(0, 0.0);
    //     let parsed_ace = get_parsed_ace_file_for_testing();
    //     for za_iz_pair in parsed_ace.izaw() {
    //         assert_eq!(za_iz_pair, correct_za_iz_pair);
    //     }
    //     assert_eq!(parsed_ace.izaw().len(), 16)
    // }
}

#[cfg(test)]
mod binary_tests {
}