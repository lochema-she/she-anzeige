use super::config::ChairliftConfig;

#[derive(Debug, Clone, Copy)]
pub struct RopePosition {
    pulses: u64,
    rope_length_pulses: u64,
}

impl RopePosition {
    pub fn new(config: &ChairliftConfig) -> Self {
        Self {
            pulses: 0,
            rope_length_pulses: config.total_rope_pulses(),
        }
    }

    pub fn pulses(&self) -> u64 {
        self.pulses
    }

    pub fn rope_length_pulses(&self) -> u64 {
        self.rope_length_pulses
    }

    pub fn meters(&self, pulses_per_meter: u64) -> f64 {
        self.pulses as f64 / pulses_per_meter as f64
    }

    pub fn add_pulses(&mut self, pulses: u64) {
        self.pulses = (self.pulses + pulses) % self.rope_length_pulses;
    }

    pub fn set_pulses(&mut self, pulses: u64) {
        self.pulses = pulses % self.rope_length_pulses;
    }

    pub fn add_one_pulse(&mut self) {
        self.add_pulses(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ChairliftConfig {
        ChairliftConfig {
            valley_to_mountain_pulses: 58_000,
            mountain_to_valley_pulses: 58_000,
            mountain_station_pulses: 2_900,
            valley_station_pulses: 2_900,
            pulses_per_meter: 29,
        }
    }

    #[test]
    fn rope_position_counts_pulses() {
        let config = test_config();
        let mut position = RopePosition::new(&config);

        position.add_pulses(1000);

        assert_eq!(position.pulses(), 1000);
    }

    #[test]
    fn rope_position_wraps_at_end_of_rope() {
        let config = test_config();
        let mut position = RopePosition::new(&config);

        position.set_pulses(config.total_rope_pulses() - 10);
        position.add_pulses(20);

        assert_eq!(position.pulses(), 10);
    }
}
