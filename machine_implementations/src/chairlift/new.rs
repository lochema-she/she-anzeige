use anyhow::{Error, Result};
use qitech_lib::ethercat_hal::devices::EthercatDevice;
use qitech_lib::ethercat_hal::devices::wago_modules::wago_750_354::Wago750_354;

use crate::{MachineHardware, MachineNew};

use super::ChairliftMachine;
use super::api::ChairliftMachineNamespace;
use super::config::ChairliftConfig;

fn create_chairs(config: &ChairliftConfig) -> Vec<super::chair::Chair> {
    (1..=113)
        .map(|id| super::chair::Chair::new(id, super::position::RopePosition::new(config)))
        .collect()
}

impl MachineNew for ChairliftMachine {
    fn new(hw: MachineHardware) -> Result<Self, Error> {
        // Role 0: WAGO 750-354 bus coupler carrying the 750-430 DI module.
        let (wago_750_354, coupler_addr) =
            hw.try_get_ethercat_device_and_addr_by_role::<Wago750_354>(0)?;

        let interface = hw.ethercat_interface.clone().ok_or_else(|| {
            anyhow::anyhow!("ChairliftMachine: no EtherCAT interface was supplied")
        })?;

        // Discover and initialize the coupler's attached I/O modules (the 750-430
        // DI module the encoder is wired to) via SDO.
        let modules = Wago750_354::initialize_modules(interface.clone(), coupler_addr)?;
        {
            let mut coupler = wago_750_354.borrow_mut();
            for module in modules {
                coupler.set_module(module);
            }
            coupler.init_slot_modules(interface, coupler_addr);

            // TEMPORARY: diagnose why encoder pulses don't reach the software
            // even though the module's own input LED lights up. Remove once
            // the encoder is confirmed working.
            println!(
                "[chairlift] coupler input_len={} bytes, output_len={} bytes, module_count={}, dev_count={}",
                coupler.input_len(),
                coupler.output_len(),
                coupler.module_count,
                coupler.dev_count,
            );
            for (i, slot) in coupler.slots.iter().enumerate() {
                if let Some(module) = slot {
                    println!(
                        "[chairlift] slot {i}: name={} has_tx={} has_rx={} tx_offset={} rx_offset={}",
                        module.name,
                        module.has_tx,
                        module.has_rx,
                        module.tx_offset,
                        module.rx_offset
                    );
                }
            }
        }

        let (sender, receiver) = tokio::sync::mpsc::channel(16);

        let config = ChairliftConfig::default();

        let mut machine = Self {
            machine_identification_unique: hw.identification,
            sender,
            receiver,
            wago_750_354,
            last_encoder_input: false,
            namespace: ChairliftMachineNamespace { namespace: None },
            last_state_emit: std::time::Instant::now(),
            rope_position: super::position::RopePosition::new(&config),
            chairs: create_chairs(&config),
            config,
        };

        machine.emit_state();

        Ok(machine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_113_chairs() {
        let config = ChairliftConfig::default();
        let chairs = create_chairs(&config);

        assert_eq!(chairs.len(), 113);
        assert_eq!(chairs.first().unwrap().id, 1);
        assert_eq!(chairs.last().unwrap().id, 113);

        for chair in &chairs {
            assert_eq!(chair.state, super::super::chair::ChairState::NotActive);
        }
    }
}
