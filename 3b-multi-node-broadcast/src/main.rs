mod message;
mod node;
mod storage;

use crate::message::{Body, Message};
use crate::node::Node;

use std::io::prelude::*;
use std::io::{BufReader, Write};
use std::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

fn main() {
    let (reader_tx, mut reader_rx) = mpsc::channel();
    let (writer_tx, mut writer_rx) = mpsc::channel();

    let node = Arc::new(Mutex::new(Node::default()));

    let n1 = node.clone();
    let n2 = node.clone();

    let reader_tx1: Sender<Message> = reader_tx.clone();
    let writer_tx1: Sender<Message> = writer_tx.clone();
    let writer_tx2: Sender<Message> = writer_tx.clone();

    let read = thread::spawn(move || {
        read_from_stdin(reader_tx1);
    });

    let write = thread::spawn(move || {
        write_to_stdout(&mut writer_rx);
    });

    let gossip = thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        gossip_messages(n1.clone(), writer_tx2.clone());
    });

    let handle = thread::spawn(move || {
        handle_messages(n2, &mut reader_rx, writer_tx1);
    });
    let _ = handle.join();
    let _ = write.join();
    let _ = gossip.join();
    let _ = read.join();
}

fn read_from_stdin(reader_tx: Sender<Message>) {
    let stdin = std::io::stdin();
    let mut reader = BufReader::new(stdin.lock());

    loop {
        let mut buf = String::new();
        reader.read_line(&mut buf).unwrap();
        let message = Message::parse_message(buf.clone());
        reader_tx.send(message).unwrap();
    }
}

fn write_to_stdout(writer_rx: &mut Receiver<Message>) {
    let mut stdout = std::io::stdout();

    loop {
        let message = writer_rx.recv().unwrap();
        let message = Message::format_message(message);
        writeln!(stdout, "{}", message).unwrap();
        stdout.flush().unwrap();
    }
}

fn gossip_messages(node: Arc<Mutex<Node>>, writer: Sender<Message>) {
    let mut node = node.lock().unwrap();
    if let Some(neighbours) = node.storage.get_neighbours(&node.get_id()) {
        for n in neighbours {
            let messages = node.storage.get_messages_for_node(n.clone());
            let message = Message {
                src: node.id.clone(),
                dest: n.clone(),
                body: Body::Gossip {
                    messages: messages.clone(),
                },
            };

            writer.send(message).unwrap();
            node.storage
                .add_to_sent_messages(messages.into_iter().collect(), n);
        }
    }
}

fn handle_messages(node: Arc<Mutex<Node>>, input: &mut Receiver<Message>, writer: Sender<Message>) {
    while let Ok(input) = input.recv() {
        match input.body {
            Body::Init { msg_id, .. } => {
                node.lock().unwrap().init(input.clone());
                let id = node.lock().unwrap().get_id();
                let response = Message {
                    src: id,
                    dest: input.src,
                    body: Body::InitOk {
                        in_reply_to: msg_id,
                    },
                };

                writer.send(response).unwrap();
            }
            Body::Broadcast { msg_id, message } => {
                let id = node.lock().unwrap().get_id();
                node.lock()
                    .unwrap()
                    .storage
                    .add_message(message, id.clone());

                let response = Message {
                    src: id,
                    dest: input.src,
                    body: Body::BroadcastOk {
                        msg_id,
                        in_reply_to: msg_id,
                    },
                };

                writer.send(response).unwrap();
            }
            Body::Gossip { messages } => {
                let id = node.lock().unwrap().get_id();
                for m in messages.into_iter() {
                    node.lock().unwrap().storage.add_message(m, id.clone());
                }
            }
            Body::Read { msg_id } => {
                let id = node.lock().unwrap().get_id();

                let response = Message {
                    src: id,
                    dest: input.src,
                    body: Body::ReadOk {
                        msg_id,
                        in_reply_to: msg_id,
                        messages: node.lock().unwrap().storage.get_messages(),
                    },
                };

                writer.send(response).unwrap();
            }
            Body::Topology { msg_id, topology } => {
                let id = node.lock().unwrap().get_id();
                node.lock().unwrap().storage.init_topology(topology);

                let response = Message {
                    src: id,
                    dest: input.src,
                    body: Body::TopologyOk {
                        msg_id,
                        in_reply_to: msg_id,
                    },
                };

                writer.send(response).unwrap();
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
