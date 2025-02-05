// This file contains the infrastructure to convert an ASCII ACE file into our own binary format.

// The format as follows (from start of file to end):
//    - Header section
//        - SZAID is written as ASCII bytes and padded to 16 bytes with whitespace if available,
//          if it is not available, we simply write 16 bytes of whitespace.
//        - ZAID is written as ASCII bytes and padded to 16 bytes with whitespace.
//        - kT is written as an f64.
//    - IZAW array
//        - 16 pairs of i64 / f64 values
//    - NXS array
//        - 16 i64s
//    - JXS array
//        - 32 i64s
//    - XXS array
//        - Faithfully recreated 1:1 to ASCII file (sans whitespace) using i64s or f64s where appropriate.

use std::{error::Error, fs::File, io::{BufRead, BufReader, Write}, path::Path};

use memmap2::MmapOptions;

use crate::ace::header::AceHeader;

// This function converts an ASCII ACE file into a binary format
pub fn convert_ascii_to_binary<P: AsRef<Path>>(input_path: P) -> Result<String, Box<dyn std::error::Error>> {
    // Open input file for reading
    let input_file = File::open(input_path.as_ref())?;
    let mut reader = BufReader::new(input_file);

    // File headers are inconsistent across ACE files, so we will invoke the ascii header parsing
    // we implemented earlier to pull the relavent data.
    let header = AceHeader::from_ascii_file(&mut reader)?;

    // Set the binary file name to the SZAID if it is available. Otherwise, set it to the ZAID.
    let output_filename = if let Some(ref val) = header.szaid {
        Path::new(val)
    } else {
        Path::new(&header.zaid)
    };
    let output_path = input_path.as_ref()
        .parent()
        .unwrap()
        .join(output_filename);

    // Create output file for writing
    let mut output_file = File::create(output_path.clone())?;

    // Write the header information
    match header.szaid {
        // If we have an SZAID, write it padded out with whitespace to 16 bytes.
        Some(val) => {
            let padding_length = 16 - val.len();
            output_file.write_all(val.into_bytes().as_slice())?;
            output_file.write_all(" ".repeat(padding_length).into_bytes().as_slice())?;
            
        },
        // If we don't have a SZIAD, just write 16 bytes of whitespace.
        None => {
            output_file.write_all(" ".repeat(16).into_bytes().as_slice())?;
        }
    }

    // Write ZAID padded out with whitespace to 16 bytes.
    let padding_length = 16 - header.zaid.len();
    output_file.write_all(header.zaid.into_bytes().as_slice())?;
    output_file.write_all(" ".repeat(padding_length).into_bytes().as_slice())?;

    // Deal with atomic mass fraction and kT.
    output_file.write_all(&header.atomic_mass_fraction.to_ne_bytes())?;
    output_file.write_all(&header.kT.to_ne_bytes())?;

    // Process each line into binary
    for line_result in reader.lines() {
        let line = line_result?;
        // Split line into whitespace-separated tokens
        for token in line.split_whitespace() {
            // Try parsing as integer first
            if let Ok(integer) = token.parse::<i64>() {
                output_file.write_all(&integer.to_ne_bytes())?;
            }
            // Then try parsing as float
            else if let Ok(float) = token.parse::<f64>() {
                output_file.write_all(&float.to_ne_bytes())?;
            } else {
                return Err(format!("Invalid number format: '{}'", token).into());
            }
        }
    }

    Ok(output_path.to_string_lossy().into_owned())
}

pub struct BinaryMmap {
    pub mmap: memmap2::Mmap,
    // pub header_bytes: [u8; 48],
    // pub izaw_bytes: [u8; 32 * 8],
    // pub nxs_bytes: [u8; 16 * 8],
    // pub jxs_bytes: [u8; 32 * 8],
    // pub xxs_bytes: Vec<u8>,
}

impl BinaryMmap {
    pub fn from_binary_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        // Open up the binary file
        let file = File::open(path).map_err(|e| format!("Error opening ACE binary file: {}", e))?;

        // Create a memory map of the binary file
        let mmap = unsafe { MmapOptions::new().map(&file)? };
        Ok(Self { mmap })
    }

    // Compute the XXS slice on-demand
    pub fn header_bytes(&self) -> &[u8] {
        &self.mmap[0..48]
    }
    // Compute the XXS slice on-demand
    pub fn izaw_bytes(&self) -> &[u8] {
        &self.mmap[48..304]
    }
    // Compute the XXS slice on-demand
    pub fn nxs_bytes(&self) -> &[u8] {
        &self.mmap[304..432]
    }
    // Compute the XXS slice on-demand
    pub fn jxs_bytes(&self) -> &[u8] {
        &self.mmap[432..688]
    }
    // Compute the XXS slice on-demand
    pub fn xxs_bytes(&self) -> &[u8] {
        &self.mmap[688..]
    }
}