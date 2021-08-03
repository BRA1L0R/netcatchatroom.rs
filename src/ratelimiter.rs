use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, SystemTime},
};

use tokio::sync::Mutex;

pub struct IpBanner {
    list: Mutex<HashMap<IpAddr, SystemTime>>,
}

impl IpBanner {
    pub fn new() -> IpBanner {
        IpBanner {
            list: Mutex::new(HashMap::new()),
        }
    }

    pub async fn add(&self, ip: IpAddr, duration: Duration) {
        let mut list = self.list.lock().await;
        list.insert(ip, SystemTime::now().checked_add(duration).unwrap());
    }

    pub async fn is_blacklisted(&self, ip: &IpAddr) -> bool {
        let mut list = self.list.lock().await;
        match list.get(ip) {
            None => false,
            Some(v) => {
                let bled = SystemTime::now() < *v;

                // remove from hashmap if not blacklisted anymore
                if !bled {
                    list.remove(ip);
                }

                bled
            }
        }
    }
}
