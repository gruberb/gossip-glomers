use serde::{Deserialize, Serialize};

use crate::message::{Body, Message};
use crate::storage::Storage;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct Node {
    pub(crate) id: String,
    pub(crate) availble_nodes: Vec<String>,
    pub(crate) storage: Storage,
}

impl Node {
    pub(crate) fn init(&mut self, message: Message) {
        match message.body {
            Body::Init {
                node_id, node_ids, ..
            } => {
                self.id = node_id.clone();
                self.availble_nodes = node_ids.clone();
            }
            _ => panic!("Invalid message type"),
        }
    }

    pub(crate) fn get_id(&self) -> String {
        self.id.clone()
    }
}
