use std::convert::Infallible;
use std::future::{Future, Ready};
use std::pin::Pin;
use std::task::Context;
use futures::future::BoxFuture;
use hyper::service::Service;
use tokio::macros::support::Poll;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct UniqueIdGen {
    counter: &'static AtomicUsize,
}

impl UniqueIdGen {
    pub fn new(init: &'static AtomicUsize) -> UniqueIdGen {
        UniqueIdGen {
            counter: init
        }
    }
}

impl Service<()> for UniqueIdGen {
    type Response = usize;
    type Error = Infallible;
    // type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;
    type Future = Ready<Result<Self::Response, Infallible>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: ()) -> Self::Future {
        // mock unique id source
        let id = self.counter.load(Ordering::Relaxed);
        core::future::ready(Ok(id))
    }
}
