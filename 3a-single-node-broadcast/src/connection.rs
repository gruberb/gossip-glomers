use crate::message::Message;
use log::info;
use std::io::{BufRead, Write};

#[derive(Debug)]
pub struct Connection<'a> {
    reader: std::io::BufReader<std::io::StdinLock<'a>>,
    writer: std::io::Stdout,
}

impl<'a> Connection<'a> {
    pub fn new(stdin: std::io::Stdin) -> Self {
        Connection {
            reader: std::io::BufReader::new(stdin.lock()),
            writer: std::io::stdout(),
        }
    }

    pub fn read_one(&mut self) -> Option<Message> {
        let mut buf = String::new();
        let _ = self.reader.read_line(&mut buf);
        info!("read_from_stdin:: {buf:?}");
        return Some(Message::parse_message(buf));
    }

    pub fn read(&mut self) -> Option<Message> {
        let mut buffer = String::new();

        match self.reader.read_line(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read > 0 {
                    serde_json::from_str(&buffer).ok()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn write(&mut self, message: Message) {
        info!("write to stdout: {message:?}");
        let message = Message::format_message(message);
        writeln!(self.writer, "{}", message).unwrap();
        self.writer.flush().unwrap();
    }
}
