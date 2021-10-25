use std::collections::hash_map::DefaultHasher;
use std::convert::Infallible;
use hyper::service::Service;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

use std::future::Future;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll};
use futures::future::BoxFuture;
use hash_ids::HashIds;

mod kvservice;
mod inmem_kvstore;
mod uniqueid;

use kvservice::KVService;
use crate::kvservice::{GetByKey, Put};
use crate::uniqueid::{GetUniqueId, UniqueId, UniqueIdGen};

//TODO add logging
//TODO rename project
//TODO publish to github

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let addr = ([127, 0, 0, 1], 3000).into();
    let kv_service = KVService::new();
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
    type Response = Svc;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let kv_service = self.kv_service.clone();
        let unique_id_gen = self.unique_id_gen.clone();
        let hash_ids = HashIds::builder()
            .with_salt("Arbitrary string")
            .finish();
        let fut = async move { Ok(Svc { kv_service, unique_id_gen, hash_ids }) };
        Box::pin(fut)
    }
}

#[derive(Clone)]
struct Svc {
    kv_service: KVService,
    unique_id_gen: UniqueIdGen,
    hash_ids: HashIds,
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

        match (req.method(), req.uri().path().to_string()) {
            (&Method::POST, url) => { // PUT? does put has a return value?
                // TODO get url from the body

                // TODO validate url?

                // TODO add caching if needed
                // TODO look up in cache and return if exists
                // TODO save in cache in the end

                let mut kv_service = self.kv_service.clone();
                let mut unique_id_gen = self.unique_id_gen.clone();
                let mut hash_ids = self.hash_ids.clone();

                Box::pin(async move {
                    let found_short_or_orig_url = kv_service.call(GetByKey(url.clone())).await.unwrap();
                    if let Some(found_url) = found_short_or_orig_url.clone() {
                        println!("Look up from the KV-store: {} by {}", found_url, url);
                        Ok(Response::builder().body(Body::from(found_url)).unwrap())
                    } else {
                        // generate a new unique id, if short url not found
                        let UniqueId(unique_id) = unique_id_gen.call(GetUniqueId).await.unwrap();
                        let new_short_url = hash_ids.encode(&vec![unique_id as u64]);
                        println!("Generate new short_url: {} for {}", new_short_url, url);

                        // store new pairs long_url -> short_url and short_url -> long_url
                        // NOTE: we could potentially replace long_url -> short_url pair, but it's not an issue
                        // old url is stored as short_url -> long_url and will still work,
                        // service will advertise a last written short_url, all short_urls will still work
                        kv_service.call(Put::new(new_short_url.clone(), url.clone())).await;
                        println!("Store pair {} {} into the KV-store", new_short_url, url);
                        kv_service.call(Put::new(url.clone(), new_short_url.clone())).await;
                        println!("Store pair {} {} into the KV-store", url, new_short_url);

                        Ok(Response::builder().body(Body::from(new_short_url)).unwrap())
                    }
                })
            },
            (&Method::GET, url) => {
                let mut kv_service = self.kv_service.clone();

                Box::pin(async move {
                    // look up short/original url by `url`
                    let found_short_or_orig_url = kv_service.call(GetByKey(url.clone())).await.unwrap();
                    println!("Look up from the KV-store {} by {}", found_short_or_orig_url.clone().unwrap_or("Not found!".to_string()), url.clone());
                    if let Some(short_or_orig_url) = found_short_or_orig_url {
                        Ok(Response::builder().body(Body::from(short_or_orig_url)).unwrap())
                    } else {
                        *response.status_mut() = StatusCode::NOT_FOUND;
                        Ok(response)
                    }
                })
            },
            _ => {
                *response.status_mut() = StatusCode::NOT_FOUND;
                Box::pin(async move {
                    Ok(response)
                })
            },
        }
    }
}

