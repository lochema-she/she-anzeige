use std::{cell::RefCell, rc::Rc, time::Instant};

use control_core::socketio::namespace::NamespaceCacheingLogic;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_354::Wago750_354;
use qitech_lib::machines::{MachineIdentification, MachineIdentificationUnique};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{MachineMessage, VENDOR_QITECH};

pub mod act;
pub mod api;
pub mod chair;
pub mod config;
pub mod new;
pub mod position;

pub struct ChairliftMachine {
    pub machine_identification_unique: MachineIdentificationUnique,

    sender: Sender<MachineMessage>,
    receiver: Receiver<MachineMessage>,

    /// WAGO 750-354 bus coupler carrying the 750-430 DI module the encoder is wired to.
    wago_750_354: Rc<RefCell<Wago750_354>>,
    /// Last sampled encoder input value, used for software rising-edge detection.
    last_encoder_input: bool,

    namespace: api::ChairliftMachineNamespace,
    last_state_emit: Instant,

    config: config::ChairliftConfig,
    pub rope_position: position::RopePosition,
    pub chairs: Vec<chair::Chair>,
}

fn activate_next_chair(
    chairs: &mut [chair::Chair],
    position: position::RopePosition,
) -> Option<u16> {
    if let Some(chair) = chairs.iter_mut().find(|chair| !chair.is_active()) {
        let id = chair.id;
        chair.activate(position);
        Some(id)
    } else {
        None
    }
}

impl ChairliftMachine {
    pub const MACHINE_IDENTIFICATION: MachineIdentification = MachineIdentification {
        vendor: VENDOR_QITECH,
        machine: 0x0050,
    };

    pub fn chair(&self, id: u16) -> Option<&chair::Chair> {
        self.chairs.iter().find(|chair| chair.id == id)
    }

    pub fn chair_mut(&mut self, id: u16) -> Option<&mut chair::Chair> {
        self.chairs.iter_mut().find(|chair| chair.id == id)
    }

    pub fn chair_count(&self) -> usize {
        self.chairs.len()
    }

    pub fn process_encoder_pulse(&mut self) {
        self.rope_position.add_one_pulse();
    }

    pub fn chair_position(&self, id: u16) -> Option<u64> {
        let chair = self.chair(id)?;

        if !chair.is_active() {
            return None;
        }

        Some(chair.position(self.rope_position))
    }

    pub fn activate_next_chair(&mut self) -> Option<u16> {
        activate_next_chair(&mut self.chairs, self.rope_position)
    }

    pub fn departure(&mut self) -> Option<u16> {
        self.activate_next_chair()
    }

    pub fn get_state(&self) -> api::StateEvent {
        let pulses_per_meter = self.config.pulses_per_meter;

        api::StateEvent {
            rope_position_pulses: self.rope_position.pulses(),
            rope_length_pulses: self.rope_position.rope_length_pulses(),
            rope_position_meters: self.rope_position.meters(pulses_per_meter),
            valley_to_mountain_pulses: self.config.valley_to_mountain_pulses,
            mountain_to_valley_pulses: self.config.mountain_to_valley_pulses,
            mountain_station_pulses: self.config.mountain_station_pulses,
            valley_station_pulses: self.config.valley_station_pulses,
            chairs: self
                .chairs
                .iter()
                .map(|chair| {
                    let position_pulses = if chair.is_active() {
                        chair.position(self.rope_position)
                    } else {
                        0
                    };
                    api::ChairEvent {
                        id: chair.id,
                        status: chair.display_status(self.rope_position, &self.config),
                        position_pulses,
                        position_meters: position_pulses as f64 / pulses_per_meter as f64,
                    }
                })
                .collect(),
        }
    }

    pub fn emit_state(&mut self) {
        let event = self.get_state().build();
        self.namespace
            .emit(api::ChairliftMachineEvents::State(event));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_chairs() -> Vec<chair::Chair> {
        let config = config::ChairliftConfig::default();

        (1..=113)
            .map(|id| chair::Chair::new(id, position::RopePosition::new(&config)))
            .collect()
    }

    #[test]
    fn machine_has_113_chairs() {
        let chairs = test_chairs();

        assert_eq!(chairs.len(), 113);
    }

    #[test]
    fn activate_next_chair_activates_first_chair() {
        let mut chairs = test_chairs();
        let position = position::RopePosition::new(&config::ChairliftConfig::default());

        let id = activate_next_chair(&mut chairs, position);

        assert_eq!(id, Some(1));
        assert_eq!(chairs[0].state, chair::ChairState::OnRope);
        assert_eq!(chairs[0].start_position.pulses(), 0);
    }

    #[test]
    fn activate_next_chair_activates_second_chair() {
        let mut chairs = test_chairs();
        let config = config::ChairliftConfig::default();

        let first_position = position::RopePosition::new(&config);
        let first_id = activate_next_chair(&mut chairs, first_position);

        let mut second_position = position::RopePosition::new(&config);
        second_position.set_pulses(5000);

        let second_id = activate_next_chair(&mut chairs, second_position);

        assert_eq!(first_id, Some(1));
        assert_eq!(second_id, Some(2));

        assert_eq!(chairs[0].state, chair::ChairState::OnRope);
        assert_eq!(chairs[1].state, chair::ChairState::OnRope);

        assert_eq!(chairs[0].start_position.pulses(), 0);
        assert_eq!(chairs[1].start_position.pulses(), 5000);
    }

    #[test]
    fn encoder_pulse_moves_rope_position() {
        let config = config::ChairliftConfig::default();

        let mut position = position::RopePosition::new(&config);

        assert_eq!(position.pulses(), 0);

        position.add_one_pulse();

        assert_eq!(position.pulses(), 1);

        position.add_pulses(99);

        assert_eq!(position.pulses(), 100);
    }

    #[test]
    fn chair_position_follows_rope_position() {
        let config = config::ChairliftConfig::default();

        let mut chairs = test_chairs();

        let start_position = position::RopePosition::new(&config);

        let id = activate_next_chair(&mut chairs, start_position);

        assert_eq!(id, Some(1));

        let mut current_position = position::RopePosition::new(&config);
        current_position.add_pulses(1000);

        let chair = &chairs[0];

        assert_eq!(chair.position(current_position), 1000);
    }

    #[test]
    fn chair_position_follows_rope_position_across_wrap() {
        let config = config::ChairliftConfig::default();
        let mut chairs = test_chairs();

        let mut start_position = position::RopePosition::new(&config);
        start_position.set_pulses(config.total_rope_pulses() - 100);

        let id = activate_next_chair(&mut chairs, start_position);

        assert_eq!(id, Some(1));

        let mut current_position = position::RopePosition::new(&config);
        current_position.set_pulses(100);

        let chair = &chairs[0];

        assert_eq!(chair.position(current_position), 200);
    }

    #[test]
    fn departure_activates_next_chair_at_current_position() {
        let config = config::ChairliftConfig::default();
        let mut chairs = test_chairs();

        let mut current_position = position::RopePosition::new(&config);
        current_position.set_pulses(5000);

        let id = activate_next_chair(&mut chairs, current_position);

        assert_eq!(id, Some(1));
        assert_eq!(chairs[0].state, chair::ChairState::OnRope);
        assert_eq!(chairs[0].start_position.pulses(), 5000);
    }
}
