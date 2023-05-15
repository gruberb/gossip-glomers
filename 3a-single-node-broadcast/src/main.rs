mod connection;
mod message;
mod node;
mod storage;

use crate::connection::Connection;
use crate::message::{Body, Message};
use crate::node::Node;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    let stdin = std::io::stdin();
    let mut connection = Connection::new(stdin);

    let mut node = init_node(&mut connection);

    while let Some(message) = connection.read() {
        handle_message(&mut node, &mut connection, message);
    }
}

fn init_node(connection: &mut Connection) -> Node {
    let input = connection.read_one().expect("Didn't get input");

    let node;
    match input.body {
        Body::Init { msg_id, .. } => {
            node = Node::init(input.clone());

            let response = Message {
                src: node.id.clone(),
                dest: input.src,
                body: Body::InitOk {
                    in_reply_to: msg_id,
                },
            };

            connection.write(response);
        }
        _ => panic!("Node is not initalized yet"),
    }

    node
}

fn handle_message(node: &mut Node, connection: &mut Connection, input: Message) {
    match input.body {
        Body::Broadcast { msg_id, message } => {
            node.storage.add_message(message);

            let response = Message {
                src: node.id.clone(),
                dest: input.src,
                body: Body::BroadcastOk {
                    msg_id,
                    in_reply_to: msg_id,
                },
            };

            connection.write(response);
        }
        Body::Read { msg_id } => {
            let output = Message {
                src: node.id.clone(),
                dest: input.src,
                body: Body::ReadOk {
                    msg_id,
                    in_reply_to: msg_id,
                    messages: node.storage.get_messages(),
                },
            };

            connection.write(output);
        }
        Body::Topology { msg_id, topology } => {
            node.storage.init_topology(topology);

            let output = Message {
                src: node.id.clone(),
                dest: input.src,
                body: Body::TopologyOk {
                    msg_id,
                    in_reply_to: msg_id,
                },
            };

            connection.write(output);
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
