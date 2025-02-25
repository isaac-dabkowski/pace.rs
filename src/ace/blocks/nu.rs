use crate::ace::arrays::{NxsArray, JxsArray};
use crate::ace::blocks::{DataBlockType, InterpolationTable};

// NU may be given in one of two forms: polynomial or tabulated
#[derive(Debug, Clone)]
pub enum NuFormulation {
    Polynomial(PolynomialNu),
    Tabulated(TabulatedNu),
}

impl NuFormulation {
    pub fn evaluate(&self, energy: f64) -> f64 {
        match self {
            NuFormulation::Polynomial(nu) => nu.evaluate(energy),
            NuFormulation::Tabulated(nu) => nu.evaluate(energy),
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
    pub fn evaluate(&self, energy: f64) -> f64 {
        let mut nu = 0.0;
        for (i, coef) in self.coefficients.iter().enumerate() {
            nu += coef * energy.powi(i as i32);
        }
        nu
    }
}

// Polynomial formulation for NU
#[derive(Debug, Clone)]
pub struct TabulatedNu {
    pub table: InterpolationTable
}

impl TabulatedNu {
    // Evaluate the tabulated nu at an energy (given in MeV)
    pub fn evaluate(&self, energy: f64) -> f64 {
        self.table.interpolate(energy).unwrap()
    }
}

#[derive(Debug, Clone, Default)]
pub struct NU {
    pub prompt: Option<NuFormulation>,
    pub total: Option<NuFormulation>,
}

impl NU {
    pub fn process(data: &[f64], jxs_array: &JxsArray) -> Self {
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
        } else if jxs_array.get(&DataBlockType::DNU) != 0 {
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

    pub fn pull_from_xxs_array<'a>(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64] {
        // Block start index (binary XXS is zero indexed for speed)
        let block_start = jxs_array.get(&DataBlockType::NU) - 1;

        // Check if we have prompt and total or just one of the two
        let prompt_and_or_total_flag = xxs_array[block_start].to_bits() as isize;
        let first_nu_length = prompt_and_or_total_flag.unsigned_abs() + 1;
        let mut block_length = first_nu_length;
        // We have both blocks, so we need to check the length of the second block
        if prompt_and_or_total_flag < 0 {
            // Jump to start of total nu and check if it is polynomial or tabulated
            let total_nu_poly_or_tabulated =  xxs_array[block_start + block_length].to_bits() as usize;
            let total_nu_start = block_start + block_length + 1;
            // We have a polynomial formulation for total nu
            if total_nu_poly_or_tabulated == 1 {
                block_length += 2 + xxs_array[total_nu_start].to_bits() as usize;
            // We have a tabulated formulation for total nu
            } else if total_nu_poly_or_tabulated == 2 {
                block_length += 1 + InterpolationTable::get_table_length(total_nu_start, xxs_array);
            } else {
                panic!("Unknown total nu formulation");
            }
        }

        // Avoid issues if this is the last block in the file
        let mut block_end = block_start + block_length;
        if block_end == xxs_array.len() + 1 {
            block_end -= 1;
        }
        // Return the block
        &xxs_array[block_start..block_end]
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ace::utils::get_parsed_test_file;

    #[tokio::test]
    async fn test_nu_parsing() {
        let parsed_ace = get_parsed_test_file().await;

        // Check contents
        let nu = parsed_ace.data_blocks.NU.unwrap();
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
        assert_eq!(prompt.evaluate(1.0), 3.3);
        assert_eq!(prompt.evaluate(1.5), 5.35);
        assert_eq!(prompt.evaluate(2.0), 8.0);

        // Check total nu
        let total = match nu.total.unwrap() {
            NuFormulation::Tabulated(total) => total,
            _ => panic!("This should be tabulated")
        };
        assert_eq!(total.table.len(), 2);
        assert_eq!(total.evaluate(1e-11), 1.0);
        assert_eq!(total.evaluate(1.0), 2.0);
        assert_eq!(total.evaluate(10.0), 3.0);
        assert_eq!(total.evaluate(1e-5), 1.0);
        assert_eq!(total.evaluate(5.5), 2.5);
    }
}