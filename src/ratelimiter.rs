use tokio::sync::{Mutex, MutexGuard};
use tokio::time::Instant;

use std::future::Future;
use std::time::Duration;
use std::sync::Arc;



#[derive(Debug, Clone)]
pub struct RateLimiter {
  inner: Arc<RateLimiterInner>
}

impl RateLimiter {
  pub fn new(delay: Duration) -> Self {
    RateLimiter {
      inner: Arc::new(RateLimiterInner {
        delay, deadline: Mutex::new(None)
      })
    }
  }

  /// Waits to acquire a lock and to exhaust the current deadline.
  pub async fn get(&self) -> TimeSlice {
    let mut guard = self.inner.deadline.lock().await;
    if let Some(deadline) = guard.take() {
      tokio::time::sleep_until(deadline).await;
    };

    TimeSlice { guard }
  }

  pub async fn get_consume<F, Fut, R>(&self, operation: F) -> R
  where F: FnOnce() -> Fut, Fut: Future<Output = R> {
    self.get().await.consume(operation()).await
  }
}

#[derive(Debug)]
struct RateLimiterInner {
  delay: Duration,
  deadline: Mutex<Option<Instant>>
}

pub struct TimeSlice<'a> {
  guard: MutexGuard<'a, Option<Instant>>
}

impl<'a> TimeSlice<'a> {
  /// Destroys this timeline, resetting it after the given future completes.
  pub async fn consume<Fut, R>(mut self, operation: Fut) -> R
  where Fut: Future<Output = R> {
    let ret = operation.await;
    *self.guard = Some(Instant::now());
    std::mem::drop(self.guard);
    ret
  }
}
