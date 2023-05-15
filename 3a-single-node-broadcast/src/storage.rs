use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Topology(pub(crate) HashMap<String, Vec<String>>);

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Messages(pub(crate) Vec<u64>);

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Storage {
    pub(crate) messages: Messages,
    pub(crate) topology: Topology,
}

impl Storage {
    pub(crate) fn new() -> Storage {
        Storage::default()
    }

    pub(crate) fn add_message(&mut self, message: u64) {
        self.messages.0.push(message);
    }

    pub(crate) fn get_messages(&mut self) -> Vec<u64> {
        self.messages.0.to_owned()
    }

    pub(crate) fn init_topology(&mut self, topology: HashMap<String, Vec<String>>) {
        self.topology.0 = topology;
    }
}
