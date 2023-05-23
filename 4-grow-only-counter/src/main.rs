use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};

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
	Init {
		msg_id: u64,
		node_id: String,
		node_ids: Vec<String>,
	},
	InitOk {
		in_reply_to: u64,
	},
	Add {
		delta: u64,
		msg_id: u64,
		in_reply_to: u64,
	},
	AddOk {
		msg_id: u64,
		in_reply_to: u64,
	},
	Read {
		msg_id: u64,
	},
	ReadOk {
		value: u64,
		msg_id: u64,
		in_reply_to: u64,
	},
}

fn main() {
	let stdin = io::stdin();
	let mut stdout = io::stdout();

	for line in stdin.lock().lines() {
		let input: Message = serde_json::from_str(&line.unwrap()).unwrap();
		match input.body {
			Body::Init {
				msg_id, node_id, ..
			} => {
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
			_ => (),
		}
	}
}
