use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::future::BoxFuture;

mod kvservice;
mod uniqueid;
mod urlshortener;

use kvservice::KVService;
use crate::kvservice::{GetByKey, Put};
use crate::uniqueid::{GetUniqueId, UniqueId, UniqueIdGen};
use crate::urlshortener::UrlShortener;

//TODO add logging
//TODO rename project
//TODO publish to github

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let addr = ([127, 0, 0, 1], 3000).into();

    // KV-store mock impl
    let kv_service = KVService::new();
    // unique id gen mock impl
    let mut unique_id_gen = UniqueIdGen::new();

    // TODO compose service out of layers:
    // 1. Parse GET request
    // 2. Request from cache
    // 3. Request from store
    // 2'. Update cache
    // 4. Serialize response (could be an error)
    //
    // 1. Parse POST request
    // 2. Update store
    // 3. Update cache (can skip it for simplicity)
    // 4. Serialize response (could be an error)


    let server = Server::bind(&addr)
        .serve(
            MakeSvc {
                kv_service: kv_service.clone(),
                unique_id_gen
            }
        );

    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}

//TODO replace it with a makeservice_fn
struct MakeSvc {
    kv_service: KVService,
    unique_id_gen: UniqueIdGen,
}

impl<T> Service<T> for MakeSvc {
    type Response = UrlShortener;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let kv_service = self.kv_service.clone();
        let unique_id_gen = self.unique_id_gen.clone();
        let fut = async move { Ok(UrlShortener::new(kv_service, unique_id_gen)) };
        Box::pin(fut)
    }
}
