use std::io::{self, BufRead, Write};

use serde::{Deserialize, Serialize};

const SEQ_KV: &str = "seq-kv";
const KEY: &str = "counter";

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
		msg_id: Option<u64>,
		key: Option<String>,
	},
	ReadOk {
		in_reply_to: Option<u64>,
		value: u64,
	},
	Add {
		msg_id: u64,
		delta: u64,
	},
	AddOk {
		in_reply_to: u64,
	},
	Write {
		msg_id: u64,
		key: String,
		value: u64,
	},
	WriteOk {
		in_reply_to: u64,
	},
	Cas {
		msg_id: u64,
		key: String,
		from: u64,
		to: u64,
	},
	CasOk {
		msg_id: u64,
		in_reply_to: u64,
	},
}

fn main() {
	let stdin = io::stdin();
	let mut stdout = io::stdout();
	let mut stderr = io::stderr();

	let mut id = String::new();
	let mut counter = 0;
	let mut tmp_counter = 0;

	for line in stdin.lock().lines() {
		let input = serde_json::from_str::<Message>(&line.as_ref().unwrap());

		if let Err(e) = input {
			writeln!(stderr, "Error: {:?}", e).unwrap();
			stderr.flush().unwrap();
			continue;
		}
		let input: Message = serde_json::from_str(&line.unwrap()).unwrap();

		match input.body {
			Body::Error { code, .. } => match code {
				22 => {
					let output = Message {
						src: id.clone(),
						dest: SEQ_KV.to_string(),
						body: Body::Read {
							msg_id: Some(2),
							key: Some(KEY.to_string()),
						},
					};
					let output_json = serde_json::to_string(&output).unwrap();
					writeln!(stdout, "{}", output_json).unwrap();
					stdout.flush().unwrap();
				}
				_ => {
					writeln!(stderr, "Error: {:?}", input).unwrap();
					stderr.flush().unwrap();
				}
			},
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
					dest: SEQ_KV.to_string(),
					body: Body::Write {
						msg_id,
						key: KEY.to_string(),
						value: counter,
					},
				};
				let output_json = serde_json::to_string(&output).unwrap();
				writeln!(stdout, "{}", output_json).unwrap();
				stdout.flush().unwrap();
			}
			Body::Read { msg_id, .. } => {
				let output = Message {
					src: id.clone(),
					dest: input.src,
					body: Body::ReadOk {
						in_reply_to: msg_id,
						value: counter,
					},
				};
				let output_json = serde_json::to_string(&output).unwrap();
				writeln!(stdout, "{}", output_json).unwrap();

				stdout.flush().unwrap();
			}
			Body::Add { msg_id, delta } => {
				tmp_counter += delta;

				let output = Message {
					src: id.clone(),
					dest: SEQ_KV.to_string(),
					body: Body::Cas {
						msg_id,
						key: KEY.to_string(),
						from: counter,
						to: tmp_counter,
					},
				};
				let output_json = serde_json::to_string(&output).unwrap();
				writeln!(stdout, "{}", output_json).unwrap();

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
			Body::AddOk { .. } => {
				//
			}
			Body::CasOk { .. } => {
				counter = tmp_counter;
			}
			Body::ReadOk { value, .. } => {
				counter = value;
			}
			_ => println!("Unhandled message: {:?}", input),
		}
	}
}
