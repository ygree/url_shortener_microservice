use std::convert::Infallible;
use std::future::{Future, Ready};
use std::pin::Pin;
use std::task::Context;
use futures::future::BoxFuture;
use hyper::service::Service;
use tokio::macros::support::Poll;

use crate::inmem_kvstore::InMemKVStore;

#[derive(Clone)]
pub struct KVService {
    kv_store: InMemKVStore,
}

impl KVService {
    pub fn new() -> KVService {
        KVService {
            kv_store: InMemKVStore::new()
        }
    }
}

pub enum KVServiceRequest {
    Put { key: String, value: String },
    Get(String),
}

// pub enum KVServiceResponse {
//     Put,
//     Value(String),
// }

impl Service<KVServiceRequest> for KVService {
    type Response = Option<String>;
    type Error = ();
    // type Future = Pin<Box<dyn Future<Output=Result<Self::Response, Self::Error>> + Send>>;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: KVServiceRequest) -> Self::Future {
        match req {
            KVServiceRequest::Get(key) => {
                let value = self.kv_store.get(key);
                Box::pin(async { Ok(value) })
            }
            KVServiceRequest::Put { key, value } => {
                self.kv_store.put(key, value);
                Box::pin(async { Ok(None) })
            }
        }
    }
}

// alternatively could separate it to two service implementation of the one for Put and the other for Get requests
pub struct Put {
    key: String,
    value: String,
}

impl Service<Put> for KVService {
    type Response = ();
    type Error = Infallible;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Put) -> Self::Future {
        let Put { key, value } = req;
        // mock call to a KV-store
        self.kv_store.put(key, value);
        // Box::pin(async { Ok(()) })
        core::future::ready(Ok(()))
    }
}


pub struct Get {
    key: String,
}

impl Get {
    pub fn new(key: String) -> Get {
        Get { key }
    }
}

impl Service<Get> for KVService {
    type Response = Option<String>;
    type Error = Infallible;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Get) -> Self::Future {
        // mock call to a KV-store
        let value = self.kv_store.get(req.key);
        core::future::ready(Ok(value))
    }
}