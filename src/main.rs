use std::convert::Infallible;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use futures::future::BoxFuture;

mod kvservice;
mod inmem_kvstore;

use kvservice::KVService;

//TODO rename project
//TODO publish to github
type Counter = i32;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let addr = ([127, 0, 0, 1], 3000).into();
    let kv_service = KVService::new();

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
                kv_service: kv_service.clone()
            }
        );

    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}

//TODO replace it with a makeservice_fn
struct MakeSvc {
    kv_service: KVService,
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
        let fut = async move { Ok(Svc { kv_service }) };
        Box::pin(fut)
    }
}

#[derive(Clone)]
struct Svc {
    kv_service: KVService,
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
            (&Method::GET, "/") => {
                *response.body_mut() = Body::from("Try POSTing data to /echo");
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

