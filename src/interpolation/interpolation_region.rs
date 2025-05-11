use crate::interpolation::InterpolationScheme;

//=====================================================================
// X/Y pair for interpolation.
//=====================================================================
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct XY {
    pub x: f64,
    pub y: f64,
}

impl Eq for XY {}

//=====================================================================
// Interpolation region. This contains a set of X/Y pairs and the
// interpolation scheme to be used in the region.
//=====================================================================
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq)]
pub struct InterpolationRegion {
    pub data: Vec<XY>,
    pub interpolation_scheme: InterpolationScheme,
}

impl InterpolationRegion {
    pub fn from_x_and_y(x: Vec<f64>, y: Vec<f64>, interpolation_scheme: InterpolationScheme) -> Self {
        // Ensure that the x and y vectors are of the same length
        if x.len() != y.len() {
            panic!("InterpolationRegion: x ({}) and y ({}) vectors must be of the same length", x.len(), y.len());
        }

        // Zip the x and y vectors together into a vector of XY structs
        let data = x.into_iter().zip(y.into_iter()).map(|(x, y)| XY { x, y }).collect();

        Self { data, interpolation_scheme }
    }
}