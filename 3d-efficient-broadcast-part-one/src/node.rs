use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};

use crate::message::{Body, Message};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Neighbours(pub(crate) HashSet<String>);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Node {
    pub(crate) id: String,
    pub(crate) availble_nodes: Vec<String>,
    pub(crate) neighbours: Neighbours,
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
                    neighbours: self.init_topology(node_ids),
                }
            }
            _ => panic!("Invalid message type"),
        }
    }

    fn init_topology(&self, nodes: Vec<String>) -> Neighbours {
        let mut neighbours = Neighbours::default();

        let mut rng = thread_rng();
        let selections: Vec<String> = nodes.choose_multiple(&mut rng, 9).cloned().collect();

        neighbours.0.extend(selections);
        neighbours
    }

    pub(crate) fn get_neighbours(&self) -> HashSet<String> {
        self.neighbours.0.clone()
    }
}
