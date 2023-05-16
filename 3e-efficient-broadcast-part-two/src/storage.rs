use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Messages(pub(crate) HashSet<u64>);

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub(crate) struct Storage {
	pub(crate) messages: Messages,
	pub(crate) received_gossip_messages: HashMap<String, Messages>,
	pub(crate) sent_messages: HashMap<String, Messages>,
	pub(crate) retry: HashMap<String, u8>,
}

impl Storage {
	pub(crate) fn add_message(&mut self, message: u64) {
		if !self.messages.0.contains(&message) {
			self.messages.0.insert(message);
		}
	}

	pub(crate) fn add_messages(&mut self, messages: Vec<u64>, node: String) {
		if self.received_gossip_messages.contains_key(&node) {
			self.received_gossip_messages
				.get_mut(&node)
				.unwrap()
				.0
				.extend(messages.iter());
		} else {
			let mut v = Messages::default();
			v.0.extend(messages.iter());
			self.received_gossip_messages.insert(node, v);
		}

		for m in messages {
			if !self.messages.0.contains(&m) {
				self.messages.0.insert(m);
			}
		}
	}

	pub(crate) fn get_messages(&mut self) -> Vec<u64> {
		self.messages.0.iter().cloned().collect()
	}

	pub(crate) fn get_new_messages_for_neighbour(&self, node: String) -> Vec<u64> {
		let received_messages = self.messages.0.clone().into_iter().collect::<Vec<_>>();

		let sent_to_node: Vec<u64> = self
			.sent_messages
			.iter()
			.filter(|(key, _)| *key == &node)
			.flat_map(|(_, Messages(value))| value)
			.cloned()
			.collect();

		let received_from_node: Vec<u64> = self
			.received_gossip_messages
			.iter()
			.filter(|(key, _)| *key == &node)
			.flat_map(|(_, Messages(value))| value)
			.cloned()
			.collect();

		let filtered_messages: Vec<u64> = received_messages
			.into_iter()
			.filter(|x| !sent_to_node.contains(x) && !received_from_node.contains(x))
			.collect();

		filtered_messages
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
