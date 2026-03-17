const GRADIENTS3: [[f64; 3]; 24] = [
    [-11.0, 4.0, 4.0],
    [-4.0, 11.0, 4.0],
    [-4.0, 4.0, 11.0],
    [11.0, 4.0, 4.0],
    [4.0, 11.0, 4.0],
    [4.0, 4.0, 11.0],
    [-11.0, -4.0, 4.0],
    [-4.0, -11.0, 4.0],
    [-4.0, -4.0, 11.0],
    [11.0, -4.0, 4.0],
    [4.0, -11.0, 4.0],
    [4.0, -4.0, 11.0],
    [-11.0, 4.0, -4.0],
    [-4.0, 11.0, -4.0],
    [-4.0, 4.0, -11.0],
    [11.0, 4.0, -4.0],
    [4.0, 11.0, -4.0],
    [4.0, 4.0, -11.0],
    [-11.0, -4.0, -4.0],
    [-4.0, -11.0, -4.0],
    [-4.0, -4.0, -11.0],
    [11.0, -4.0, -4.0],
    [4.0, -11.0, -4.0],
    [4.0, -4.0, -11.0],
];

const STRETCH_CONSTANT3: f64 = -1.0 / 6.0;
const SQUISH_CONSTANT3: f64 = 1.0 / 3.0;
const NORM_CONSTANT3: f64 = 103.0;
const PERM_SIZE: usize = 256;
const SEED_MULTIPLIER: i64 = 6_364_136_223_846_793_005;
const SEED_INCREMENT: i64 = 1_442_695_040_888_963_407;

#[derive(Clone, Debug)]
pub(crate) struct OpenSimplexNoise {
    perm: [i64; PERM_SIZE],
}

impl OpenSimplexNoise {
    pub(crate) fn new(seed: i64) -> Self {
        let mut perm = [0; PERM_SIZE];
        let mut source = [0; PERM_SIZE];
        for (index, slot) in source.iter_mut().enumerate() {
            *slot = index as i64;
        }

        let mut state = seed;
        for _ in 0..3 {
            state = state
                .wrapping_mul(SEED_MULTIPLIER)
                .wrapping_add(SEED_INCREMENT);
        }

        for index in (0..PERM_SIZE).rev() {
            state = state
                .wrapping_mul(SEED_MULTIPLIER)
                .wrapping_add(SEED_INCREMENT);
            let choice =
                ((state as i128 + 31).rem_euclid((index + 1) as i128)) as usize;
            perm[index] = source[choice];
            source[choice] = source[index];
        }

        Self { perm }
    }

    pub(crate) fn noise3(&self, x: f64, y: f64, z: f64) -> f64 {
        let stretch_offset = (x + y + z) * STRETCH_CONSTANT3;
        let xs = x + stretch_offset;
        let ys = y + stretch_offset;
        let zs = z + stretch_offset;

        let base = [xs.floor() as i64, ys.floor() as i64, zs.floor() as i64];
        let squish_offset = (base[0] + base[1] + base[2]) as f64 * SQUISH_CONSTANT3;
        let origin = [
            x - (base[0] as f64 + squish_offset),
            y - (base[1] as f64 + squish_offset),
            z - (base[2] as f64 + squish_offset),
        ];
        let ins = [
            xs - base[0] as f64,
            ys - base[1] as f64,
            zs - base[2] as f64,
        ];
        let in_sum = ins[0] + ins[1] + ins[2];

        let value = if in_sum <= 1.0 {
            self.noise3_region_origin(base, origin, ins, in_sum)
        } else if in_sum >= 2.0 {
            self.noise3_region_far(base, origin, ins, in_sum)
        } else {
            self.noise3_region_middle(base, origin, ins)
        };

        value / NORM_CONSTANT3
    }

    fn noise3_region_origin(
        &self,
        base: [i64; 3],
        origin: [f64; 3],
        ins: [f64; 3],
        in_sum: f64,
    ) -> f64 {
        let mut a_score = ins[0];
        let mut a_point = 0x01i64;
        let mut b_score = ins[1];
        let mut b_point = 0x02i64;

        if a_score >= b_score && ins[2] > b_score {
            b_score = ins[2];
            b_point = 0x04;
        } else if a_score < b_score && ins[2] > a_score {
            a_score = ins[2];
            a_point = 0x04;
        }

        let wins = 1.0 - in_sum;
        let extra = if wins > a_score || wins > b_score {
            let closest = if b_score > a_score { b_point } else { a_point };
            match closest {
                0x01 => {
                    self.contribute(base, origin, [1.0, -1.0, 0.0])
                        + self.contribute(base, origin, [1.0, 0.0, -1.0])
                }
                0x02 => {
                    self.contribute(base, origin, [-1.0, 1.0, 0.0])
                        + self.contribute(base, origin, [0.0, 1.0, -1.0])
                }
                _ => {
                    self.contribute(base, origin, [-1.0, 0.0, 1.0])
                        + self.contribute(base, origin, [0.0, -1.0, 1.0])
                }
            }
        } else {
            let closest = a_point | b_point;
            match closest {
                0x03 => {
                    self.contribute(base, origin, [1.0, 1.0, 0.0])
                        + self.contribute(base, origin, [1.0, 1.0, -1.0])
                }
                0x05 => {
                    self.contribute(base, origin, [1.0, 0.0, 1.0])
                        + self.contribute(base, origin, [1.0, -1.0, 1.0])
                }
                _ => {
                    self.contribute(base, origin, [0.0, 1.0, 1.0])
                        + self.contribute(base, origin, [-1.0, 1.0, 1.0])
                }
            }
        };

        extra
            + self.contribute(base, origin, [0.0, 0.0, 0.0])
            + self.contribute(base, origin, [1.0, 0.0, 0.0])
            + self.contribute(base, origin, [0.0, 1.0, 0.0])
            + self.contribute(base, origin, [0.0, 0.0, 1.0])
    }

    fn noise3_region_far(
        &self,
        base: [i64; 3],
        origin: [f64; 3],
        ins: [f64; 3],
        in_sum: f64,
    ) -> f64 {
        let mut a_score = ins[0];
        let mut a_point = 0x06i64;
        let mut b_score = ins[1];
        let mut b_point = 0x05i64;

        if a_score <= b_score && ins[2] < b_score {
            b_score = ins[2];
            b_point = 0x03;
        } else if a_score > b_score && ins[2] < a_score {
            a_score = ins[2];
            a_point = 0x03;
        }

        let wins = 3.0 - in_sum;
        let extra = if wins < a_score || wins < b_score {
            let closest = if b_score < a_score { b_point } else { a_point };
            match closest {
                0x03 => {
                    self.contribute(base, origin, [2.0, 1.0, 0.0])
                        + self.contribute(base, origin, [1.0, 2.0, 0.0])
                }
                0x05 => {
                    self.contribute(base, origin, [2.0, 0.0, 1.0])
                        + self.contribute(base, origin, [1.0, 0.0, 2.0])
                }
                _ => {
                    self.contribute(base, origin, [0.0, 2.0, 1.0])
                        + self.contribute(base, origin, [0.0, 1.0, 2.0])
                }
            }
        } else {
            let closest = a_point & b_point;
            match closest {
                0x01 => {
                    self.contribute(base, origin, [1.0, 0.0, 0.0])
                        + self.contribute(base, origin, [2.0, 0.0, 0.0])
                }
                0x02 => {
                    self.contribute(base, origin, [0.0, 1.0, 0.0])
                        + self.contribute(base, origin, [0.0, 2.0, 0.0])
                }
                _ => {
                    self.contribute(base, origin, [0.0, 0.0, 1.0])
                        + self.contribute(base, origin, [0.0, 0.0, 2.0])
                }
            }
        };

        extra
            + self.contribute(base, origin, [1.0, 1.0, 0.0])
            + self.contribute(base, origin, [1.0, 0.0, 1.0])
            + self.contribute(base, origin, [0.0, 1.0, 1.0])
            + self.contribute(base, origin, [1.0, 1.0, 1.0])
    }

    fn noise3_region_middle(&self, base: [i64; 3], origin: [f64; 3], ins: [f64; 3]) -> f64 {
        let p1 = ins[0] + ins[1];
        let (a_score, mut a_point, mut a_is_further) = if p1 > 1.0 {
            (p1 - 1.0, 0x03i64, true)
        } else {
            (1.0 - p1, 0x04i64, false)
        };

        let p2 = ins[0] + ins[2];
        let (b_score, mut b_point, mut b_is_further) = if p2 > 1.0 {
            (p2 - 1.0, 0x05i64, true)
        } else {
            (1.0 - p2, 0x02i64, false)
        };

        let p3 = ins[1] + ins[2];
        if p3 > 1.0 {
            let score = p3 - 1.0;
            if a_score <= b_score && a_score < score {
                a_point = 0x06;
                a_is_further = true;
            } else if a_score > b_score && b_score < score {
                b_point = 0x06;
                b_is_further = true;
            }
        } else {
            let score = 1.0 - p3;
            if a_score <= b_score && a_score < score {
                a_point = 0x01;
                a_is_further = false;
            } else if a_score > b_score && b_score < score {
                b_point = 0x01;
                b_is_further = false;
            }
        }

        let extra = if a_is_further == b_is_further {
            if a_is_further {
                let shared = a_point & b_point;
                self.contribute(base, origin, [1.0, 1.0, 1.0])
                    + match shared {
                        0x01 => self.contribute(base, origin, [2.0, 0.0, 0.0]),
                        0x02 => self.contribute(base, origin, [0.0, 2.0, 0.0]),
                        _ => self.contribute(base, origin, [0.0, 0.0, 2.0]),
                    }
            } else {
                let omitted = a_point | b_point;
                self.contribute(base, origin, [0.0, 0.0, 0.0])
                    + match omitted {
                        0x03 => self.contribute(base, origin, [1.0, 1.0, -1.0]),
                        0x04 => self.contribute(base, origin, [1.0, -1.0, 1.0]),
                        _ => self.contribute(base, origin, [-1.0, 1.0, 1.0]),
                    }
            }
        } else {
            let (further, closer) = if a_is_further {
                (a_point, b_point)
            } else {
                (b_point, a_point)
            };
            let further_value = match further {
                0x03 => self.contribute(base, origin, [1.0, 1.0, -1.0]),
                0x05 => self.contribute(base, origin, [1.0, -1.0, 1.0]),
                _ => self.contribute(base, origin, [-1.0, 1.0, 1.0]),
            };
            let closer_value = match closer {
                0x01 => self.contribute(base, origin, [2.0, 0.0, 0.0]),
                0x02 => self.contribute(base, origin, [0.0, 2.0, 0.0]),
                _ => self.contribute(base, origin, [0.0, 0.0, 2.0]),
            };
            further_value + closer_value
        };

        extra
            + self.contribute(base, origin, [1.0, 0.0, 0.0])
            + self.contribute(base, origin, [0.0, 1.0, 0.0])
            + self.contribute(base, origin, [0.0, 0.0, 1.0])
            + self.contribute(base, origin, [1.0, 1.0, 0.0])
            + self.contribute(base, origin, [1.0, 0.0, 1.0])
            + self.contribute(base, origin, [0.0, 1.0, 1.0])
    }

    fn contribute(&self, base: [i64; 3], origin: [f64; 3], offset: [f64; 3]) -> f64 {
        let squish = SQUISH_CONSTANT3 * (offset[0] + offset[1] + offset[2]);
        let dx = origin[0] - offset[0] - squish;
        let dy = origin[1] - offset[1] - squish;
        let dz = origin[2] - offset[2] - squish;
        let attenuation = 2.0 - dx * dx - dy * dy - dz * dz;
        if attenuation <= 0.0 {
            return 0.0;
        }

        let attenuation_sq = attenuation * attenuation;
        let value = self.extrapolate(
            base[0] + offset[0] as i64,
            base[1] + offset[1] as i64,
            base[2] + offset[2] as i64,
            dx,
            dy,
            dz,
        );
        attenuation_sq * attenuation_sq * value
    }

    fn extrapolate(&self, xsb: i64, ysb: i64, zsb: i64, dx: f64, dy: f64, dz: f64) -> f64 {
        let index0 = ((self.perm[(xsb & 0xFF) as usize] + ysb) & 0xFF) as usize;
        let index1 = ((self.perm[index0] + zsb) & 0xFF) as usize;
        let gradient = GRADIENTS3[self.perm[index1] as usize % GRADIENTS3.len()];
        gradient[0] * dx + gradient[1] * dy + gradient[2] * dz
    }
}

#[cfg(test)]
mod tests {
    use super::OpenSimplexNoise;

    #[test]
    fn noise3_matches_python_opensimplex_reference_values() {
        let simplex = OpenSimplexNoise::new(123_456_789);
        let cases = [
            ((0.0, 0.0, 0.0), 2.1510994893306976e-67),
            ((1.25, -3.5, 8.0), 0.4174582150537894),
            ((2.0 / 3.0, 5.0 / 7.0, 6.0), -0.32957906882363436),
            ((10.1, 4.2, 7.0), -0.18410162621359225),
            ((-11.0, 2.5, 1.0), -0.16968168487055021),
        ];

        for ((x, y, z), expected) in cases {
            let actual = simplex.noise3(x, y, z);
            assert!(
                (actual - expected).abs() < 1e-12,
                "noise3 mismatch for ({x}, {y}, {z}): expected {expected}, got {actual}",
            );
        }
    }
}
