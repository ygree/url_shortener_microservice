use std::error::Error;
use std::process::Command;
use std::thread;
use std::time::Duration;
use assert_cmd::prelude::*;
use hyper::{Body, Method, Client, Request};
use reqwest::StatusCode;

#[tokio::test]
async fn test_service() -> Result<(), Box<dyn Error>> {
    let mut cmd = Command::cargo_bin("url-shortener-microservice")?;
    let mut svc_handle = cmd.env("RUST_LOG", "url_shortener_microservice=debug").spawn().unwrap();

    thread::sleep(Duration::from_millis(200));

    let client = Client::new();

    let orig_url = "http://localhost:3000/www.youtube.com/watch?v=lL9zveDz8H12";

    let req = Request::builder()
        .method(Method::GET)
        .uri(orig_url)
        .body(Body::from(""))
        .unwrap();

    let res = client.request(req).await?;
    let first_status = res.status();

    let req = Request::builder()
        .method(Method::POST)
        .uri(orig_url)
        .body(Body::from(""))
        .unwrap();

    let res = client.request(req).await?;
    let second_status = res.status();

    svc_handle.kill().unwrap();


    assert_eq!(first_status, StatusCode::NOT_FOUND);
    assert_eq!(second_status, StatusCode::OK);

    Ok(())
}
