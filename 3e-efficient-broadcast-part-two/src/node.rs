use crate::message::{Body, Message};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Network(pub(crate) HashSet<String>);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Node {
    pub(crate) id: String,
    pub(crate) availble_nodes: Vec<String>,
    pub(crate) network: Network,
}

impl Node {
    pub(crate) fn init(&self, message: Message) -> Node {
        match message.body {
            Body::Init {
                node_id, node_ids, ..
            } => {
                return Node {
                    id: node_id.clone(),
                    availble_nodes: node_ids.clone(),
                    network: self.init_network(node_ids),
                }
            }
            _ => panic!("Invalid message type"),
        }
    }

    fn init_network(&self, nodes: Vec<String>) -> Network {
        let mut neighbours = Network::default();
        neighbours.0.extend(nodes);
        neighbours
    }

    pub(crate) fn get_network(&self) -> Vec<String> {
        self.network.0.clone().into_iter().collect::<Vec<_>>()
    }
}
