use std::sync::Arc;

use control_core::socketio::{
    event::{Event, GenericEvent},
    namespace::{
        CacheFn, CacheableEvents, Namespace, NamespaceCacheingLogic, cache_first_and_last_event,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{MachineApi, MachineMessage, MachineValues};

use super::{ChairliftMachine, chair::ChairState};

#[derive(Serialize, Debug, Clone)]
pub struct ChairEvent {
    pub id: u16,
    pub status: ChairState,
    pub position_pulses: u64,
    pub position_meters: f64,
}

#[derive(Serialize, Debug, Clone)]
pub struct StateEvent {
    pub rope_position_pulses: u64,
    pub rope_length_pulses: u64,
    pub rope_position_meters: f64,
    // Layout of the loop, so the frontend can draw it to scale instead of assuming
    // a symmetric track. Mirrors `ChairliftConfig`.
    pub valley_to_mountain_pulses: u64,
    pub mountain_to_valley_pulses: u64,
    pub mountain_station_pulses: u64,
    pub valley_station_pulses: u64,
    pub chairs: Vec<ChairEvent>,
}

impl StateEvent {
    pub fn build(&self) -> Event<Self> {
        Event::new("StateEvent", self.clone())
    }
}

pub enum ChairliftMachineEvents {
    State(Event<StateEvent>),
}

impl CacheableEvents<ChairliftMachineEvents> for ChairliftMachineEvents {
    fn event_value(&self) -> GenericEvent {
        match self {
            ChairliftMachineEvents::State(event) => event.clone().into(),
        }
    }

    fn event_cache_fn(&self) -> CacheFn {
        cache_first_and_last_event()
    }
}

#[derive(Debug, Clone)]
pub struct ChairliftMachineNamespace {
    pub namespace: Option<Namespace>,
}

impl NamespaceCacheingLogic<ChairliftMachineEvents> for ChairliftMachineNamespace {
    fn emit(&mut self, events: ChairliftMachineEvents) {
        let event = Arc::new(events.event_value());
        let buffer_fn = events.event_cache_fn();
        if let Some(ns) = &mut self.namespace {
            ns.emit(event, &buffer_fn);
        }
    }
}

#[derive(Deserialize, Serialize)]
pub enum Mutation {
    /// Manually mark the next waiting chair as having left the station and
    /// attach it to the rope at the current rope position. Stands in for a real
    /// departure sensor, which this installation doesn't have (yet).
    Departure,
}

impl MachineApi for ChairliftMachine {
    fn act_machine_message(&mut self, msg: MachineMessage) {
        match msg {
            MachineMessage::SubscribeNamespace(namespace) => {
                self.namespace.namespace = Some(namespace);
                self.emit_state();
            }
            MachineMessage::UnsubscribeNamespace => self.namespace.namespace = None,
            MachineMessage::HttpApiJsonRequest(value) => {
                let _res = self.api_mutate(value);
            }
            MachineMessage::RequestValues(sender) => {
                sender
                    .send(MachineValues {
                        state: serde_json::to_value(self.get_state())
                            .expect("Failed to serialize state"),
                        live_values: serde_json::Value::Null,
                    })
                    .expect("Failed to send values");
            }
        }
    }

    fn get_api_sender(&self) -> tokio::sync::mpsc::Sender<MachineMessage> {
        self.sender.clone()
    }

    fn api_mutate(&mut self, value: Value) -> Result<(), anyhow::Error> {
        let mutation: Mutation = serde_json::from_value(value)?;
        match mutation {
            Mutation::Departure => {
                self.departure();
            }
        }
        self.emit_state();
        Ok(())
    }

    fn api_event_namespace(&mut self) -> Option<Namespace> {
        self.namespace.namespace.clone()
    }
}
