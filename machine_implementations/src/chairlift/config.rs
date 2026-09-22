#[derive(Debug, Clone, Copy)]
pub struct ChairliftConfig {
    pub valley_to_mountain_pulses: u64,
    pub mountain_to_valley_pulses: u64,
    pub mountain_station_pulses: u64,
    pub valley_station_pulses: u64,
}

impl ChairliftConfig {
    pub fn total_rope_pulses(&self) -> u64 {
        self.valley_to_mountain_pulses + self.mountain_to_valley_pulses
    }
}

impl Default for ChairliftConfig {
    fn default() -> Self {
        Self {
            valley_to_mountain_pulses: 58_000,
            mountain_to_valley_pulses: 58_000,
            mountain_station_pulses: 2_900,
            valley_station_pulses: 2_900,
        }
    }
}
