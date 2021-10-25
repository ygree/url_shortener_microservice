use std::task::{Context, Poll};
use futures::future::BoxFuture;
use hash_ids::HashIds;
use hyper::{Body, Method, Request, Response, StatusCode};
use hyper::service::Service;
use log::debug;

use crate::kvservice::{GetByKey, KVService, Put};
use crate::uniqueid::{GetUniqueId, UniqueId, UniqueIdGen};

#[derive(Clone)]
pub struct UrlShortener {
    pub kv_service: KVService,
    pub unique_id_gen: UniqueIdGen,
    pub hash_ids: HashIds,
}

impl UrlShortener {
    pub fn new(kv_service: KVService, unique_id_gen: UniqueIdGen) -> UrlShortener {
        let hash_ids = HashIds::builder()
            .with_salt("Arbitrary string")
            .finish();

        UrlShortener {
            kv_service,
            unique_id_gen,
            hash_ids,
        }
    }
}

impl Service<Request<Body>> for UrlShortener {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut response = Response::new(Body::empty());

        let url = req.uri().path().to_string();
        match (req.method(), url) {
            (&Method::POST, url) => {
                // TODO add caching if needed
                //  look up in cache and return if exists
                //  save in cache in the end
                //  could be implemented as a separate service

                let mut kv_service = self.kv_service.clone();
                let mut unique_id_gen = self.unique_id_gen.clone();
                let hash_ids = self.hash_ids.clone();

                Box::pin(async move {
                    // TODO improve error handling
                    let found_short_or_orig_url = kv_service.call(GetByKey(url.clone())).await.unwrap();
                    if let Some(found_url) = found_short_or_orig_url.clone() {
                        debug!("Look up from the KV-store: {} by {}", found_url, url);
                        Ok(Response::builder().body(Body::from(found_url)).unwrap())
                    } else {
                        // generate a new unique id, if short url not found
                        let UniqueId(unique_id) = unique_id_gen.call(GetUniqueId).await.unwrap();
                        let mut new_short_url = String::new();
                        new_short_url.push_str("/");
                        new_short_url.push_str(&hash_ids.encode(&vec![unique_id as u64]));
                        debug!("Generate new short_url: {} for {}", new_short_url, url);

                        // store new pairs long_url -> short_url and short_url -> long_url
                        // NOTE: we could potentially replace long_url -> short_url pair, but it's not an issue
                        // old url is stored as short_url -> long_url and will still work,
                        // service will advertise a last written short_url, all short_urls will still work
                        kv_service.call(Put::new(new_short_url.clone(), url.clone())).await.unwrap();
                        debug!("Store pair into the KV-store: {} {}", new_short_url, url);
                        kv_service.call(Put::new(url.clone(), new_short_url.clone())).await.unwrap();
                        debug!("Store pair into the KV-store: {} {}", url, new_short_url);

                        Ok(Response::builder().body(Body::from(new_short_url)).unwrap())
                    }
                })
            },
            (&Method::GET, url) => {
                let mut kv_service = self.kv_service.clone();

                Box::pin(async move {
                    // look up short/original url by `url`
                    let found_short_or_orig_url = kv_service.call(GetByKey(url.to_string())).await.unwrap();
                    debug!("Look up from the KV-store: {} by {}", found_short_or_orig_url.clone().unwrap_or("Not found!".to_string()), url.clone());
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

