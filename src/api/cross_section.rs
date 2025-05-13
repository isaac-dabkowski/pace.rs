

use std::ops::{Deref, DerefMut};

use crate::interpolation::{InterpolationTable, InterpolationScheme};

//=====================================================================
// Helper struct to represent a cross section.
//=====================================================================
#[derive(Debug, Clone)]
pub struct CrossSection ( pub InterpolationTable );

impl Deref for CrossSection {
    type Target = InterpolationTable;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CrossSection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CrossSection {
    pub fn from_e_and_sigma(e: Vec<f64>, sigma: Vec<f64>) -> Self {
        Self ( InterpolationTable::from_x_and_y(e, sigma, InterpolationScheme::LinLin) )
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_abs_diff_eq;

    #[test]
    fn test_cross_section_creation() {
        let energies = vec![1.0, 2.0, 3.0];
        let sigmas = vec![0.1, 0.2, 0.3];
        let xs = CrossSection::from_e_and_sigma(energies.clone(), sigmas.clone());
    }

    #[test]
    fn test_cross_section_interpolation() {
        let xs = CrossSection::from_e_and_sigma(vec![1.0, 2.0, 3.0], vec![0.1, 0.2, 0.3]);
        
        assert_abs_diff_eq!(xs.interpolate(1.5).unwrap(), 0.15);
        assert_abs_diff_eq!(xs.interpolate(2.5).unwrap(), 0.25);
    }
}
