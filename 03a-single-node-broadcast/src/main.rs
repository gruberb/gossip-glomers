use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Body {
    Error {
        in_reply_to: u64,
        code: u64,
        text: String,
    },
    Init {
        msg_id: u64,
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk { in_reply_to: u64 },
    Broadcast {
        msg_id: u64,
        message: u64,
    },
    BroadcastOk {
        msg_id: u64,
        in_reply_to: u64,
    },
    Read {
        msg_id: u64,
    },
    ReadOk {
        msg_id: u64,
        in_reply_to: u64,
        messages: Vec<u64>,
    },
    Topology {
        msg_id: u64,
        topology: Topology,
    },
    TopologyOk {
        msg_id: u64,
        in_reply_to: u64,
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Messages(Vec<u64>);

#[derive(Serialize, Deserialize, Debug)]
struct Topology(HashMap<String, Vec<String>>);

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut node_id = String::new();

    let mut messages = Messages(Vec::new());

    for line in stdin.lock().lines() {
        let input: Message = serde_json::from_str(&line.unwrap()).unwrap();
        match input.body {
            Body::Init {
                msg_id,
                node_id: id,
                ..
            } => {
                node_id = id;
                let output = Message {
                    src: node_id.clone(),
                    dest: input.src,
                    body: Body::InitOk {
                        in_reply_to: msg_id,
                    },
                };
                let output_json = serde_json::to_string(&output).unwrap();
                writeln!(stdout, "{}", output_json).unwrap();
                stdout.flush().unwrap();
            }
            Body::Broadcast { msg_id, message } => {
                messages.0.push(message);

                let output = Message {
                    src: node_id.clone(),
                    dest: input.src,
                    body: Body::BroadcastOk {
                        msg_id,
                        in_reply_to: msg_id,
                    },
                };
                let output_json = serde_json::to_string(&output).unwrap();
                writeln!(stdout, "{}", output_json).unwrap();
                stdout.flush().unwrap();
            }
            Body::Read { msg_id } => {
                let output = Message {
                    src: node_id.clone(),
                    dest: input.src,
                    body: Body::ReadOk {
                        msg_id,
                        in_reply_to: msg_id,
                        messages: messages.0.clone(),
                    },
                };
                let output_json = serde_json::to_string(&output).unwrap();
                writeln!(stdout, "{}", output_json).unwrap();
                stdout.flush().unwrap();
            }
            Body::Topology { msg_id, .. } => {
                let output = Message {
                    src: node_id.clone(),
                    dest: input.src,
                    body: Body::TopologyOk {
                        msg_id,
                        in_reply_to: msg_id,
                    },
                };
                let output_json = serde_json::to_string(&output).unwrap();
                writeln!(stdout, "{}", output_json).unwrap();
                stdout.flush().unwrap();
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
