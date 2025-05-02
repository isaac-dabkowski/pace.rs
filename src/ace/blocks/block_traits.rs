use crate::ace::arrays::{JxsArray, NxsArray};

pub trait Process<'a> {
    type Dependencies;

    fn process(data: &[f64], dependencies: Self::Dependencies) -> Self
    where
        Self: Sized;
}

pub trait PullFromXXS<'a> {
    fn pull_from_xxs_array(nxs_array: &NxsArray, jxs_array: &JxsArray, xxs_array: &'a [f64]) -> &'a [f64]
    where
        Self: Sized;
}