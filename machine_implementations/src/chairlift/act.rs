use std::time::{Duration, Instant};

use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_430::Wago750_430;
use qitech_lib::ethercat_hal::io::digital_input::DigitalInputDevice;
use qitech_lib::machines::{Machine, MachineDataRegistry, MachineError, MachineIdentificationUnique};

use crate::{MachineApi, QiTechMachine};

use super::ChairliftMachine;

impl ChairliftMachine {
    /// Port on the WAGO 750-430 the encoder's single pulse channel is wired to
    /// (Port1 == index 0).
    const ENCODER_PORT: usize = 0;

    fn read_encoder_input(&self) -> bool {
        let coupler = self.wago_750_354.borrow();
        coupler
            .slot_devices
            .first()
            .and_then(|slot| slot.as_ref())
            .and_then(|device| device.as_any().downcast_ref::<Wago750_430>())
            .and_then(|di| di.get_input(Self::ENCODER_PORT).ok())
            .unwrap_or(false)
    }

    /// Samples the encoder's digital input and counts a rope pulse on every
    /// rising edge. Single-channel wiring: direction is not detected, the rope
    /// is always assumed to move forward.
    fn poll_encoder(&mut self) {
        let current = self.read_encoder_input();
        if current && !self.last_encoder_input {
            self.process_encoder_pulse();
        }
        self.last_encoder_input = current;
    }
}

impl Machine for ChairliftMachine {
    fn act(&mut self, _registry: Option<&mut MachineDataRegistry>) -> Result<(), MachineError> {
        while let Ok(msg) = self.receiver.try_recv() {
            self.act_machine_message(msg);
        }

        self.poll_encoder();

        let now = Instant::now();
        if now.duration_since(self.last_state_emit) > Duration::from_secs_f64(1.0 / 30.0) {
            self.emit_state();
            self.last_state_emit = now;
        }

        Ok(())
    }

    fn react(&mut self, _registry: &MachineDataRegistry) {}

    fn get_identification(&self) -> MachineIdentificationUnique {
        self.machine_identification_unique
    }
}

impl QiTechMachine for ChairliftMachine {}
