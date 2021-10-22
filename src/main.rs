use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn get_by_key() -> Result<String, Infallible> {
    Ok("".into())
}

async fn get_from_cache() -> Result<String, Infallible> {
    Ok("Hello world!".into())
}

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // let a = get_from_cache().map(|v| { v + ", World!"})
    match get_from_cache().await {
        Result::Ok(s) => Ok(Response::new(s.into())),
        _ => Ok(Response::new("Not found!".into()))
    }   
}

#[tokio::main]
async fn main() {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

