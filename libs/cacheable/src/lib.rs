extern crate async_trait;

pub use async_trait::async_trait;

use std::convert::Infallible;

pub struct Cache<S> {
  state: Option<S>
}

impl<S> Cache<S> {
  pub const fn new() -> Self {
    Cache { state: None }
  }
}

impl<S: Cacheable> Cache<S> {
  pub fn get(&mut self, args: S::Args<'_>) -> &mut S
  where S: Cacheable<Error = Infallible> {
    match self.try_get(args) {
      Ok(state) => state,
      Err(i) => match i {}
    }
  }

  pub fn try_get(&mut self, args: S::Args<'_>) -> Result<&mut S, S::Error> {
    let state = match self.state {
      None => S::operation(args),
      Some(ref state) if state.is_invalid(&args) => {
        self.state = None;
        S::operation(args)
      },
      Some(ref mut state) => return Ok(state)
    }?;

    Ok(self.state.insert(state))
  }
}

impl<S: CacheableAsync> Cache<S> {
  pub async fn get_async(&mut self, args: S::Args<'_>) -> &mut S
  where S: CacheableAsync<Error = Infallible> {
    match self.try_get_async(args).await {
      Ok(state) => state,
      Err(i) => match i {}
    }
  }

  pub async fn try_get_async(&mut self, args: S::Args<'_>) -> Result<&mut S, S::Error> {
    let state = match self.state {
      None => S::operation(args).await,
      Some(ref state) if state.is_invalid(&args) => {
        self.state = None;
        S::operation(args).await
      },
      Some(ref mut state) => return Ok(state)
    }?;

    Ok(self.state.insert(state))
  }
}

pub trait Cacheable: Sized {
  type Args<'a>;
  type Error;

  fn operation(args: Self::Args<'_>) -> Result<Self, Self::Error>;
  fn is_invalid(&self, args: &Self::Args<'_>) -> bool;
}

#[async_trait]
pub trait CacheableAsync: Sized {
  type Args<'a>;
  type Error;

  async fn operation(args: Self::Args<'_>) -> Result<Self, Self::Error>;
  fn is_invalid(&self, args: &Self::Args<'_>) -> bool;
}
