pub trait Process<'a> {
    type Dependencies;

    fn process(data: &[f64], dependencies: Self::Dependencies) -> Self
    where
        Self: Sized;
}