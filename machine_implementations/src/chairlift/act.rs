use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_430::Wago750_430;
use qitech_lib::ethercat_hal::io::digital_input::DigitalInputDevice;
use qitech_lib::machines::{
    Machine, MachineDataRegistry, MachineError, MachineIdentificationUnique,
};

use crate::{MachineApi, QiTechMachine};

use super::ChairliftMachine;

// TEMPORARY: heartbeat counter for periodic raw-status debug prints below.
// Remove together with the debug prints once the encoder is confirmed working.
static POLL_COUNT: AtomicU32 = AtomicU32::new(0);

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

    /// TEMPORARY: dumps the coupler's reported input size and all 8 raw DI
    /// ports, to see whether the software is receiving any input data at all
    /// for this slave and whether the signal shows up on a different port
    /// than expected. Remove once the encoder is confirmed working.
    fn debug_dump_raw_state(&self) {
        let coupler = self.wago_750_354.borrow();
        let module = coupler
            .slot_devices
            .first()
            .and_then(|slot| slot.as_ref())
            .and_then(|device| device.as_any().downcast_ref::<Wago750_430>());

        match module {
            Some(di) => {
                let ports: Vec<bool> = (0..di.get_port_count())
                    .map(|p| di.get_input(p).unwrap_or(false))
                    .collect();
                println!(
                    "[chairlift] heartbeat: coupler input_len={} bytes, DI ports={:?}",
                    coupler.input_len(),
                    ports
                );
            }
            None => {
                println!(
                    "[chairlift] heartbeat: coupler input_len={} bytes, slot 0 is not a Wago750_430 (or empty)",
                    coupler.input_len()
                );
            }
        }
    }

    /// Samples the encoder's digital input and counts a rope pulse on every
    /// rising edge. Single-channel wiring: direction is not detected, the rope
    /// is always assumed to move forward.
    fn poll_encoder(&mut self) {
        let current = self.read_encoder_input();
        // TEMPORARY: prints every time the sampled input changes, to check
        // whether the software actually sees the physical signal toggle.
        // Remove once the encoder wiring/mapping is confirmed working.
        if current != self.last_encoder_input {
            println!(
                "[chairlift] encoder DI (Port1, index {}) changed to {current}",
                Self::ENCODER_PORT
            );
        }
        if current && !self.last_encoder_input {
            self.process_encoder_pulse();
        }
        self.last_encoder_input = current;

        // TEMPORARY: print a full raw status snapshot roughly once a second
        // (act() runs at the ~1kHz EtherCAT cycle), regardless of whether
        // anything changed. Remove together with the above.
        if POLL_COUNT.fetch_add(1, Ordering::Relaxed) % 1000 == 0 {
            self.debug_dump_raw_state();
        }
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
