use std::collections::HashMap;
use std::convert::Infallible;
use std::future::{Future, Ready};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
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


pub struct GetByKey(pub String);

impl Service<GetByKey> for KVService {
    type Response = Option<String>;
    type Error = Infallible;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: GetByKey) -> Self::Future {
        // mock call to a KV-store
        let value = self.kv_store.get(req.0);
        core::future::ready(Ok(value))
    }
}