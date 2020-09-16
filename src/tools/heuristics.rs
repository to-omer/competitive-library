use crate::tools::Xorshift;

#[cargo_snippet::snippet("SimuratedAnnealing")]
#[derive(Debug)]
pub struct SimuratedAnnealing {
    pub iter_count: usize,
    pub now: std::time::Instant,
    pub time: f64,
    pub temperture: f64,
    pub log_table: Vec<f64>,
    pub rand: Xorshift,
}
#[cargo_snippet::snippet("SimuratedAnnealing")]
impl Default for SimuratedAnnealing {
    fn default() -> Self {
        let now = std::time::Instant::now();
        let log_table = (0..Self::LOG_TABLE_SIZE)
            .map(|i| ((i * 2 + 1) as f64 / (Self::LOG_TABLE_SIZE * 2) as f64).ln())
            .collect();
        Self {
            iter_count: 0,
            now,
            time: 0.,
            temperture: Self::START_TEMP,
            log_table,
            rand: Xorshift::new(Self::SEED),
        }
    }
}
#[cargo_snippet::snippet("SimuratedAnnealing")]
impl SimuratedAnnealing {
    pub const IS_MAXIMIZE: bool = true;
    pub const START_TEMP: f64 = 3e3;
    pub const END_TEMP: f64 = 1e-8;
    pub const TEMP_RATIO: f64 = (Self::END_TEMP - Self::START_TEMP) / Self::TIME_LIMIT;
    pub const TIME_LIMIT: f64 = 1.99;
    pub const LOG_TABLE_SIZE: usize = 0x10000;
    pub const UPDATE_INTERVAL: usize = 0xff;
    pub const SEED: u64 = 0xbeefcafe;

    pub fn is_accepted(&mut self, current_score: f64, next_score: f64) -> bool {
        let diff = if Self::IS_MAXIMIZE {
            next_score - current_score
        } else {
            current_score - next_score
        };
        diff >= 0.
            || diff
                > self.log_table[self.rand.rand(Self::LOG_TABLE_SIZE as u64) as usize]
                    * self.temperture
    }
    pub fn is_end(&mut self) -> bool {
        self.iter_count += 1;
        if self.iter_count & Self::UPDATE_INTERVAL == 0 {
            self.time = self.now.elapsed().as_secs_f64();
            self.temperture = Self::START_TEMP + Self::TEMP_RATIO * self.time;
            self.time >= Self::TIME_LIMIT
        } else {
            false
        }
    }
}
