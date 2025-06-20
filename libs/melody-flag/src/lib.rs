use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};



#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct Flag(Arc<AtomicBool>);

impl Flag {
  pub fn new(state: bool) -> Self {
    Flag(Arc::new(AtomicBool::new(state)))
  }

  pub fn get(&self) -> bool {
    self.0.load(Ordering::Relaxed)
  }

  pub fn set(&self, state: bool) {
    self.0.store(state, Ordering::Relaxed);
  }

  pub fn swap(&self, state: bool) -> bool {
    self.0.swap(state, Ordering::Relaxed)
  }
}
