use hyper::service::Service;
use hyper::Server;

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use hyper::server::conn::AddrIncoming;
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

    info!("Listening on http://{}", addr);

    let graceful = start_server(&addr).await
        .with_graceful_shutdown(shutdown_signal());

    if let Err(e) = graceful.await {
        error!("server error: {}", e);
    }
}

async fn start_server(addr: &SocketAddr) -> Server<AddrIncoming, MakeSvc> {
    // KV-store mock impl
    let kv_service = KVService::new();
    // unique id gen mock impl
    let unique_id_gen = UniqueIdGen::new();

    Server::bind(addr)
        .serve(
            MakeSvc {
                kv_service,
                unique_id_gen
            }
        )
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

// #[tokio::test]
// async fn test_service() {
//     use hyper::Client;
//
//     let port = 3000;
//     let addr = ([127, 0, 0, 1], port).into();
//
//     // let server = start_server(&addr).await;
//     let handle = tokio::spawn(async move {
//         start_server(&addr).await;
//     });
//
//     handle.await;
//
//     thread::sleep(Duration::from_millis(4000));
//
//     // println!("test {}", server.local_addr().port());
//
//     let client = Client::new();
//
//     let uri: Uri = format!("localhost:{}", port).parse().unwrap();
//
//     println!("Request: {}", uri);
//
//     let resp = client.get(uri).await.unwrap();
//     // assert_eq!(resp.status(), 200);
//     // println!("Response: {}", resp.status());
//
//     // spawn()
//     // main().await;
//     // setup_wiremock().await;
//     // let r = router(http::Client::new(), init_db().await);
//     // let resp = request()
//     //     .path("/todo")
//     //     .method("POST")
//     //     .body("")
//     //     .reply(&r)
//     //     .await;
//     // assert_eq!(resp.status(), 200);
//     // assert_eq!(
//     //     resp.body(),
//     //     r#"{"id":1,"name":"wiremock cat fact","checked":false}"#
//     // );
//     //
//     // let resp = request().path("/todo").reply(&r).await;
//     // assert_eq!(resp.status(), 200);
//     // assert_eq!(
//     //     resp.body(),
//     //     r#"[{"id":1,"name":"wiremock cat fact","checked":false}]"#
//     // );
// }