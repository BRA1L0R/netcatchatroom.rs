// use std::time::Duration;

use clienthandle::HandleExit;
use event::EventSystem;
use std::{sync::Arc, time::Duration};
use tokio::{io, net::TcpListener};

use crate::clienthandle::DisconnectError;

mod clienthandle;
mod connection;
mod event;
mod ratelimiter;

const BANTIME: Duration = Duration::from_secs(10);

async fn message_logger(ms: Arc<EventSystem>) {
    let mut rx = ms.subscribe();
    while let Ok(evt) = rx.recv().await {
        println!("{}", evt);
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
            match clienthandle::process_client(socket, ms).await {
                Err(HandleExit::Disconnect(DisconnectError::Spam)) => {
                    // ban the client if disconnected for spam
                    limiter.add(remote, BANTIME).await
                }
                // do nothing if client disconnected normally
                Err(HandleExit::Disconnect(_)) => (),
                // print error if disconnection reason was anything else
                Err(err) => println!("{} exited with error: {:?}", remote, err),
                _ => (),
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    run().await
}
