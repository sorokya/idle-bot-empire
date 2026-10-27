use serde::{Deserialize, Serialize};

/// PCG32 — small, fast, seedable. Its whole state is in the save file, so a
/// reload replays identically and any bug is reproducible from a save.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Pcg32 {
    pub(crate) state: u64,
    pub(crate) inc: u64,
}

impl Pcg32 {
    pub(crate) fn new(seed: u64) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (seed << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rotation = (old >> 59) as u32;
        xorshifted.rotate_right(rotation)
    }

    /// Uniform in `[0, 1)`.
    pub(crate) fn f64(&mut self) -> f64 {
        f64::from(self.next_u32()) / (f64::from(u32::MAX) + 1.0)
    }

    /// Approximately normal, mean 0, standard deviation 1 (sum of uniforms).
    pub(crate) fn normal(&mut self) -> f64 {
        let first = self.f64();
        let second = self.f64();
        let third = self.f64();
        ((first + second + third) - 1.5) * 2.0
    }
}
