// use std::io;
use std::{fmt::Display, net::IpAddr};
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{clienthandle::DisconnectError, connection::Message};

#[derive(Clone, Debug)]
pub struct MessageEvent {
    pub sender: IpAddr,
    pub message: Message,
}

#[derive(Clone, Debug)]
pub enum Event {
    Message(MessageEvent),
    Connect(IpAddr),
    Disconnect(IpAddr, DisconnectError),
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Message(msg) => write!(f, "{} > {}", msg.sender, msg.message.content),
            Event::Connect(ip) => write!(f, "{} connected to the server", ip),

            Event::Disconnect(ip, DisconnectError::Spam) => {
                write!(f, "{} has been kicked for spamming", ip)
            }
            Event::Disconnect(ip, _) => write!(f, "{} disconnected from the server", ip),
        }
    }
}

pub struct EventSystem {
    tx: Sender<Event>,
    // rx: Receiver<Message>,
}

impl EventSystem {
    pub fn new(cap: usize) -> EventSystem {
        let (tx, _) = broadcast::channel::<Event>(cap);

        EventSystem { tx }
    }

    pub fn sender(&self) -> Sender<Event> {
        self.tx.clone()
    }

    pub fn subscribe(&self) -> Receiver<Event> {
        Sender::subscribe(&self.tx)
    }
}
