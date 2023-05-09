use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    src: String,
    dest: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
enum Body {
    #[serde(rename = "init")]
    Init {
        msg_id: u64,
        node_id: String,
        node_ids: Vec<String>,
    },
    #[serde(rename = "init_ok")]
    InitOk { in_reply_to: u64 },
    #[serde(rename = "echo")]
    Echo { msg_id: u64, echo: String },
    #[serde(rename = "echo_ok")]
    EchoOk {
        msg_id: u64,
        in_reply_to: u64,
        echo: String,
    },
    #[serde(rename = "error")]
    Error {
        in_reply_to: u64,
        code: u64,
        text: String,
    },
}

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut node_id = String::new();

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
            Body::Echo { msg_id, echo } => {
                let output = Message {
                    src: node_id.clone(),
                    dest: input.src,
                    body: Body::EchoOk {
                        msg_id,
                        in_reply_to: msg_id,
                        echo,
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
