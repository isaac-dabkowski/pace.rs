// Enum for possible interpolation schemes from ENDF standard
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq)]
pub enum InterpolationScheme {
    Histogram = 1,
    LinLin = 2,
    LinLog = 3,
    LogLin = 4,
    LogLog = 5,
    Gamow = 6,
}

impl From<usize> for InterpolationScheme {
    fn from(value: usize) -> Self {
        match value {
            1 => InterpolationScheme::Histogram,
            2 => InterpolationScheme::LinLin,
            3 => InterpolationScheme::LinLog,
            4 => InterpolationScheme::LogLin,
            5 => InterpolationScheme::LogLog,
            6 => InterpolationScheme::Gamow,
            _ => panic!("Invalid interpolation scheme"),
        }
    }
}