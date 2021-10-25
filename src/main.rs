use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicUsize;
use std::task::{Context, Poll};
use futures::future::BoxFuture;

mod kvservice;
mod inmem_kvstore;
mod uniqueid;

use kvservice::KVService;
use crate::uniqueid::UniqueIdGen;

//TODO rename project
//TODO publish to github

static UNIQUE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let addr = ([127, 0, 0, 1], 3000).into();
    let kv_service = KVService::new();
    let unique_id_gen = UniqueIdGen::new(&UNIQUE_ID_COUNTER);

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
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let kv_service = self.kv_service.clone();
        let unique_id_gen = self.unique_id_gen.clone();
        let fut = async move { Ok(Svc { kv_service, unique_id_gen }) };
        Box::pin(fut)
    }
}

#[derive(Clone)]
struct Svc {
    kv_service: KVService,
    unique_id_gen: UniqueIdGen,
}

impl Svc {
    async fn get_value(&self, key: &str) -> Result<Option<String>, Infallible> {
        let mut kvs = self.kv_service.clone();
        kvs.call(kvservice::Get::new(key.to_string())).await
        // kvs.call(KVServiceRequest::Get("test".to_string())).await
    }
}

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    // type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>; // TODO how to avoid allocation here?
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>; // TODO how to avoid allocation here?

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        // TODO handle POST request { url = <.full url.> } and return ( url = <.short url.> }
        // TODO handle GET request { url = <.full or short url.> } and return { url = <.short or full url.> }

        let mut response = Response::new(Body::empty());

        match (req.method(), req.uri().path()) {
            (&Method::POST, url) => {
                // TODO validate url?
                // TODO check if url is already shortened?

                // we still need a key-value store to be able find already existing url

                // TODO calc hash
                // TODO check if url already exists
                //
                // TODO generate hash and save url
                // TODO return generated url

                let mut s = DefaultHasher::new();
                url.hash(&mut s);
                let hashcode = s.finish();

                *response.body_mut() = Body::from(format!("{}", hashcode));
            },
            (&Method::GET, url) => {
                // TODO look up short/full url by `url`
                *response.status_mut() = StatusCode::NOT_FOUND;
            },
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
            },
        }

        let svc = self.clone();

        return Box::pin(async move {
            let r = svc.get_value("test").await.unwrap();
            // Ok(Response::builder().body(Body::from("Hey".to_string())).unwrap())
            Ok(response)
        });

    }
}

