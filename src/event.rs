// use std::io;
use std::net::IpAddr;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::connection::Message;

#[derive(Clone, Debug)]
pub struct MessageEvent {
    pub sender: IpAddr,
    pub message: Message,
}

#[derive(Clone, Debug)]
pub enum Event {
    Message(MessageEvent),
    Connect(IpAddr),
    Disconnect(IpAddr),
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
