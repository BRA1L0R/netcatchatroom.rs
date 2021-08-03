use leaky_bucket::RateLimiter;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::broadcast::error::{RecvError, SendError};

use crate::connection::{self, Connection, Message};
use crate::event::{Event, EventSystem, MessageEvent};

enum ProcessLoop {
    ConnectionMessage(Message),
    ServerEvent(Event),
}

#[derive(Debug, Clone)]
pub enum DisconnectError {
    EOF,
    Spam,
}

#[derive(Debug)]
pub enum HandleExit {
    IoError(std::io::Error),

    RecvError(RecvError),
    SendError(SendError<Event>),

    Disconnect(DisconnectError),
}

impl From<RecvError> for HandleExit {
    fn from(err: RecvError) -> Self {
        HandleExit::RecvError(err)
    }
}

impl From<SendError<Event>> for HandleExit {
    fn from(err: SendError<Event>) -> Self {
        HandleExit::SendError(err)
    }
}

impl From<std::io::Error> for HandleExit {
    fn from(err: std::io::Error) -> Self {
        HandleExit::IoError(err)
    }
}

impl From<connection::ConnectionError> for HandleExit {
    fn from(err: connection::ConnectionError) -> Self {
        match err {
            connection::ConnectionError::IoError(err) => HandleExit::IoError(err),
            connection::ConnectionError::Eof => HandleExit::Disconnect(DisconnectError::EOF),
        }
    }
}

async fn handle_client(mut conn: Connection, ms: Arc<EventSystem>) -> Result<(), HandleExit> {
    let sender = ms.sender();
    let mut sub = ms.subscribe();

    // send a connect event
    sender.send(Event::Connect(conn.remote()))?;

    // setup a rate limiter using the leaky bucket algorithm
    let limiter = RateLimiter::builder()
        .max(10)
        .initial(10)
        .refill(5)
        .interval(std::time::Duration::from_secs(1))
        .build();

    loop {
        let evt = tokio::select! {
            msg = conn.read_message() => ProcessLoop::ConnectionMessage(msg?),
            msg = sub.recv() => ProcessLoop::ServerEvent(msg?),
        };

        let message = match evt {
            ProcessLoop::ConnectionMessage(message) if !message.content.is_empty() => message,
            ProcessLoop::ServerEvent(evt) => {
                conn.write_message(&Message {
                    content: format!("{}\n", evt),
                })
                .await?;

                continue;
            }

            _ => continue,
        };

        // if the leaky bucket does not contain enough tokens disconnect with a spam error
        if limiter.balance() < 1 {
            return Err(HandleExit::Disconnect(DisconnectError::Spam));
        }

        // acquire one from the leaky bucket
        limiter.acquire_one().await;

        let event = Event::Message(MessageEvent {
            sender: conn.remote(),
            message,
        });

        // send message event
        sender.send(event)?;
    }
}

pub async fn process_client(socket: TcpStream, ms: Arc<EventSystem>) -> Result<(), HandleExit> {
    let remote = socket.peer_addr()?.ip();
    let connection = connection::Connection::new(socket);

    handle_client(connection, ms.clone())
        .await
        .map_err(|err| match err {
            HandleExit::Disconnect(d) => {
                // send out a disconnection event
                ms.sender().send(Event::Disconnect(remote, d.clone()))?;

                // return the error to the top function
                Err(HandleExit::Disconnect(d))
            }
            err => Err(err),
        })
        .unwrap_err()

    // Ok(())
}
