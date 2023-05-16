use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Messages(pub(crate) HashSet<u64>);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Storage {
    pub(crate) messages: Messages,
    pub(crate) received_messages: HashMap<String, Messages>,
    pub(crate) sent_messages: HashMap<String, Messages>,
    pub(crate) retry: HashMap<String, u8>,
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

    pub(crate) fn get_retries(&self, node: String) -> u8 {
        match self.retry.get(&node) {
            Some(count) => *count,
            None => 0,
        }
    }

    pub(crate) fn increase_or_insert(&mut self, node: String) {
        let count = self.retry.entry(node).or_insert(0);
        *count += 1;
    }

    pub(crate) fn decrease_or_remove(&mut self, node: String) {
        match self.retry.get_mut(&node) {
            Some(count) => {
                *count -= 1;
                if *count == 0 {
                    self.retry.remove(&node);
                }
            }
            None => (),
        }
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
}
