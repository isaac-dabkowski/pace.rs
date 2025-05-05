// External codes which utilize the sampling functions in this library should ALWAYS provide a
// random number between 0.0 and 1.0, inclusive. This struct is used to enforce that the value, but
// this is not a runtime check. It is up to the user to ensure that the value is in the range [0.0, 1.0].
//
// During debug builds, a panic will occur if the value is outside of the range [0.0, 1.0].
//
// This decision was made to avoid the overhead of runtime checks in release builds, while allowing
// the end user to set their own RNG protocol.
#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct UnitF64(pub f64);

impl UnitF64 {
    #[inline(always)]
    pub fn new_unchecked(val: f64) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&val),
            "UnitF64 must be in [0.0, 1.0], got {}",
            val
        );
        UnitF64(val)
    }
}