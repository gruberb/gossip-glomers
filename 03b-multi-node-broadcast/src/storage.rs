use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Topology(pub(crate) HashMap<String, Vec<String>>);

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Messages(pub(crate) HashSet<u64>);

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Storage {
    pub(crate) messages: Messages,
    pub(crate) received_messages: HashMap<String, Messages>,
    pub(crate) sent_messages: HashMap<String, Messages>,
    pub(crate) topology: Topology,
}

impl Storage {
    pub(crate) fn add_message(&mut self, message: u64, node: String) {
        self.messages.0.insert(message);

        if self.received_messages.contains_key(&node) {
            self.received_messages
                .get_mut(&node)
                .unwrap()
                .0
                .insert(message);
        } else {
            let mut v = Messages::default();
            v.0.insert(message);
            self.received_messages.insert(node, v);
        }
    }

    pub(crate) fn get_messages(&mut self) -> Vec<u64> {
        self.messages.0.clone().into_iter().collect()
    }

    pub(crate) fn get_messages_for_node(&self, node: String) -> Vec<u64> {
        let received: Vec<u64> = self
            .received_messages
            .iter()
            .filter(|(key, _)| *key == &node)
            .flat_map(|(_, Messages(value))| value)
            .cloned()
            .collect();

        let sent: Vec<u64> = self
            .sent_messages
            .iter()
            .filter(|(key, _)| *key == &node)
            .flat_map(|(_, Messages(value))| value)
            .cloned()
            .collect();

        self.messages
            .0
            .iter()
            .filter(|m| !received.contains(m) && !sent.contains(m))
            .cloned()
            .collect()
    }

    pub(crate) fn add_to_sent_messages(&mut self, messages: HashSet<u64>, node: String) {
        if self.sent_messages.contains_key(&node) {
            self.sent_messages
                .get_mut(&node)
                .unwrap()
                .0
                .extend(messages);
        } else {
            self.sent_messages.insert(node, Messages(messages));
        }
    }

    pub(crate) fn init_topology(&mut self, topology: HashMap<String, Vec<String>>) {
        self.topology.0 = topology;
    }

    pub(crate) fn get_neighbours(&self, node_id: &str) -> Vec<String> {
        self.topology.0.get(node_id).unwrap().to_owned()
    }
}
