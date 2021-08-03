use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    net::IpAddr,
};

pub trait HashToString {
    fn hash_to_string(&self) -> String;
}

impl HashToString for IpAddr {
    fn hash_to_string(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        let hash = hasher.finish().to_be_bytes();
        return hex::encode(hash);
    }
}
