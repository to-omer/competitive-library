use super::Xorshift;

#[derive(Debug)]
pub struct SimuratedAnnealing {
    pub iter_count: usize,
    pub now: std::time::Instant,
    pub time: f64,
    pub temperture: f64,
    pub log_table: Vec<f64>,
    pub rand: Xorshift,

    pub is_maximize: bool,
    pub start_temp: f64,
    pub end_temp: f64,
    pub time_limit: f64,
    pub update_interval: usize,
}
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
            temperture: 3e3,
            log_table,
            rand: Xorshift::new_with_seed(Self::SEED),
            is_maximize: true,
            start_temp: 3e3,
            end_temp: 1e-8,
            time_limit: 1.99,
            update_interval: 0xff,
        }
    }
}
impl SimuratedAnnealing {
    pub const LOG_TABLE_SIZE: usize = 0x10000;
    pub const SEED: u64 = 0xbeef_cafe;

    pub fn new() -> Self {
        Default::default()
    }
    pub fn minimize(mut self) -> Self {
        self.is_maximize = false;
        self
    }
    pub fn set_start_temp(mut self, start_temp: f64) -> Self {
        assert_eq!(self.iter_count, 0);
        self.start_temp = start_temp;
        self.temperture = start_temp;
        self
    }
    pub fn set_end_temp(mut self, end_temp: f64) -> Self {
        self.end_temp = end_temp;
        self
    }
    pub fn set_time_limit(mut self, time_limit: f64) -> Self {
        self.time_limit = time_limit;
        self
    }
    pub fn set_update_interval(mut self, update_interval: usize) -> Self {
        assert!(update_interval > 0);
        self.update_interval = update_interval;
        self
    }
    pub fn is_accepted(&mut self, current_score: f64, next_score: f64) -> bool {
        let diff = if self.is_maximize {
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
        if self.iter_count % self.update_interval == 0 {
            self.time = self.now.elapsed().as_secs_f64();
            let temp_ratio = (self.end_temp - self.start_temp) / self.time_limit;
            self.temperture = self.start_temp + temp_ratio * self.time;
            self.time >= self.time_limit
        } else {
            false
        }
    }
}
