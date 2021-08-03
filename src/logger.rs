use std::sync::Arc;

use crate::{event::EventSystem, iphash::HashToString};

pub async fn message_logger(ms: Arc<EventSystem>) {
    let mut rx = ms.subscribe();
    while let Ok(evt) = rx.recv().await {
        let origin = if let Some(ip) = evt.origin {
            format!("[{}] <{}>", ip, ip.hash_to_string())
        } else {
            "[Server]".to_string()
        };

        println!("{} {}", origin, evt.event);
    }
}
