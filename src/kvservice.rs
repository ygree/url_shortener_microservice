use std::collections::HashMap;
use std::convert::Infallible;
use std::future::Ready;
use std::sync::{Arc, Mutex};
use std::task::Context;
use hyper::service::Service;
use tokio::macros::support::Poll;

#[derive(Clone)]
pub struct KVService {
    hashmap: Arc<Mutex<HashMap<String, String>>>,
}

impl KVService {
    pub fn new() -> KVService {
        KVService {
            hashmap: Arc::new(Mutex::new(HashMap::new()))
        }
    }
}

pub struct Put {
    key: String,
    value: String,
}

impl Put {
    pub fn new(key: String, value: String) -> Put {
        Put { key, value }
    }
}

impl Service<Put> for KVService {
    type Response = ();
    type Error = Infallible;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Put) -> Self::Future {
        let Put { key, value } = req;
        // mock call to a KV-store
        let mut hm = self.hashmap.lock().unwrap();
        let _ = hm.insert(key, value);
        core::future::ready(Ok(()))
    }
}


pub struct GetByKey(pub String);

impl Service<GetByKey> for KVService {
    type Response = Option<String>;
    type Error = Infallible;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: GetByKey) -> Self::Future {
        // mock call to a KV-store
        let hm = self.hashmap.lock().unwrap();
        let GetByKey(key) = req;
        let value = hm.get(&key).map(|x| x.to_string());
        core::future::ready(Ok(value))
    }
}