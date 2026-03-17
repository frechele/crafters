use rand_mt::Mt;

const DOUBLE_UNIT: f64 = 1.0 / 9_007_199_254_740_992.0;

#[derive(Clone, Debug)]
pub(crate) struct PyRandom {
    mt: Mt,
}

impl PyRandom {
    pub(crate) fn new(seed: u32) -> Self {
        Self { mt: Mt::new(seed) }
    }

    pub(crate) fn next_u32(&mut self) -> u32 {
        self.mt.next_u32()
    }

    pub(crate) fn uniform(&mut self) -> f64 {
        let a = (self.mt.next_u32() >> 5) as u64;
        let b = (self.mt.next_u32() >> 6) as u64;
        ((a << 26) + b) as f64 * DOUBLE_UNIT
    }

    pub(crate) fn randint(&mut self, upper_exclusive: u32) -> u32 {
        assert!(upper_exclusive > 0, "randint upper bound must be positive");
        self.interval(upper_exclusive - 1)
    }

    fn interval(&mut self, max: u32) -> u32 {
        let mut mask = max;
        mask |= mask >> 1;
        mask |= mask >> 2;
        mask |= mask >> 4;
        mask |= mask >> 8;
        mask |= mask >> 16;
        loop {
            let value = self.mt.next_u32() & mask;
            if value <= max {
                return value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PyRandom;

    #[test]
    fn uniform_matches_numpy_randomstate() {
        let mut rng = PyRandom::new(123);
        let expected = [
            0.6964691855978616,
            0.28613933495037946,
            0.2268514535642031,
            0.5513147690828912,
            0.7194689697855631,
        ];
        for expected in expected {
            let actual = rng.uniform();
            assert!((actual - expected).abs() < 1e-16);
        }
    }

    #[test]
    fn randint_matches_numpy_randomstate() {
        let mut rng = PyRandom::new(123);
        let expected = [843_828_734, 914_636_141, 1_228_959_102, 1_840_268_610, 974_319_580];
        for expected in expected {
            assert_eq!(rng.randint((1_u32 << 31) - 1), expected);
        }
    }
}
