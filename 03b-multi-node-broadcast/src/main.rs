mod message;
mod node;
mod storage;

use crate::message::{Body, Message};
use crate::node::Node;

use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;
use tokio::sync::{mpsc, mpsc::Receiver, mpsc::Sender};
use tokio::time;

#[tokio::main]
async fn main() {
    // let mut interval = time::interval(Duration::from_secs(5));

    let (reader_tx, mut reader_rx) = mpsc::channel(100);
    let (writer_tx, mut writer_rx) = mpsc::channel(100);

    let node = Arc::new(Mutex::new(Node::default()));
    // let writer_tx = Arc::new(Mutex::new(writer_tx));

    let n1 = node.clone();
    let n2 = node.clone();

    let w1_tx = writer_tx.clone();
    let w2_tx = writer_tx.clone();

    let read = tokio::spawn(async move {
        read_from_stdin(reader_tx).await;
    });

    let write = tokio::spawn(async move {
        write_to_stdout(&mut writer_rx).await;
    });

    // tokio::spawn(async move {
    //     loop {
    //         interval.tick().await;
    //         gossip_messages(n1.clone(), w1_tx.clone()).await;
    //     }
    // });

    let handle = tokio::spawn(async move {
        handle_messages(n2, &mut reader_rx, writer_tx).await;
    });

    let _ = tokio::join!(read, write, handle);
}

async fn read_from_stdin(reader_tx: Sender<Message>) {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();
    eprintln!("Reading from stdin");
    while let Ok(Some(line)) = reader.next_line().await {
        eprintln!("Reading next line {line:?}");
        let message = Message::parse_message(line);
        reader_tx.send(message).await.unwrap();
    }
}

async fn write_to_stdout(writer_rx: &mut Receiver<Message>) {
    let stdout = tokio::io::stdout();
    let mut writer = BufWriter::new(stdout);

    while let Some(message) = writer_rx.recv().await {
        let message = Message::format_message(message);
        if let Err(e) = writer.write_all(message.as_bytes()).await {
            eprintln!("Failed to write to stdout: {}", e);
        }

        if let Err(e) = writer.flush().await {
            eprintln!("Failed to flush stdout: {}", e);
        }
    }
}

async fn gossip_messages(node: Arc<Mutex<Node>>, writer: Arc<Mutex<Sender<Message>>>) {
    let mut node = node.lock().await;
    let writer = writer.lock().await;

    for n in node.storage.get_neighbours(&node.get_id()) {
        let messages = node.storage.get_messages_for_node(n.clone());
        let message = Message {
            src: node.id.clone(),
            dest: n.clone(),
            body: Body::Gossip {
                messages: messages.clone(),
            },
        };

        let _ = writer.send(message).await.unwrap();
        node.storage
            .add_to_sent_messages(messages.into_iter().collect(), n);
    }
}

async fn handle_messages(
    node: Arc<Mutex<Node>>,
    input: &mut Receiver<Message>,
    writer: Sender<Message>,
) {
    while let Some(input) = input.recv().await {
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

                let _ = writer.send(response).await;
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

                let _ = writer.send(response).await;
            }
            Body::Read { msg_id } => {
                let id = node.lock().await.get_id();

                let output = Message {
                    src: id,
                    dest: input.src,
                    body: Body::ReadOk {
                        msg_id,
                        in_reply_to: msg_id,
                        messages: node.lock().await.storage.get_messages(),
                    },
                };

                let _ = writer.send(output).await;
            }
            Body::Topology { msg_id, topology } => {
                let id = node.lock().await.get_id();
                node.lock().await.storage.init_topology(topology);

                let output = Message {
                    src: id,
                    dest: input.src,
                    body: Body::TopologyOk {
                        msg_id,
                        in_reply_to: msg_id,
                    },
                };

                let _ = writer.send(output).await;
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
}
