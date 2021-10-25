use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::future::BoxFuture;
use log::{debug, error, log_enabled, info, Level};

mod kvservice;
mod uniqueid;
mod urlshortener;

use kvservice::KVService;
use urlshortener::UrlShortener;
use uniqueid::UniqueIdGen;

//TODO publish to github

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    let addr = ([127, 0, 0, 1], 3000).into();

    // KV-store mock impl
    let kv_service = KVService::new();
    // unique id gen mock impl
    let unique_id_gen = UniqueIdGen::new();

    let server = Server::bind(&addr)
        .serve(
            MakeSvc {
                kv_service,
                unique_id_gen
            }
        );

    info!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}

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
