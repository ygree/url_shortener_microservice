use std::collections::HashMap;

use std::sync::{Arc, Mutex};

pub struct KVStore {
    hashmap: Arc<Mutex<HashMap<String, String>>>,
}

impl KVStore {
    pub fn new() -> KVStore {
        KVStore {
            hashmap: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn put(&mut self, key: String, value: String) {
        let mut hm = self.hashmap.lock().unwrap();
        hm.insert(key, value);
    }

    pub fn get(&self, key: String) -> Option<String> {
        let mut hm = self.hashmap.lock().unwrap();
        hm.get(&key).map(|x| x.to_string())
    }
}