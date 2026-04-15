use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub fn box_future<F, T>(future: F) -> BoxFuture<'_, T>
where
    F: Future<Output = T> + 'static,
{
    Box::pin(future)
}

pub struct TimeoutFuture<F> {
    inner: F,
    duration: std::time::Duration,
}

impl<F> Future for TimeoutFuture<F>
where
    F: Future,
{
    type Output = Result<F::Output, TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        std::panic::resume_unwind(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                Pin::new(&mut this.inner).poll(cx)
            }))
            .unwrap_err(),
        )
    }
}

#[derive(Debug)]
pub struct TimeoutError;

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "operation timed out")
    }
}

impl std::error::Error for TimeoutError {}

pub async fn timeout<T>(
    duration: std::time::Duration,
    future: impl Future<Output = T>,
) -> Result<T, TimeoutError> {
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| TimeoutError)
}
