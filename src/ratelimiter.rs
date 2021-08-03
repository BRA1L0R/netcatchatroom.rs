use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, SystemTime},
};

use tokio::sync::Mutex;

pub struct RateLimiter {
    list: Mutex<HashMap<IpAddr, SystemTime>>,
}

impl RateLimiter {
    pub fn new() -> RateLimiter {
        RateLimiter {
            list: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add(&self, ip: IpAddr, duration: Duration) {
        let mut list = self.list.lock().await;
        list.insert(ip, SystemTime::now().checked_add(duration).unwrap());
    }

    pub async fn is_blacklisted(&self, ip: &IpAddr) -> bool {
        let list = self.list.lock().await;
        match list.get(ip) {
            None => false,
            Some(v) => SystemTime::now() > *v,
        }
    }
}
