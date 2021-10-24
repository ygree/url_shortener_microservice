mod kvstore;

use hyper::service::Service;
use hyper::{Body, Request, Response, Server};

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};

use kvstore::KVStore;


type Counter = i32;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kv_store = KVStore::new();

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr)
        .serve(
            MakeSvc {
                counter: Arc::new(Mutex::new(81818)),
            }
        );

    println!("Listening on http://{}", addr);

    server.await?;
    Ok(())
}

struct MakeSvc {
    counter: Arc<Mutex<Counter>>,
}

impl<T> Service<T> for MakeSvc {
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let counter = self.counter.clone();
        let fut = async move { Ok(Svc { counter }) };
        Box::pin(fut)
    }
}

struct Svc {
    counter: Arc<Mutex<Counter>>,
}

impl Service<Request<Body>> for Svc {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        fn mk_response(s: String) -> Result<Response<Body>, hyper::Error> {
            Ok(Response::builder().body(Body::from(s)).unwrap())
        }

        let res = match req.uri().path() {
            "/" => mk_response(format!("home! counter = {:?}", self.counter)),
            "/posts" => mk_response(format!("posts, of course! counter = {:?}", self.counter)),
            "/authors" => mk_response(format!(
                "authors extraordinare! counter = {:?}",
                self.counter
            )),
            // Return the 404 Not Found for other routes, and don't increment counter.
            _ => return Box::pin(async { mk_response("oh no! not found".into()) }),
        };

        if req.uri().path() != "/favicon.ico" {
            let mut c = self.counter.lock().unwrap();
            *c += 1;

        }

        Box::pin(async { res })
    }
}

struct KVService {
    kv_store: KVStore,
}

enum KVServiceRequest {
    Put { key: String, value: String },
    Get(String),
}

impl Service<KVServiceRequest> for KVService {
    type Response = Option<String>;
    type Error = ();
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: KVServiceRequest) -> Self::Future {
        match req {
            KVServiceRequest::Get(key) => {
                let value = self.kv_store.get(key);
                Box::pin(async { Ok(value) })
            }
            KVServiceRequest::Put {key, value} => {
                Box::pin(async { Ok(None) })
            }
        }
    }
}