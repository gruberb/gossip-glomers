use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Neighbours(pub(crate) HashSet<String>);

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Messages(pub(crate) HashSet<u64>);

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Storage {
    pub(crate) messages: Messages,
    pub(crate) received_messages: HashMap<String, Messages>,
    pub(crate) sent_messages: HashMap<String, Messages>,
    pub(crate) neighbours: Neighbours,
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

    pub(crate) fn add_to_sent_messages(&mut self, messages: Vec<u64>, node: String) {
        if self.sent_messages.contains_key(&node) {
            self.sent_messages
                .get_mut(&node)
                .unwrap()
                .0
                .extend(messages);
        } else {
            self.sent_messages
                .insert(node, Messages(messages.iter().cloned().collect()));
        }
    }

    pub(crate) fn init_topology(&mut self, node_id: String, nodes: &Vec<String>) {
        let i = nodes.iter().position(|x| *x == node_id).unwrap();

        let left_neighbor = nodes[(i + nodes.len() - 1) % nodes.len()].clone();
        let right_neighbor = nodes[(i + 1) % nodes.len()].clone();

        let mut rng = thread_rng();
        let selections: Vec<String> = nodes.choose_multiple(&mut rng, 2).cloned().collect();

        self.neighbours.0.extend(selections);
        self.neighbours.0.insert(left_neighbor);
        self.neighbours.0.insert(right_neighbor);
    }

    pub(crate) fn get_neighbours(&self) -> HashSet<String> {
        self.neighbours.0.clone()
    }
}
