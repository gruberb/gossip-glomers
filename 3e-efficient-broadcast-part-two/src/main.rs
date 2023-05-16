mod message;
mod node;
mod storage;

use std::{io::Write, println, sync::Arc, thread, time::Duration};

use rand::{prelude::*, rngs::StdRng};
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	sync::{
		mpsc,
		mpsc::{Receiver, Sender},
		Mutex,
	},
};

use crate::{
	message::{Body, Message},
	node::Node,
	storage::Storage,
};

const GOSSIP_DELAY: u64 = 500;
const MIN_AMOUNT_NODES: usize = 1;
const NETWORK_SIZE: usize = 25;

#[tokio::main]
async fn main() {
	let (reader_tx, mut reader_rx) = mpsc::channel(1000);
	let (writer_tx, mut writer_rx) = mpsc::channel(1000);

	let reader_tx1: Sender<Message> = reader_tx.clone();
	let writer_tx1: Sender<Message> = writer_tx.clone();
	let writer_tx2: Sender<Message> = writer_tx.clone();

	let store = Arc::new(Mutex::new(Storage::default()));

	let node = Node::bootstrap().await;

	let n1 = node.clone();
	let s1 = store.clone();

	let read = tokio::spawn(async move {
		read_from_stdin(reader_tx1).await;
	});

	let write = tokio::spawn(async move {
		write_to_stdout(&mut writer_rx).await;
	});

	let gossip = tokio::spawn(async move {
		loop {
			thread::sleep(Duration::from_millis(GOSSIP_DELAY));
			gossip_messages(n1.clone(), s1.clone(), writer_tx2.clone()).await;
		}
	});

	let handle = tokio::spawn(async move {
		handle_messages(node.clone(), store.clone(), &mut reader_rx, writer_tx1).await;
	});

	let _ = tokio::try_join!(read, handle, write, gossip);
}

async fn read_from_stdin(reader_tx: Sender<Message>) {
	let stdin = tokio::io::stdin();
	let mut reader = BufReader::new(stdin);

	loop {
		let mut buf = String::new();
		reader.read_line(&mut buf).await.unwrap();
		let message = Message::parse_message(buf.clone());
		reader_tx.send(message).await.unwrap();
	}
}

async fn write_to_stdout(writer_rx: &mut Receiver<Message>) {
	let mut stdout = std::io::stdout();

	loop {
		let message = writer_rx.recv().await.unwrap();
		let message = Message::format_message(message);
		writeln!(stdout, "{}", message).unwrap();
		stdout.flush().unwrap();
	}
}

async fn gossip_messages(node: Node, storage: Arc<Mutex<Storage>>, writer: Sender<Message>) {
	let mut rng = StdRng::from_entropy();

	let num_to_select = rng.gen_range(MIN_AMOUNT_NODES..=NETWORK_SIZE);

	let selected_neighbours: Vec<String> = node
		.get_network()
		.choose_multiple(&mut rng, num_to_select)
		.cloned()
		.collect();

	let mut tasks = vec![];

	for n in selected_neighbours {
		let storage_clone = storage.clone();
		let writer_clone = writer.clone();
		let node_clone = node.clone();

		let task = tokio::spawn(async move {
			let messages = storage_clone
				.lock()
				.await
				.get_new_messages_for_neighbour(n.clone());

			if messages.is_empty() {
				return;
			}

			let message = Message {
				src: node_clone.id.clone(),
				dest: n.clone(),
				body: Body::Gossip {
					messages: messages.clone(),
				},
			};

			writer_clone.send(message).await.unwrap();
		});

		tasks.push(task);
	}

	// Wait for all the gossip tasks to complete
	for task in tasks {
		task.await.unwrap();
	}
}

async fn handle_messages(
	node: Node,
	storage: Arc<Mutex<Storage>>,
	input: &mut Receiver<Message>,
	writer: Sender<Message>,
) {
	while let Some(input) = input.recv().await {
		match input.body {
			Body::Broadcast { msg_id, message } => {
				let id = node.id.clone();
				storage.lock().await.add_message(message);

				let response = Message {
					src: id,
					dest: input.src,
					body: Body::BroadcastOk {
						msg_id,
						in_reply_to: msg_id,
					},
				};

				writer.send(response).await.unwrap();
			}
			Body::Gossip { messages } => {
				let id = node.id.clone();
				storage
					.lock()
					.await
					.add_messages(messages.clone(), input.src.clone());

				let response = Message {
					src: id,
					dest: input.src,
					body: Body::GossipOk { messages },
				};

				writer.send(response).await.unwrap();
			}
			Body::GossipOk { messages } => {
				storage
					.lock()
					.await
					.add_to_sent_messages(messages, input.src);
			}
			Body::Read { msg_id } => {
				let response = Message {
					src: node.id.clone(),
					dest: input.src,
					body: Body::ReadOk {
						msg_id,
						in_reply_to: msg_id,
						messages: storage.lock().await.get_messages(),
					},
				};

				writer.send(response).await.unwrap();
			}
			Body::Topology { msg_id, .. } => {
				let response = Message {
					src: node.id.clone(),
					dest: input.src,
					body: Body::TopologyOk {
						msg_id,
						in_reply_to: msg_id,
					},
				};

				writer.send(response).await.unwrap();
			}
			Body::Error {
				in_reply_to,
				code,
				text,
			} => {
				eprintln!(
					"Error received (in_reply_to: {}, code: {}, text: {})",
					in_reply_to, code, text
				);
			}
			_ => (),
		}
	}
	println!("Error, nothing to read from receiver");
}
