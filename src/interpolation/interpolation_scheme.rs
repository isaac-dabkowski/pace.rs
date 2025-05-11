//=====================================================================
// Enum for possible interpolation schemes from ENDF standard.
//=====================================================================
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

impl std::fmt::Display for InterpolationScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpolationScheme::Histogram => write!(f, "Histogram"),
            InterpolationScheme::LinLin => write!(f, "LinLin"),
            InterpolationScheme::LinLog => write!(f, "LinLog"),
            InterpolationScheme::LogLin => write!(f, "LogLin"),
            InterpolationScheme::LogLog => write!(f, "LogLog"),
            InterpolationScheme::Gamow => write!(f, "Gamow"),
        }
    }
}