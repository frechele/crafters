#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvConfig {
    pub area: [usize; 2],
    pub view: [usize; 2],
    pub size: [usize; 2],
    pub reward: bool,
    pub length: Option<u32>,
    pub seed: u64,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            area: [64, 64],
            view: [9, 9],
            size: [64, 64],
            reward: true,
            length: Some(10_000),
            seed: 0,
        }
    }
}
