use tokio::sync::{Mutex, MutexGuard};
use tokio::time::Instant;

use std::ops::{Deref, DerefMut};
use std::time::Duration;
use std::sync::Arc;

#[derive(Debug)]
pub struct RateLimiter<T> {
  inner: Arc<RateLimiterInner<T>>
}

impl<T> RateLimiter<T> {
  pub fn new(value: T, delay: Duration) -> Self {
    RateLimiter {
      inner: Arc::new(RateLimiterInner {
        resource: Mutex::new(Resource {
          value, deadline: Instant::now()
        }),
        delay
      })
    }
  }

  pub async fn get(&self) -> TimeSlice<T> {
    let guard = self.inner.resource.lock().await;
    tokio::time::sleep_until(guard.deadline).await;
    TimeSlice { guard, delay: self.inner.delay }
  }

  pub fn try_get(&self) -> Option<TimeSlice<T>> {
    let guard = self.inner.resource.try_lock().ok()?;
    if guard.deadline.elapsed() <= Duration::ZERO {
      Some(TimeSlice { guard, delay: self.inner.delay })
    } else {
      None
    }
  }
}

impl<T> Clone for RateLimiter<T> {
  fn clone(&self) -> Self {
    RateLimiter { inner: self.inner.clone() }
  }
}

#[derive(Debug)]
struct RateLimiterInner<T> {
  resource: Mutex<Resource<T>>,
  delay: Duration
}

#[derive(Debug)]
struct Resource<T> {
  value: T,
  deadline: Instant
}

#[derive(Debug)]
pub struct TimeSlice<'t, T> {
  guard: MutexGuard<'t, Resource<T>>,
  delay: Duration
}

impl<'t, T> Deref for TimeSlice<'t, T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.guard.value
  }
}

impl<'t, T> DerefMut for TimeSlice<'t, T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.guard.value
  }
}

impl<'t, T> Drop for TimeSlice<'t, T> {
  fn drop(&mut self) {
    self.guard.deadline = Instant::now() + self.delay;
  }
}
