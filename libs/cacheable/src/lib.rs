extern crate async_trait;

pub use async_trait::async_trait;

use std::convert::Infallible;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CacheStatus {
  Uncached,
  CachedValid,
  CachedInvalid
}

macro_rules! impl_cache_common_functions {
  ($Cache:ident<$S:ident>) => {
    #[inline]
    pub const fn new() -> Self {
      $Cache { state: None }
    }

    #[inline]
    pub const fn get_state(&self) -> Option<&$S> {
      self.state.as_ref()
    }

    #[inline]
    pub const fn get_state_mut(&mut self) -> Option<&mut $S> {
      self.state.as_mut()
    }
  };
  ($Cache:ident<$S:ident> @bound) => {
    pub fn get_status(&self, args: &$S::Args<'_>) -> CacheStatus {
      match self.get_state() {
        None => CacheStatus::Uncached,
        Some(state) => match state.is_invalid(args) {
          true => CacheStatus::CachedInvalid,
          false => CacheStatus::CachedValid
        }
      }
    }
  };
}

#[derive(Debug, Clone)]
pub struct Cache<S> {
  state: Option<S>
}

impl<S> Cache<S> {
  impl_cache_common_functions!(Cache<S>);
}

impl<S: Cacheable> Cache<S> {
  impl_cache_common_functions!(Cache<S> @bound);

  pub fn get_or_init(&mut self, args: S::Args<'_>) -> &mut S
  where S: Cacheable<Error = Infallible> {
    into_ok(self.try_get_or_init(args))
  }

  pub fn try_get_or_init(&mut self, args: S::Args<'_>) -> Result<&mut S, S::Error> {
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

#[derive(Debug, Clone)]
pub struct CacheAsync<S> {
  state: Option<S>
}

impl<S> CacheAsync<S> {
  impl_cache_common_functions!(CacheAsync<S>);
}

impl<S: CacheableAsync> CacheAsync<S> {
  impl_cache_common_functions!(CacheAsync<S> @bound);

  pub async fn get_or_init(&mut self, args: S::Args<'_>) -> &mut S
  where S: CacheableAsync<Error = Infallible> {
    into_ok(self.try_get_or_init(args).await)
  }

  pub async fn try_get_or_init(&mut self, args: S::Args<'_>) -> Result<&mut S, S::Error> {
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

#[inline]
fn into_ok<T>(result: Result<T, Infallible>) -> T {
  match result {
    Ok(value) => value,
    Err(i) => match i {}
  }
}
