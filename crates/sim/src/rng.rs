use serde::{Deserialize, Serialize};

/// PCG32 — small, fast, seedable. Its whole state is in the save file, so a
/// reload replays identically and any bug is reproducible from a save.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pcg32 {
    pub state: u64,
    pub inc: u64,
}

impl Pcg32 {
    pub fn new(seed: u64) -> Self {
        let mut r = Self {
            state: 0,
            inc: (seed << 1) | 1,
        };
        r.next_u32();
        r.state = r.state.wrapping_add(seed);
        r.next_u32();
        r
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform in [0, 1).
    pub fn f64(&mut self) -> f64 {
        (self.next_u32() as f64) / (u32::MAX as f64 + 1.0)
    }

    /// Approximately normal, mean 0, sd 1 (sum of uniforms).
    pub fn normal(&mut self) -> f64 {
        let a = self.f64();
        let b = self.f64();
        let c = self.f64();
        ((a + b + c) - 1.5) * 2.0
    }
}
