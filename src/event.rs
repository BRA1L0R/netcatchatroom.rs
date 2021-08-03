// use std::io;
use crate::{clienthandle::DisconnectError, connection::Message};
use displaydoc::Display;
use std::net::IpAddr;
use tokio::sync::broadcast::{self, Receiver, Sender};

#[derive(Clone, Debug, Display)]
pub enum EventType {
    /// {0}
    Message(Message),
    /// disconnected from the server
    Disconnect(DisconnectError),
    /// connected to the server
    Connect,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub origin: Option<IpAddr>,
    pub event: EventType,
}

impl Event {
    pub fn new(ip: IpAddr, event: EventType) -> Event {
        Event {
            origin: Some(ip),
            event,
        }
    }
}

pub struct EventSystem {
    tx: Sender<Event>,
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
