#![allow(clippy::await_holding_lock, dead_code)]

use std::io::{BufReader, BufRead};
use std::fs::File;
use anyhow::Result;

//====================================================================
// Assorted helper functions.
//====================================================================

// Read a specified number of lines into a BufReader
#[inline]
pub fn read_lines(reader: &mut BufReader<File>, num_lines: usize) -> Result<Vec<String>> {
    reader.lines()
        .take(num_lines)
        .map(|line| line.map_err(anyhow::Error::from))
        .collect::<Result<Vec<_>>>()
}

// Provided a temperature in MeV, convert to K
#[inline]
pub fn compute_temperature_from_kT(kT: f64) -> f64 {
    kT * 1e6 / 8.617333262e-5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_temperature_from_kT() {
        let kT = 8.617333262e-8;
        let expected_temperature = 1000.0; // Kelvin
        assert!((compute_temperature_from_kT(kT) - expected_temperature).abs() < 1e-9);
    }
}