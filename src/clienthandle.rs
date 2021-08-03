use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::broadcast::error::{RecvError, SendError};

use crate::connection::{self, Connection, Message};
use crate::event::{Event, EventSystem, MessageEvent};

enum ProcessLoop {
    ConnectionMessage(Message),
    ServerEvent(Event),
}

#[derive(Debug)]
pub enum ProcessError {
    IoError(std::io::Error),

    RecvError(RecvError),
    SendError(SendError<Event>),

    Spam,
    Disconnected,
}

impl From<RecvError> for ProcessError {
    fn from(err: RecvError) -> Self {
        ProcessError::RecvError(err)
    }
}

impl From<SendError<Event>> for ProcessError {
    fn from(err: SendError<Event>) -> Self {
        ProcessError::SendError(err)
    }
}

impl From<std::io::Error> for ProcessError {
    fn from(err: std::io::Error) -> Self {
        ProcessError::IoError(err)
    }
}

impl From<connection::ConnectionError> for ProcessError {
    fn from(err: connection::ConnectionError) -> Self {
        match err {
            connection::ConnectionError::IoError(err) => ProcessError::IoError(err),
            connection::ConnectionError::Eof => ProcessError::Disconnected,
        }
    }
}

async fn handle_client(mut conn: Connection, ms: Arc<EventSystem>) -> Result<(), ProcessError> {
    let sender = ms.sender();
    let mut sub = ms.subscribe();

    // send a connect event
    sender.send(Event::Connect(conn.remote()))?;

    loop {
        // conn.stream.
        let evt = tokio::select! {
            msg = conn.read_message() => ProcessLoop::ConnectionMessage(msg?),
            msg = sub.recv() => ProcessLoop::ServerEvent(msg?),
        };

        let evt = match evt {
            ProcessLoop::ServerEvent(evt) => evt,
            ProcessLoop::ConnectionMessage(message) if !message.content.is_empty() => {
                let event = Event::Message(MessageEvent {
                    sender: conn.remote(),
                    message,
                });

                sender.send(event)?;
                continue;
            }
            _ => continue,
        };

        if let Event::Message(msg) = evt {
            let content = format!("<{}> {}\n", msg.sender, msg.message);
            conn.write_message(&Message { content }).await?;
        }
    }
}

pub async fn process_client(socket: TcpStream, ms: Arc<EventSystem>) -> Result<(), ProcessError> {
    let remote = socket.peer_addr()?.ip();
    let connection = connection::Connection::new(socket);

    handle_client(connection, ms.clone())
        .await
        .map_err(|err| match err {
            ProcessError::Disconnected => {
                ms.sender().send(Event::Disconnect(remote))?;
                Err(ProcessError::Disconnected)
            }
            err => Err(err),
        })
        .unwrap_err()

    // Ok(())
}
