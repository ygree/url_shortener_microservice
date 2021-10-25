
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct InMemKVStore {
    hashmap: Arc<Mutex<HashMap<String, String>>>,
}

impl InMemKVStore {
    pub fn new() -> InMemKVStore {
        InMemKVStore {
            hashmap: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn put(&mut self, key: String, value: String) {
        let mut hm = self.hashmap.lock().unwrap();
        let old_value = hm.insert(key, value);

    }

    pub fn get(&self, key: String) -> Option<String> {
        let mut hm = self.hashmap.lock().unwrap();
        hm.get(&key).map(|x| x.to_string())
    }
}