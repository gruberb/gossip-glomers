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
    pub(crate) fn init(message: Message) -> Node {
        match message.body {
            Body::Init {
                node_id, node_ids, ..
            } => {
                return Node {
                    id: node_id,
                    availble_nodes: node_ids,
                    storage: Storage::new(),
                }
            }
            _ => panic!("Invalid message type"),
        }
    }
}
