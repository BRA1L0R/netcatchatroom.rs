// use std::time::Duration;

use clienthandle::ProcessError;
use event::EventSystem;
use std::{net::IpAddr, sync::Arc, time::Duration};
use tokio::{io, net::TcpListener};

use crate::event::Event;

mod clienthandle;
mod connection;
mod event;
mod ratelimiter;

const BANTIME: Duration = Duration::from_secs(10);

async fn message_logger(ms: Arc<EventSystem>) {
    let mut rx = ms.subscribe();
    while let Ok(evt) = rx.recv().await {
        // println!("Received message: {:?}", msg);
        println!("{:?}", evt);

        let message: (IpAddr, String) = match evt {
            Event::Message(msg) => (msg.sender, msg.message.content),
            Event::Connect(ip) => (ip, "connected to the server".to_string()),
            Event::Disconnect(ip) => (ip, "disconnected from the server".to_string()),
        };

        println!("{} > {} here", message.0, message.1);
    }
}

async fn run() -> Result<(), io::Error> {
    let listener = TcpListener::bind("0.0.0.0:6161").await.unwrap();
    let ms = Arc::new(EventSystem::new(50));

    {
        let ms = Arc::clone(&ms);
        tokio::spawn(message_logger(ms));
    }

    let limiter = Arc::new(ratelimiter::RateLimiter::new());

    loop {
        let (socket, _) = listener.accept().await?;
        let remote = socket.peer_addr().unwrap().ip();

        // ratelimit checking
        if limiter.is_blacklisted(&remote).await {
            continue;
        };

        // clone all the values the coroutine will need
        let ms = Arc::clone(&ms);
        let limiter = limiter.clone();

        // launch the coroutine
        tokio::spawn(async move {
            if let Err(err) = clienthandle::process_client(socket, ms).await {
                match err {
                    ProcessError::Disconnected => {}
                    ProcessError::Spam => limiter.add(remote, BANTIME).await,
                    err => eprintln!("Error encountered: {:?}", err),
                }
            }
            // out.unwrap_or_else(async |err| match err {
            //     ProcessError::Disconnected => {}
            //     ProcessError::Spam => limiter.add(socket.peer_addr().unwrap().ip(), time).await,
            //     err => eprintln!("Error encountered: {:?}", err),
            // });
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    run().await
}
