use std::{error::Error, fs::File, io::{BufRead, BufReader, Write}, path::Path};
use rayon::prelude::*;
use std::sync::Mutex;

use memmap2::MmapOptions;

use crate::ace::header::AceHeader;
use crate::ace::utils;

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

#[inline(always)]
pub unsafe fn parse_tokens_from_line(line: &str) -> Vec<&str> {
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

// This function converts an ASCII ACE file into a binary format
pub fn convert_ascii_to_binary<P: AsRef<Path>>(input_path: P) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Open input file for reading
    let input_file = File::open(input_path.as_ref())?;
    let mut reader = BufReader::new(input_file);

    // Parse the header using the existing `from_ascii_file` method
    let header = AceHeader::from_ascii_file(&mut reader).map_err(|e| format!("Header parse failed: {}", e))?;

    // Set the binary file name to the SZAID if it is available. Otherwise, set it to the ZAID.
    let output_filename = if let Some(ref val) = header.szaid {
        format!("binary_{}", val)
    } else {
        format!("binary_{}", header.zaid)
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
    let izaw_nxs_jxs_lines = utils::read_lines(&mut reader, IZAW_NXS_JXS_LENGTH)
        .map_err(|_| "Error pulling NXS and JXS while converting ASCII to binary")?;
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
                return Err(format!("Invalid number format: '{}'", token).into());
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

    // Sort results by index to ensure correct order
    byte_batches.sort_by_key(|&(index, _)| index);
    // Write sorted results to the output file
    let mut output_file = output_file.lock().map_err(|_| "Failed to lock output file")?;
    for (_, byte_batch) in byte_batches {
        output_file.write_all(&byte_batch)?;
    }

    Ok(output_path.to_string_lossy().into_owned())
}

pub struct AceBinaryMmap ( memmap2::Mmap );

impl AceBinaryMmap {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        // Open up the binary file
        let file = File::open(path).map_err(|e| format!("Error opening ACE binary file: {}", e))?;

        // Create a memory map of the binary file
        let mmap = unsafe { MmapOptions::new().map(&file) }?;
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
        // Zero-copy Conversion to usize
        unsafe { 
            std::slice::from_raw_parts(jxs_array.as_ptr() as *const usize, jxs_array.len() / 8)
        }
    }
    
    // Pull the XXS array, interpreted as f64
    pub fn xxs_array(&self) -> &[f64] {
        let xxs_array_bytes = &self.0[688..];
        // Zero-copy Conversion to f64
        unsafe {
            std::slice::from_raw_parts(xxs_array_bytes.as_ptr() as *const f64, xxs_array_bytes.len() / 8)
        }
    }
}
