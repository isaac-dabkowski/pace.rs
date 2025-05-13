use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::Mutex,
};

use rayon::prelude::*;
use memmap2::MmapOptions;
use anyhow::{Context, Result};

use crate::utils;
use crate::header::Header;

//=====================================================================
// Infrastructure to convert an ASCII ACE file into our own "PACE"
// binary file format. This massively speeds up the parsing of ACE
// files, as it allows us to use memory-mapped files to access the data
// in a zero-copy manner.
//=====================================================================

// The format as follows (from start of file to end):
//    - Header section
//        - SZAID is written as ASCII bytes and padded to 16 bytes with whitespace if available,
//          if it is not available, we simply write 16 bytes of whitespace.
//        - ZAID is written as ASCII bytes and padded to 16 bytes with whitespace.
//        - Atomic mass fraction is written as an f64.
//        - kT is written as an f64.
//    - IZAW array
//        - 16 pairs of i64 / f64 values
//    - NXS array
//        - x16 usize
//    - JXS array
//        - x32 usize
//    - XXS array
//        - Variable size depending on the file
//        - During the creation of a PACE file, entries in the XXS array are checked to see if they
//          are integers or floats. If they are integers, they are stored as i64s, and if they are
//          floats, they are stored as f64s. However, all of these stored data values are written
//          as raw bytes in the file in f64 format. Logic elsewhere in this crate will convert to
//          the appropriate type when reading the data from the file.

//=====================================================================
// Memory-mapped file for the PACE binary format.
// All major sections of the file (the arrays) are available as slices.
// Much of the work performed by this crate is to execute the proper
// zero-copy conversions to appropriate types from the raw bytes in
// these slices.
//=====================================================================
pub struct PaceMmap ( memmap2::Mmap );

impl PaceMmap {
    // Take a pre-existing PACE file and map it into memory.
    pub fn from_PACE<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("Failed to open PACE file: {:?}", path.as_ref()))?;

        let mmap = unsafe { MmapOptions::new().map(&file) }
            .with_context(|| format!("Failed memory map PACE file: {:?}", path.as_ref()))?;
        Ok(Self(mmap))
    }

    // Pull the bytes corresponding to the header
    pub fn header_bytes(&self) -> &[u8] {
        &self.0[0..48]
    }

    // Pull the bytes corresponding to the IZAW array
    pub fn izaw_bytes(&self) -> &[u8] {
        &self.0[48..304]
    }

    // Pull the NXS array
    pub fn nxs_array(&self) -> &[usize] {
        // A JXS array consists of 16 integers
        let nxs_array = &self.0[304..432];
        // Zero-copy Conversion to usize
        unsafe { 
            std::slice::from_raw_parts(nxs_array.as_ptr() as *const usize, nxs_array.len() / 8)
        }
    }

    // Pull the JXS array
    pub fn jxs_array(&self) -> &[usize] {
        // A JXS array consists of 32 integers.
        let jxs_array = &self.0[432..688];
        // Zero-copy conversion to usize
        unsafe { 
            std::slice::from_raw_parts(jxs_array.as_ptr() as *const usize, jxs_array.len() / 8)
        }
    }
    
    // Pull the XXS array, interpreted as f64
    pub fn xxs_array(&self) -> &[f64] {
        let xxs_array_bytes = &self.0[688..];
        // Zero-copy conversion to f64
        unsafe {
            std::slice::from_raw_parts(xxs_array_bytes.as_ptr() as *const f64, xxs_array_bytes.len() / 8)
        }
    }
}


// Parse a line of the ASCII ACE file into tokens.
// This function is unsafe because it assumes that the input line is
// well-formed and contains valid ASCII characters. It does not perform
// any UTF-8 validation, as the ACE file format is expected to be
// ASCII-only.
#[inline(always)]
unsafe fn parse_tokens_from_line(line: &str) -> Vec<&str> {
    // Determine the number of 20-character regions based on the line length
    let bytes = line.as_bytes();
    let num_tokens = bytes.len() / 20;
    let mut tokens = Vec::with_capacity(num_tokens);

    // Loop over the tokens in the line
    for i in 0..num_tokens {
        let start = i * 20;

        // Manually find first non-space character
        let mut trim_start = 0;
        while trim_start < 20 && bytes[start + trim_start] == b' ' {
            trim_start += 1;
        }

        // SAFETY: Assume valid ASCII (not UTF-8 checked, we are parsing an ASCII file after all)
        unsafe {
            let token_ptr = bytes.as_ptr().add(start + trim_start);
            let token_len = 20 - trim_start;
            let token = std::str::from_utf8_unchecked(std::slice::from_raw_parts(token_ptr, token_len));
            tokens.push(token);
        }
    }
    tokens
}


// This function converts an ASCII ACE file into a PACE binary file.
pub fn convert_ACE_to_PACE<P: AsRef<Path>>(input_path: P) -> Result<String> {
    // Open ASCII ACE file
    let input_file = File::open(input_path.as_ref())?;
    let mut reader = BufReader::new(input_file);

    // Parse the header using the existing `from_ACE` method
    let header = Header::from_ACE(&mut reader)
        .with_context(|| format!("Failed to read header from ASCII ACE file {} while trying to convert to PACE file", input_path.as_ref().display()))?;

    // Set the PACE file name to the SZAID if it is available. Otherwise, set it to the ZAID.
    let output_filename = if let Some(ref val) = header.szaid {
        format!("{}.pace", val)
    } else {
        format!("{}.pace", header.zaid)
    };
    let output_filename = Path::new(&output_filename);
    let output_path = input_path.as_ref()
        .parent()
        .unwrap()
        .join(output_filename);

    // Create output file for writing (mutable from the start)
    let output_file = File::create(output_path.clone())?;
    let output_file = Mutex::new(output_file);

    // Write the header information
    {
        let mut output_file = output_file.lock().unwrap();
        match header.szaid {
            Some(ref val) => {
                let padding_length = 16 - val.len();
                output_file.write_all(val.as_bytes())?;
                output_file.write_all(&vec![b' '; padding_length])?;
            },
            None => {
                output_file.write_all(&vec![b' '; 16])?;
            }
        }

        let padding_length = 16 - header.zaid.len();
        output_file.write_all(header.zaid.as_bytes())?;
        output_file.write_all(&vec![b' '; padding_length])?;

        output_file.write_all(&header.atomic_mass_fraction.to_ne_bytes())?;
        output_file.write_all(&header.kT.to_ne_bytes())?;
    }

    // Annoyingly, the IXS, NXS, and JXS arrays have different line lengths than the XXS array.
    // To get around this we will read the next 10 lines of the file separately and parse them.
    const IZAW_NXS_JXS_LENGTH: usize = 10;
    let izaw_nxs_jxs_lines = utils::read_lines(&mut reader, IZAW_NXS_JXS_LENGTH)?;
    for line in izaw_nxs_jxs_lines {
        // Split line into whitespace-separated tokens
        for token in line.split_whitespace() {
            // Try parsing as integer first
            if let Ok(integer) = token.parse::<i64>() {
                let mut output_file = output_file.lock().unwrap();
                output_file.write_all(&integer.to_ne_bytes())?;
            }
            // Then try parsing as float
            else if let Ok(float) = token.parse::<f64>() {
                let mut output_file = output_file.lock().unwrap();
                output_file.write_all(&float.to_ne_bytes())?;
            } else {
                return Err(anyhow::anyhow!(format!("Invalid token format: '{}'", token)));
            }
        }
    }

    // Process XXS array lines in parallel batches
    const BATCH_SIZE: usize = 1000;
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
    let mut byte_batches: Vec<(usize, Vec<u8>)> = lines
        .par_chunks(BATCH_SIZE)
        .enumerate()
        .filter_map(|(index, batch)| {
            let mut local_buffer = Vec::with_capacity(BATCH_SIZE * 32);
            for line in batch {
                for token in unsafe { parse_tokens_from_line(line) } {
                    if let Ok(integer) = token.parse::<i64>() {
                        local_buffer.extend_from_slice(&integer.to_ne_bytes());
                    } else if let Ok(float) = token.parse::<f64>() {
                        local_buffer.extend_from_slice(&float.to_ne_bytes());
                    } else {
                        panic!("Invalid token \"{}\" when trying to convert ASCII to binary", token); // Skip invalid tokens
                    }
                }
            }
            Some((index, local_buffer))
        })
        .collect();

    // Sort batches of parsed binary data by index to ensure correct order
    byte_batches.sort_by_key(|&(index, _)| index);
    // Write sorted results to the output file
    let mut final_buffer = Vec::new();
    for (_, byte_batch) in byte_batches {
        final_buffer.extend_from_slice(&byte_batch);
    }
    {
        let mut output_file = output_file.lock().unwrap();
        output_file.write_all(&final_buffer)?;
    }

    // Return the path to the PACE file
    Ok(output_path.to_string_lossy().into_owned())
}
