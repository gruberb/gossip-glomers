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
	InitOk {
		in_reply_to: u64,
	},
	Read {
		msg_id: u64,
	},
	ReadOk {
		value: u64,
		in_reply_to: u64,
	},
	Add {
		msg_id: u64,
		delta: u64,
	},
	AddOk {
		in_reply_to: u64,
	},
	//SEQ-KV
	Write {
		msg_id: u64,
		key: String,
		value: u64,
	},
	WriteOk {
		in_reply_to: u64,
	},
}

fn main() {
	let stdin = io::stdin();
	let mut stdout = io::stdout();
	let mut stderr = io::stderr();

	let mut id = String::new();

	for line in stdin.lock().lines() {
		let input: Message = serde_json::from_str(&line.unwrap()).unwrap();
		match input.body {
			Body::Error { text, .. } => {
				let output_json = serde_json::to_string(&text).unwrap();
				writeln!(stderr, "{}", output_json).unwrap();
				stdout.flush().unwrap();
			}
			Body::Init {
				msg_id, node_id, ..
			} => {
				id = node_id.clone();
				let output = Message {
					src: id.clone(),
					dest: input.src,
					body: Body::InitOk {
						in_reply_to: msg_id,
					},
				};
				let output_json = serde_json::to_string(&output).unwrap();
				writeln!(stdout, "{}", output_json).unwrap();

				let output = Message {
					src: id.clone(),
					dest: "seq-kv".to_string(),
					body: Body::Write {
						msg_id: 1,
						key: "TEST".to_string(),
						value: 42,
					},
				};
				let output_json = serde_json::to_string(&output).unwrap();
				writeln!(stdout, "{}", output_json).unwrap();
				stdout.flush().unwrap();
			}
			Body::Read { msg_id } => {
				let output = Message {
					src: id.clone(),
					dest: input.src,
					body: Body::ReadOk {
						in_reply_to: msg_id,
						value: 42,
					},
				};
				let output_json = serde_json::to_string(&output).unwrap();
				writeln!(stdout, "{}", output_json).unwrap();
				stdout.flush().unwrap();
			}
			Body::Add { msg_id, .. } => {
				let output = Message {
					src: id.clone(),
					dest: input.src,
					body: Body::AddOk {
						in_reply_to: msg_id,
					},
				};
				let output_json = serde_json::to_string(&output).unwrap();
				writeln!(stdout, "{}", output_json).unwrap();
				stdout.flush().unwrap();
			}
			Body::WriteOk { .. } => {
				//
			}
			_ => println!("Unhandled message: {:?}", input),
		}
	}
}
