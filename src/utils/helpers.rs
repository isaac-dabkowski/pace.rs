// Helper functions

// Accept "300K", "900K", return Ok(300_u64), Ok(900_u64)
pub fn parse_temp_k(name: &str) -> Option<u64> {
    let s = name.strip_suffix('K')?;
    s.parse::<u64>().ok()
}
