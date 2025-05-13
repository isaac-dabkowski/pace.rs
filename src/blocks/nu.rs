use anyhow::Result;

use crate::arrays::Arrays;
use crate::interpolation::{InterpolationTable, InterpolationError};
use crate::blocks::BlockType;
use crate::blocks::block_traits::{get_block_start, block_range_to_slice, PullFromXXS, Process};

//=====================================================================
// NU data block
//
// Contains information on the number of neutrons released per fission,
// for both total and (sometimes) prompt neutrons.
//=====================================================================
#[derive(Debug, Clone, Default)]
pub struct NU {
    pub prompt: Option<NuFormulation>,
    pub total: Option<NuFormulation>,
}

impl<'a> PullFromXXS<'a> for NU {
    fn pull_from_xxs_array(arrays: &'a Arrays) -> Option<&'a [f64]> {
        // We expect NU if JXS(2) != 0
        let is_fissile = arrays.jxs.get(&BlockType::NU) != 0;

        // Validate that the block is there and get the start index
        let block_start = get_block_start(
            &BlockType::NU,
            arrays,
            is_fissile,
            "NU is expected if JXS(2) != 0, but NU was not found.".to_string(),
        )?;

        // Calculate the block length, see the NU description in the ACE spec
        // Check if we have prompt and total or just one of the two
        let prompt_and_or_total_flag = arrays.xxs[block_start].to_bits() as isize;
        let first_nu_length = prompt_and_or_total_flag.unsigned_abs() + 1;
        let mut block_length = first_nu_length;
        // We have both blocks, so we need to check the length of the second block
        if prompt_and_or_total_flag < 0 {
            // Jump to start of total nu and check if it is polynomial or tabulated
            let total_nu_poly_or_tabulated =  arrays.xxs[block_start + block_length].to_bits() as usize;
            let total_nu_start = block_start + block_length + 1;
            // We have a polynomial formulation for total nu
            if total_nu_poly_or_tabulated == 1 {
                block_length += 2 + arrays.xxs[total_nu_start].to_bits() as usize;
            // We have a tabulated formulation for total nu
            } else if total_nu_poly_or_tabulated == 2 {
                block_length += 1 + InterpolationTable::get_table_length(total_nu_start, arrays.xxs);
            } else {
                panic!("Unknown total nu formulation, expected 1 or 2, got {}", total_nu_poly_or_tabulated);
            }
        }

        // Return the block's raw data as a slice
        Some(block_range_to_slice(block_start, block_length, arrays))
    }
}

impl<'a> Process<'a> for NU {
    type Dependencies = ();

    fn process(data: &[f64], arrays: &Arrays, _dependencies: ()) -> Self {
        // Grab first nu data
        let prompt_and_or_total_flag = data[0].to_bits() as isize;
        let first_nu_length = prompt_and_or_total_flag.unsigned_abs();
        let first_nu_data = &data[1..first_nu_length + 1];

        let prompt_or_total_nu = match first_nu_data[0].to_bits() as usize {
            1 => NuFormulation::Polynomial(PolynomialNu {
                coefficients: first_nu_data[2..].to_vec()
            }),
            2 => NuFormulation::Tabulated(TabulatedNu {
                table: InterpolationTable::process(&first_nu_data[1..])
            }),
            _ => panic!("Unknown prompt/total nu formulation")
        };

        // We have both blocks
        if prompt_and_or_total_flag < 0 {
            let second_nu_data = &data[first_nu_length + 1..];
            let total_nu = match second_nu_data[0].to_bits() as usize {
                1 => NuFormulation::Polynomial(PolynomialNu {
                    coefficients: second_nu_data[2..].to_vec()
                }),
                2 => NuFormulation::Tabulated(TabulatedNu {
                    table: InterpolationTable::process(&second_nu_data[1..])
                }),
                _ => panic!("Unknown total nu formulation")
            };
            NU {
                prompt: Some(prompt_or_total_nu),
                total: Some(total_nu)
            }
        // We do not have both blocks
        } else if arrays.jxs.get(&BlockType::DNU) != 0 {
            NU {
                prompt: Some(prompt_or_total_nu),
                total: None
            }
        } else {
            NU {
                prompt: None,
                total: Some(prompt_or_total_nu)
            }
        }
    }
}

impl std::fmt::Display for NU {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut desc = String::new();
        if self.prompt.is_some() {
            desc.push_str("Prompt ");
        }
        if self.total.is_some() {
            desc.push_str("Total ");
        }
        write!(f, "NU( {})", desc)
    }
}

//=====================================================================
// NU may be given in one of two forms: polynomial or tabulated
//=====================================================================
#[derive(Debug, Clone)]
pub enum NuFormulation {
    Polynomial(PolynomialNu),
    Tabulated(TabulatedNu),
}

impl NuFormulation {
    pub fn evaluate(&self, energy: f64) -> Result<f64> {
        match self {
            NuFormulation::Polynomial(nu) => nu.evaluate(energy),
            NuFormulation::Tabulated(nu) => nu.evaluate(energy).map_err(anyhow::Error::from),
        }
    }
}

// Polynomial formulation for NU
#[derive(Debug, Clone)]
pub struct PolynomialNu {
    pub coefficients: Vec<f64>
}

impl PolynomialNu {
    // Evaluate the polynomial at an energy (given in MeV)
    pub fn evaluate(&self, energy: f64) -> Result<f64> {
        let mut nu = 0.0;
        for (i, coef) in self.coefficients.iter().enumerate() {
            nu += coef * energy.powi(i as i32);
        }
        Ok(nu)
    }
}

// Polynomial formulation for NU
#[derive(Debug, Clone)]
pub struct TabulatedNu {
    pub table: InterpolationTable
}

impl TabulatedNu {
    // Evaluate the tabulated nu at an energy (given in MeV)
    pub fn evaluate(&self, energy: f64) -> Result<f64, InterpolationError> {
        self.table.interpolate(energy)
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_nu_parsing() {
        let parsed_pace = get_parsed_test_file().await;

        // Check contents
        let nu = parsed_pace.data_blocks.NU.unwrap();
        assert!(nu.prompt.is_some());
        assert!(nu.total.is_some());
        assert!(matches!(nu.prompt.as_ref().unwrap(), NuFormulation::Polynomial(_)));
        assert!(matches!(nu.total.as_ref().unwrap(), NuFormulation::Tabulated(_)));

        // Check prompt nu
        let prompt = match nu.prompt.unwrap() {
            NuFormulation::Polynomial(poly) => poly,
            _ => panic!("This should be a polynomial")
        };
        assert_eq!(prompt.coefficients.len(), 3);
        assert_eq!(prompt.coefficients, vec![1.0, 1.1, 1.2]);
        assert_eq!(prompt.evaluate(1.0).unwrap(), 3.3);
        assert_eq!(prompt.evaluate(1.5).unwrap(), 5.35);
        assert_eq!(prompt.evaluate(2.0).unwrap(), 8.0);

        // Check total nu
        let total = match nu.total.unwrap() {
            NuFormulation::Tabulated(total) => total,
            _ => panic!("This should be tabulated")
        };
        assert_eq!(total.table.len(), 2);
        assert_eq!(total.evaluate(1e-11).unwrap(), 1.0);
        assert_eq!(total.evaluate(1.0).unwrap(), 2.0);
        assert_eq!(total.evaluate(10.0).unwrap(), 3.0);
        assert_eq!(total.evaluate(1e-5).unwrap(), 1.0);
        assert_eq!(total.evaluate(5.5).unwrap(), 2.5);
    }
}