use std::{collections::HashSet, io::Write};

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::message::{Body, Message};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Network(pub(crate) HashSet<String>);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Node {
	pub(crate) id: String,
	pub(crate) availble_nodes: Vec<String>,
	pub(crate) network: Network,
}

impl Node {
	pub(crate) async fn bootstrap() -> Node {
		let stdin = tokio::io::stdin();
		let mut stdout = std::io::stdout();

		let mut reader = BufReader::new(stdin);
		let mut buf = String::new();

		reader.read_line(&mut buf).await.unwrap();
		let message = Message::parse_message(buf.clone());
		let node = Node::init(message.clone());

		match message.body {
			Body::Init {
				msg_id, node_id, ..
			} => {
				let response = Message {
					src: node_id,
					dest: message.src.clone(),
					body: Body::InitOk {
						in_reply_to: msg_id,
					},
				};

				let message = Message::format_message(response);
				writeln!(stdout, "{}", message).unwrap();
				stdout.flush().unwrap();
			}
			_ => (),
		}

		node
	}

	pub(crate) fn init(message: Message) -> Node {
		match message.body {
			Body::Init {
				node_id, node_ids, ..
			} => {
				return Node {
					id: node_id.clone(),
					availble_nodes: node_ids.clone(),
					network: Node::init_network(node_ids),
				}
			}
			_ => panic!("Invalid message type"),
		}
	}

	fn init_network(nodes: Vec<String>) -> Network {
		let mut neighbours = Network::default();
		neighbours.0.extend(nodes);
		neighbours
	}

	pub(crate) fn get_network(&self) -> Vec<String> {
		self.network.0.clone().into_iter().collect::<Vec<_>>()
	}
}
