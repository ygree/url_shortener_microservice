use std::convert::Infallible;
use std::future::{Future, Ready};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Context;
use futures::future::BoxFuture;
use hyper::service::Service;
use tokio::macros::support::Poll;
use std::sync::atomic::{AtomicUsize, Ordering};
use atomic_counter::{AtomicCounter, RelaxedCounter};

#[derive(Clone)]
pub struct UniqueIdGen {
    counter: Arc<RelaxedCounter>,
}

impl UniqueIdGen {
    pub fn new() -> UniqueIdGen {
        UniqueIdGen {
            counter: Arc::new(RelaxedCounter::new(0))
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
        let id = self.counter.inc();
        core::future::ready(Ok(id))
    }
}
