use crate::{MachineHardware, MachineNew};

use tokio::sync::mpsc;

use super::ChairliftMachine;
use super::config::ChairliftConfig;

fn create_chairs(config: &ChairliftConfig) -> Vec<super::chair::Chair> {
    (1..=113)
        .map(|id| {
            super::chair::Chair::new(
                id,
                super::position::RopePosition::new(config),
            )
        })
        .collect()
}

impl MachineNew for ChairliftMachine {
    fn new(hw: MachineHardware) -> Result<Self, anyhow::Error> {
        let (sender, receiver) = mpsc::channel(16);

        let config = ChairliftConfig::default();

        Ok(Self {
            machine_identification_unique: hw.identification,
            sender,
            receiver,
            last_update: std::time::Instant::now(),
            rope_position: super::position::RopePosition::new(&config),
            chairs: create_chairs(&config),
        })
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
            assert_eq!(
                chair.state,
                super::super::chair::ChairState::NotActive
            );
        }
    }
}
