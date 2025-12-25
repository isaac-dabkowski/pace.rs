//=====================================================================
// Helper macro: in tests, time a parse and print; in other builds, just
// evaluate the expression without any timing or println noise.
//=====================================================================
#[cfg(test)]
#[macro_export]
macro_rules! time_it {
    ($label:expr, $expr:expr) => {{
        let start = Instant::now();
        let result = $expr;
        println!("⚛️  {}  ⚛️ : {} μs", $label, start.elapsed().as_micros());
        result
    }};
}

#[cfg(not(test))]
#[macro_export]
macro_rules! time_it {
    ($label:expr, $expr:expr) => {{ $expr }};
}