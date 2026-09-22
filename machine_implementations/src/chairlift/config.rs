#[derive(Debug, Clone, Copy)]
pub struct ChairliftConfig {
    pub valley_to_mountain_pulses: u64,
    pub mountain_to_valley_pulses: u64,
    pub mountain_station_pulses: u64,
    pub valley_station_pulses: u64,
    /// Encoder resolution: pulses generated per meter of rope travel.
    /// Needs to be calibrated to the real installation.
    pub pulses_per_meter: u64,
}

impl ChairliftConfig {
    pub fn total_rope_pulses(&self) -> u64 {
        self.valley_to_mountain_pulses + self.mountain_to_valley_pulses
    }

    /// Classifies an absolute rope-pulse position into the zone a chair at that
    /// position currently occupies.
    ///
    /// Station zones are centered on the segment boundaries (0 / total for the
    /// valley station, `valley_to_mountain_pulses` for the mountain station)
    /// since a chair passes continuously through a station as it goes around the
    /// bullwheel, rather than the station adding extra length to the loop.
    pub fn zone_at(&self, pulses: u64) -> super::chair::ChairState {
        use super::chair::ChairState;

        let total = self.total_rope_pulses();
        if total == 0 {
            return ChairState::OnRope;
        }
        let p = pulses % total;

        let valley_half = self.valley_station_pulses / 2;
        if p < valley_half || p >= total - valley_half {
            return ChairState::ValleyStation;
        }

        let mountain_half = self.mountain_station_pulses / 2;
        let mountain_center = self.valley_to_mountain_pulses % total;
        let lower = mountain_center.saturating_sub(mountain_half);
        let upper = (mountain_center + mountain_half).min(total);
        if p >= lower && p < upper {
            return ChairState::MountainStation;
        }

        ChairState::OnRope
    }
}

impl Default for ChairliftConfig {
    fn default() -> Self {
        Self {
            valley_to_mountain_pulses: 58_000,
            mountain_to_valley_pulses: 58_000,
            mountain_station_pulses: 2_900,
            valley_station_pulses: 2_900,
            pulses_per_meter: 29,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::chair::ChairState;

    #[test]
    fn zone_at_classifies_valley_station_around_wrap_point() {
        let config = ChairliftConfig::default();

        assert_eq!(config.zone_at(0), ChairState::ValleyStation);
        assert_eq!(config.zone_at(1_449), ChairState::ValleyStation);
        assert_eq!(
            config.zone_at(config.total_rope_pulses() - 1),
            ChairState::ValleyStation
        );
    }

    #[test]
    fn zone_at_classifies_mountain_station_around_segment_boundary() {
        let config = ChairliftConfig::default();

        assert_eq!(
            config.zone_at(config.valley_to_mountain_pulses),
            ChairState::MountainStation
        );
        assert_eq!(
            config.zone_at(config.valley_to_mountain_pulses - 1_000),
            ChairState::MountainStation
        );
        assert_eq!(
            config.zone_at(config.valley_to_mountain_pulses + 1_000),
            ChairState::MountainStation
        );
    }

    #[test]
    fn zone_at_classifies_open_rope_between_stations() {
        let config = ChairliftConfig::default();

        assert_eq!(config.zone_at(1_450), ChairState::OnRope);
        assert_eq!(config.zone_at(30_000), ChairState::OnRope);
        assert_eq!(
            config.zone_at(config.total_rope_pulses() - 30_000),
            ChairState::OnRope
        );
    }

    #[test]
    fn zone_at_wraps_positions_beyond_total_length() {
        let config = ChairliftConfig::default();

        assert_eq!(
            config.zone_at(config.total_rope_pulses()),
            config.zone_at(0)
        );
    }
}
