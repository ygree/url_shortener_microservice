use hyper::service::Service;
use hyper::Server;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use log::{info, error};

mod kvservice;
mod uniqueid;
mod urlshortener;

use kvservice::KVService;
use urlshortener::UrlShortener;
use uniqueid::UniqueIdGen;

#[tokio::main]
async fn main() {
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

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        error!("server error: {}", e);
    }
}

async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
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
