use serde::Serialize;

use super::position::RopePosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ChairState {
    NotActive,
    OnRope,
    MountainStation,
    ValleyStation,
}

#[derive(Debug, Clone, Copy)]
pub struct Chair {
    pub id: u16,
    pub start_position: RopePosition,
    pub state: ChairState,
}

impl Chair {
    pub fn new(id: u16, start_position: RopePosition) -> Self {
        Self {
            id,
            start_position,
            state: ChairState::NotActive,
        }
    }

    pub fn activate(&mut self, start_position: RopePosition) {
        self.start_position = start_position;
        self.state = ChairState::OnRope;
    }

    pub fn is_active(&self) -> bool {
        self.state != ChairState::NotActive
    }

    pub fn position(&self, rope_position: RopePosition) -> u64 {
        let current = rope_position.pulses();
        let start = self.start_position.pulses();

        (current + self.start_position.rope_length_pulses() - start)
            % self.start_position.rope_length_pulses()
    }

    /// Which physical zone of the loop this chair currently occupies, for display
    /// purposes. Doesn't mutate `state`, so it never affects `is_active`/`activate`.
    pub fn display_status(
        &self,
        rope_position: RopePosition,
        config: &super::config::ChairliftConfig,
    ) -> ChairState {
        if !self.is_active() {
            return ChairState::NotActive;
        }
        config.zone_at(self.position(rope_position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::ChairliftConfig;

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
    fn chair_moves_with_rope_position() {
        let config = test_config();

        let start = RopePosition::new(&config);
        let chair = Chair::new(1, start);

        let mut current = RopePosition::new(&config);
        current.add_pulses(1000);

        assert_eq!(chair.position(current), 1000);
    }

    #[test]
    fn chair_position_wraps_correctly() {
        let config = test_config();

        let mut start = RopePosition::new(&config);
        start.set_pulses(115_000);

        let chair = Chair::new(1, start);

        let mut current = RopePosition::new(&config);
        current.set_pulses(1000);

        assert_eq!(chair.position(current), 2000);
    }

    #[test]
    fn chair_can_be_activated() {
        let config = test_config();

        let start = RopePosition::new(&config);
        let mut chair = Chair::new(1, start);

        assert_eq!(chair.state, ChairState::NotActive);
        assert!(!chair.is_active());

        let mut activation_position = RopePosition::new(&config);
        activation_position.set_pulses(5000);

        chair.activate(activation_position);

        assert_eq!(chair.state, ChairState::OnRope);
        assert!(chair.is_active());
        assert_eq!(chair.start_position.pulses(), 5000);
    }

    #[test]
    fn display_status_is_not_active_before_activation() {
        let config = test_config();
        let start = RopePosition::new(&config);
        let chair = Chair::new(1, start);

        assert_eq!(chair.display_status(start, &config), ChairState::NotActive);
    }

    #[test]
    fn display_status_reflects_current_zone_once_active() {
        let config = test_config();

        let start = RopePosition::new(&config);
        let mut chair = Chair::new(1, start);
        chair.activate(start);

        // Right after departure the chair is still within the valley station zone.
        assert_eq!(
            chair.display_status(start, &config),
            ChairState::ValleyStation
        );

        // Out on the open rope, away from both stations.
        let mut mid_rope = RopePosition::new(&config);
        mid_rope.set_pulses(30_000);
        assert_eq!(chair.display_status(mid_rope, &config), ChairState::OnRope);

        // Arriving at the mountain station.
        let mut mountain = RopePosition::new(&config);
        mountain.set_pulses(config.valley_to_mountain_pulses);
        assert_eq!(
            chair.display_status(mountain, &config),
            ChairState::MountainStation
        );
    }
}
