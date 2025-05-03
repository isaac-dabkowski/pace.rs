use std::ops::{Deref, DerefMut};
use std::error::Error;
use std::iter::zip;

// Enum for possible interpolation schemes from ENDF standard
#[derive(Debug, Clone, Copy)]
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

// X/Y pair for interpolation
#[derive(Debug, Clone)]
pub struct XY {
    pub x: f64,
    pub y: f64,
}

// Interpolation region
#[derive(Debug, Clone)]
pub struct InterpolationRegion {
    pub data: Vec<XY>,
    pub interpolation_scheme: InterpolationScheme,
}

// Struct for interpolation table data
#[derive(Debug, Clone, Default)]
pub struct InterpolationTable ( Vec<InterpolationRegion> );

impl Deref for InterpolationTable {
    type Target = Vec<InterpolationRegion>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for InterpolationTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl InterpolationTable {
    pub fn process(data: &[f64]) -> Self {
        // First, get the number of interpolation regions
        let num_interp_regions = data[0].to_bits() as usize;

        // If the number of regions is zero, this means we use linear-linear interpolation
        if num_interp_regions == 0 {
            let num_data_points = data[1].to_bits() as usize;
            let x_start = 2;
            let y_start = x_start + num_data_points;

            let data_points = (0..num_data_points).map(|idx| XY {
                x: data[x_start + idx],
                y: data[y_start + idx],
            });

            // Single region with linear-linear interpolation
            return InterpolationTable(vec![InterpolationRegion {
                data: data_points.collect(),
                interpolation_scheme: InterpolationScheme::LinLin,
            }]);
        }

        // We have a list of interpolation parameters and schemes
        // Split out raw data into interpolation bounds, regions, and xy data
        let bounds_start = 1;
        let schemes_start = bounds_start + num_interp_regions;
        let schemes_end = schemes_start + num_interp_regions;
        let num_data_points = data[schemes_end].to_bits() as usize;
        let x_start = schemes_end + 1;
        let y_start = x_start + num_data_points;

        // Bounds, convert to zero-indexed for sanity
        let bounds = std::iter::once(0)
            .chain(data[bounds_start..schemes_start].iter().map(|&val| val.to_bits() as usize - 1));

        // Schemes
        let schemes = data[schemes_start..schemes_end]
            .iter()
            .map(|&val| InterpolationScheme::from(val.to_bits() as usize));

        // Data points
        let data_points = zip(
            data[x_start..y_start].iter(), 
            data[y_start..].iter()).map(|(x, y)| XY {
                x: *x,
                y: *y,
                }
        );

        // Create interpolation regions
        let regions = bounds.clone().zip(bounds.skip(1)).zip(schemes).map(|((start, end), scheme)| {
            let region_data = data_points.clone().skip(start).take(end - start + 1);
            InterpolationRegion {
                data: region_data.collect(),
                interpolation_scheme: scheme,
            }
        });

        InterpolationTable(regions.collect())
    }

    pub fn get_table_length(table_start: usize, array_containing_table: &[f64]) -> usize {
        let mut table_length = 0;

        // First, get the number of interpolation regions
        let num_interp_regions = array_containing_table[table_start].to_bits() as usize;
        // If the number of regions is zero, this means we use linear-linear interpolation
        if num_interp_regions == 0 {
            let num_data_points_per_vec = array_containing_table[table_start + 1].to_bits() as usize;
            table_length += 2 + 2 * num_data_points_per_vec;
        } else {
            // We have a list of interpolation parameters and schemes
            table_length += 1 + 2 * num_interp_regions;
            let num_data_points_per_vec = array_containing_table[table_start + table_length].to_bits() as usize;
            table_length += 1 + 2 * num_data_points_per_vec;
        }
        table_length
    }

    // Interpolate a value from the table
    pub fn interpolate(&self, x_val: f64) -> Result<f64, Box<dyn Error>> {
        // Check if the table is valid
        if self.len() < 1 {
            return Err(From::from("Invalid interpolation table: empty"));
        }
        // Find the region that x_val falls into
        let region = self.iter().find(|region| {
            region.data[0].x <= x_val && x_val <= region.data.iter().last().unwrap().x
        }).ok_or_else(|| Box::<dyn Error>::from(format!("Interpolation region for x={} not found", x_val)))?;

        // Find the index of the bin that x_val falls into
        let idx = match region.data.binary_search_by(|xy| xy.x.partial_cmp(&x_val).unwrap()) {
            // We are exactly on a data point, exit early by returning the value
            Ok(idx) => return Ok(region.data[idx].y),
            // We are inside a bin
            Err(idx) => idx - 1,
        };

        // Get the start and end points of the bin
        let start = &region.data[idx];
        let end = region.data.get(idx + 1).unwrap();

        // Here are the values we need for interpolation
        let x0 = start.x;
        let x1 = end.x;
        let y0 = start.y;
        let y1 = end.y;

        // Perform the interpolation
        match &region.interpolation_scheme {
            InterpolationScheme::Histogram => Ok(y0),
            InterpolationScheme::LinLin => Ok(y0 + (y1 - y0) * (x_val - x0) / (x1 - x0)),
            InterpolationScheme::LinLog => Ok(y0 + (y1 - y0) * (x_val.log10() - x0.log10()) / (x1.log10() - x0.log10())),
            InterpolationScheme::LogLin => Ok(y0 * ((x_val - x0) * (y1 / y0).ln() / (x1 - x0)).exp()),
            InterpolationScheme::LogLog => Ok(y0 * ((x_val / x0).ln() * (y1 / y0).ln() / (x1 / x0).ln()).exp()),
            InterpolationScheme::Gamow => todo!("Gamow interpolation")
        }
    }
}
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_histogram_interpolation() {
        let table = InterpolationTable(vec![
            InterpolationRegion {
                data: vec![
                    XY { x: 1.0, y: 2.0 },
                    XY { x: 2.0, y: 4.0 },
                    XY { x: 3.0, y: 6.0 },
                ],
                interpolation_scheme: InterpolationScheme::Histogram,
            }
        ]);

        let result = table.interpolate(1.0).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(1.5).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(2.0).unwrap();
        assert_eq!(result, 4.0);
        let result = table.interpolate(2.1).unwrap();
        assert_eq!(result, 4.0);
        let result = table.interpolate(3.0).unwrap();
        assert_eq!(result, 6.0);
        let result = table.interpolate(3.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_linlin_interpolation() {
        let table = InterpolationTable(vec![
            InterpolationRegion {
                data: vec![
                    XY { x: 1.0, y: 2.0 },
                    XY { x: 2.0, y: 4.0 },
                    XY { x: 3.0, y: 6.0 },
                ],
                interpolation_scheme: InterpolationScheme::LinLin,
            }
        ]);

        let result = table.interpolate(1.0).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(1.5).unwrap();
        assert_eq!(result, 3.0);
        let result = table.interpolate(2.0).unwrap();
        assert_eq!(result, 4.0);
        let result = table.interpolate(2.5).unwrap();
        assert_eq!(result, 5.0);
        let result = table.interpolate(3.0).unwrap();
        assert_eq!(result, 6.0);
        let result = table.interpolate(3.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_linlog_interpolation() {
        let table = InterpolationTable(vec![
            InterpolationRegion {
                data: vec![
                    XY { x: 1.0, y: 2.0 },
                    XY { x: 2.0, y: 5.0 },
                    XY { x: 3.0, y: 10.0 },
                ],
                interpolation_scheme: InterpolationScheme::LinLog,
            }
        ]);

        let result = table.interpolate(1.0).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(1.5).unwrap();
        assert!((result - 3.754888).abs() < 1e-5);
        let result = table.interpolate(2.0).unwrap();
        assert_eq!(result, 5.0);
        let result = table.interpolate(2.5).unwrap();
        assert!((result - 7.751699).abs() < 1e-5);
        let result = table.interpolate(3.0).unwrap();
        assert_eq!(result, 10.0);
        let result = table.interpolate(3.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_loglin_interpolation() {
        let table = InterpolationTable(vec![
            InterpolationRegion {
                data: vec![
                    XY { x: 1.0, y: 2.0 },
                    XY { x: 2.0, y: 5.0 },
                    XY { x: 3.0, y: 10.0 },
                ],
                interpolation_scheme: InterpolationScheme::LogLin,
            }
        ]);

        let result = table.interpolate(1.0).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(1.5).unwrap();
        assert!((result - 3.162278).abs() < 1e-5);
        let result = table.interpolate(2.0).unwrap();
        assert_eq!(result, 5.0);
        let result = table.interpolate(2.5).unwrap();
        assert!((result - 7.071068).abs() < 1e-5);
        let result = table.interpolate(3.0).unwrap();
        assert_eq!(result, 10.0);
        let result = table.interpolate(3.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_loglog_interpolation() {
        let table = InterpolationTable(vec![
            InterpolationRegion {
                data: vec![
                    XY { x: 1.0, y: 2.0 },
                    XY { x: 2.0, y: 5.0 },
                    XY { x: 3.0, y: 10.0 },
                ],
                interpolation_scheme: InterpolationScheme::LogLog,
            }
        ]);

        let result = table.interpolate(1.0).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(1.5).unwrap();
        assert!((result - 3.418298).abs() < 1e-5);
        let result = table.interpolate(2.0).unwrap();
        assert_eq!(result, 5.0);
        let result = table.interpolate(2.5).unwrap();
        assert!((result - 7.322152).abs() < 1e-5);
        let result = table.interpolate(3.0).unwrap();
        assert_eq!(result, 10.0);
        let result = table.interpolate(3.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_interpolation_regions() {
        let table = InterpolationTable(vec![
            InterpolationRegion {
                data: vec![
                    XY { x: 1.0, y: 2.0 },
                    XY { x: 2.0, y: 5.0 },
                ],
                interpolation_scheme: InterpolationScheme::Histogram,
            },
            InterpolationRegion {
                data: vec![
                    XY { x: 2.0, y: 5.0 },
                    XY { x: 3.0, y: 10.0 },
                ],
                interpolation_scheme: InterpolationScheme::LinLin,
            },
            InterpolationRegion {
                data: vec![
                    XY { x: 3.0, y: 10.0 },
                    XY { x: 4.0, y: 5.0 },
                ],
                interpolation_scheme: InterpolationScheme::LinLog,
            },
            InterpolationRegion {
                data: vec![
                    XY { x: 4.0, y: 5.0 },
                    XY { x: 5.0, y: 2.0 },
                ],
                interpolation_scheme: InterpolationScheme::LogLin,
            },
            InterpolationRegion {
                data: vec![
                    XY { x: 5.0, y: 2.0 },
                    XY { x: 6.0, y: 100.0 },
                    XY { x: 7.0, y: 1.0 },
                ],
                interpolation_scheme: InterpolationScheme::LogLog,
            },
        ]);

        // Out of bounds
        let result = table.interpolate(0.5);
        assert!(result.is_err());
        // Histogram
        let result = table.interpolate(1.0).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(1.5).unwrap();
        assert_eq!(result, 2.0);
        // Lin-lin
        let result = table.interpolate(2.0).unwrap();
        assert_eq!(result, 5.0);
        let result = table.interpolate(2.5).unwrap();
        assert_eq!(result, 7.5);
        // Lin-log
        let result = table.interpolate(3.0).unwrap();
        assert_eq!(result, 10.0);
        let result = table.interpolate(3.5).unwrap();
        assert!((result - 7.320815).abs() < 1e-5);
        // Log-lin
        let result = table.interpolate(4.0).unwrap();
        assert_eq!(result, 5.0);
        let result = table.interpolate(4.5).unwrap();
        assert!((result - 3.162278).abs() < 1e-5);
        // Log-log
        let result = table.interpolate(5.0).unwrap();
        assert_eq!(result, 2.0);
        let result = table.interpolate(5.5).unwrap();
        assert!((result - 15.458998).abs() < 1e-5);
        let result = table.interpolate(6.0).unwrap();
        assert_eq!(result, 100.0);
        let result = table.interpolate(6.5).unwrap();
        assert!((result - 9.151672).abs() < 1e-5);
        let result = table.interpolate(7.0).unwrap();
        assert_eq!(result, 1.0);
        // Out of bounds
        let result = table.interpolate(7.1);
        assert!(result.is_err());
    }

    #[test]
    fn test_out_of_bounds_interpolation() {
        let table = InterpolationTable(vec![
            InterpolationRegion {
                data: vec![
                    XY { x: 1.0, y: 2.0 },
                    XY { x: 2.0, y: 3.0 },
                    XY { x: 3.0, y: 4.0 },
                ],
                interpolation_scheme: InterpolationScheme::Histogram,
            }
        ]);

        let result = table.interpolate(0.5);
        assert!(result.is_err());
    }

}
