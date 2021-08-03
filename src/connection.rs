use std::{fmt::Display, net::IpAddr};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};
// use crate::message::Message;

#[derive(Clone, Debug)]
pub struct Message {
    pub content: String,
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

pub struct Connection {
    ip: IpAddr,
    stream: BufReader<TcpStream>,
}

pub enum ConnectionError {
    IoError(std::io::Error),
    Eof,
}

impl From<std::io::Error> for ConnectionError {
    fn from(err: std::io::Error) -> Self {
        ConnectionError::IoError(err)
    }
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            ip: stream.peer_addr().unwrap().ip(),
            stream: BufReader::new(stream),
        }
    }

    pub fn remote(&self) -> IpAddr {
        self.ip
    }

    pub async fn read_message(&mut self) -> Result<Message, ConnectionError> {
        let mut content = String::new();
        self.stream.read_line(&mut content).await?;

        if content.is_empty() {
            Err(ConnectionError::Eof)
        } else {
            // defaults to just a normal line
            Ok(Message {
                content: content.trim().to_string(),
            })
        }
    }

    pub async fn write_message(&mut self, msg: &Message) -> Result<(), std::io::Error> {
        self.stream.write_all(msg.content.as_bytes()).await
    }
}
