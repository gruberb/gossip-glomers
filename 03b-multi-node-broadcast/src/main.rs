mod message;
mod node;
mod storage;

use crate::message::{Body, Message};
use crate::node::Node;

use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncBufReadExt;
use tokio::io::{AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;
use tokio::sync::{broadcast, broadcast::Receiver, broadcast::Sender};
use tokio::time;

#[tokio::main]
async fn main() {
    let mut interval = time::interval(Duration::from_millis(200));
    let (reader_tx, mut reader_rx) = broadcast::channel(100);
    let (writer_tx, mut writer_rx) = broadcast::channel(100);

    let node = Arc::new(Mutex::new(Node::default()));

    let n1 = node.clone();
    let n2 = node.clone();

    let reader_tx1 = reader_tx.clone();
    let writer_tx1 = writer_tx.clone();
    let writer_tx2 = writer_tx.clone();

    let read = tokio::spawn(async move {
        read_from_stdin(reader_tx1).await;
    });

    let write = tokio::spawn(async move {
        write_to_stdout(&mut writer_rx).await;
    });

    tokio::spawn(async move {
        loop {
            interval.tick().await;
            gossip_messages(n1.clone(), writer_tx2.clone()).await;
        }
    });

    let handle = tokio::spawn(async move {
        handle_messages(n2, &mut reader_rx, writer_tx1).await;
    });

    let _ = tokio::try_join!(read, handle, write);
}

async fn read_from_stdin(reader_tx: Sender<Message>) {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);

    loop {
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        let message = Message::parse_message(buf.clone());
        reader_tx.send(message).unwrap();
    }
}

async fn write_to_stdout(writer_rx: &mut Receiver<Message>) {
    let stdout = tokio::io::stdout();
    let mut writer = BufWriter::new(stdout);

    loop {
        let message = writer_rx.recv().await.unwrap();
        let message = Message::format_message(message);
        writer.write_all(&message.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    }
}

async fn gossip_messages(node: Arc<Mutex<Node>>, writer: Sender<Message>) {
    let mut node = node.lock().await;

    for n in node.storage.get_neighbours(&node.get_id()) {
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

async fn handle_messages(
    node: Arc<Mutex<Node>>,
    input: &mut Receiver<Message>,
    writer: Sender<Message>,
) {
    while let Ok(input) = input.recv().await {
        match input.body {
            Body::Init { msg_id, .. } => {
                node.lock().await.init(input.clone());
                let id = node.lock().await.get_id();
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
                let id = node.lock().await.get_id();
                node.lock().await.storage.add_message(message, id.clone());

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
            Body::Read { msg_id } => {
                let id = node.lock().await.get_id();

                let response = Message {
                    src: id,
                    dest: input.src,
                    body: Body::ReadOk {
                        msg_id,
                        in_reply_to: msg_id,
                        messages: node.lock().await.storage.get_messages(),
                    },
                };

                writer.send(response).unwrap();
            }
            Body::Topology { msg_id, topology } => {
                let id = node.lock().await.get_id();
                node.lock().await.storage.init_topology(topology);

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
