

//=====================================================================
// Helper struct to represent a reaction.
//=====================================================================

use crate::api::CrossSection;

#[derive(Clone, Debug)]
pub struct Reaction {
    pub mt: usize,
    pub q: Option<f64>,
    pub cross_section: CrossSection,
}