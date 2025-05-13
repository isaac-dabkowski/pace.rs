use rand::Rng;
use rand::RngCore;

pub struct MockRng {
    values: Vec<f64>,
    index: usize,
}

impl MockRng {
    pub fn new(values: Vec<f64>) -> Self {
        Self { values, index: 0 }
    }
}

impl RngCore for MockRng {
    fn next_u32(&mut self) -> u32 {
        unimplemented!("MockRNG does not support next_u32")
    }

    fn next_u64(&mut self) -> u64 {
        if self.index >= self.values.len() {
            panic!("MockRNG: Ran out of values to return");
        }
        let bits = self.values[self.index].to_bits();
        self.index += 1;
        bits
    }

    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        unimplemented!("MockRNG does not support fill_bytes")
    }
}

// No need to implement Rng manually since it's automatically implemented for any type that implements RngCore