pub extern crate rand;

use rand::SeedableRng;
use rand::distr::uniform::{SampleBorrow, SampleUniform};
use rand::distr::weighted::Weight;
use rand::rngs::{SysRng, SysError};
use rand::seq::{IndexedMutRandom, IndexedRandom, IndexedSamples, SliceRandom, WeightError};

use std::ops::{Deref, DerefMut};



pub trait SeedableRngUtils: SeedableRng {
  fn from_sys_rng() -> Self {
    Self::try_from_sys_rng().expect("try_from_sys_rng failed")
  }

  fn try_from_sys_rng() -> Result<Self, SysError>;
}

impl<R> SeedableRngUtils for R
where R: ?Sized + SeedableRng {
  fn try_from_sys_rng() -> Result<Self, SysError> {
    Self::try_from_rng(&mut SysRng)
  }
}

pub trait SliceRandomUtils {
  type Output: ?Sized;

  fn choose_default(&self) -> Option<&Self::Output>;

  fn choose_weighted_default<F, B, X>(&self, weight: F) -> Result<&Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>;

  fn sample_default(&self, amount: usize) -> IndexedSamples<'_, [Self::Output], Self::Output>
  where Self::Output: Sized;

  fn sample_array_default<const N: usize>(&self) -> Option<[Self::Output; N]>
  where Self::Output: Clone + Sized;

  fn sample_weighted_default<F, X>(&self, amount: usize, weight: F) -> Result<IndexedSamples<'_, [Self::Output], Self::Output>, WeightError>
  where
    Self::Output: Sized,
    F: Fn(&Self::Output) -> X,
    X: Into<f64>;
}

pub trait SliceMutRandomUtils: SliceRandomUtils {
  fn shuffle_default(&mut self);

  fn partial_shuffle_default(&mut self, amount: usize) -> (&mut [Self::Output], &mut [Self::Output])
  where Self::Output: Sized;

  fn choose_mut_default(&mut self) -> Option<&mut Self::Output>;

  fn choose_weighted_mut_default<F, B, X>(&mut self, weight: F) -> Result<&mut Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>;
}

impl<T> SliceRandomUtils for T
where T: Deref, T::Target: SliceRandomUtils {
  type Output = <T::Target as SliceRandomUtils>::Output;

  #[inline]
  fn choose_default(&self) -> Option<&Self::Output> {
    <T::Target>::choose_default(self)
  }

  #[inline]
  fn choose_weighted_default<F, B, X>(&self, weight: F) -> Result<&Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>
  {
    <T::Target>::choose_weighted_default(self, weight)
  }

  #[inline]
  fn sample_default(&self, amount: usize) -> IndexedSamples<'_, [Self::Output], Self::Output>
  where Self::Output: Sized {
    <T::Target>::sample_default(self, amount)
  }

  #[inline]
  fn sample_array_default<const N: usize>(&self) -> Option<[Self::Output; N]>
  where Self::Output: Clone + Sized {
    <T::Target>::sample_array_default(self)
  }

  #[inline]
  fn sample_weighted_default<F, X>(&self, amount: usize, weight: F) -> Result<IndexedSamples<'_, [Self::Output], Self::Output>, WeightError>
  where Self::Output: Sized, F: Fn(&Self::Output) -> X, X: Into<f64> {
    <T::Target>::sample_weighted_default(self, amount, weight)
  }
}

impl<T> SliceMutRandomUtils for T
where T: DerefMut, T::Target: SliceMutRandomUtils {
  #[inline]
  fn shuffle_default(&mut self) {
    <T::Target>::shuffle_default(self)
  }

  #[inline]
  fn partial_shuffle_default(&mut self, amount: usize) -> (&mut [Self::Output], &mut [Self::Output])
  where Self::Output: Sized {
    <T::Target>::partial_shuffle_default(self, amount)
  }

  #[inline]
  fn choose_mut_default(&mut self) -> Option<&mut Self::Output> {
    <T::Target>::choose_mut_default(self)
  }

  #[inline]
  fn choose_weighted_mut_default<F, B, X>(&mut self, weight: F) -> Result<&mut Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>
  {
    <T::Target>::choose_weighted_mut_default(self, weight)
  }
}

impl<T> SliceRandomUtils for [T] {
  type Output = T;

  #[inline]
  fn choose_default(&self) -> Option<&Self::Output> {
    IndexedRandom::choose(self, &mut rand::rng())
  }

  #[inline]
  fn choose_weighted_default<F, B, X>(&self, weight: F) -> Result<&Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>
  {
    IndexedRandom::choose_weighted(self, &mut rand::rng(), weight)
  }

  #[inline]
  fn sample_default(&self, amount: usize) -> IndexedSamples<'_, [Self::Output], Self::Output>
  where Self::Output: Sized {
    IndexedRandom::sample(self, &mut rand::rng(), amount)
  }

  #[inline]
  fn sample_array_default<const N: usize>(&self) -> Option<[Self::Output; N]>
  where Self::Output: Clone + Sized {
    IndexedRandom::sample_array(self, &mut rand::rng())
  }

  #[inline]
  fn sample_weighted_default<F, X>(&self, amount: usize, weight: F) -> Result<IndexedSamples<'_, [Self::Output], Self::Output>, WeightError>
  where Self::Output: Sized, F: Fn(&Self::Output) -> X, X: Into<f64> {
    IndexedRandom::sample_weighted(self, &mut rand::rng(), amount, weight)
  }
}

impl<T> SliceMutRandomUtils for [T] {
  #[inline]
  fn shuffle_default(&mut self) {
    SliceRandom::shuffle(self, &mut rand::rng());
  }

  #[inline]
  fn partial_shuffle_default(&mut self, amount: usize) -> (&mut [Self::Output], &mut [Self::Output])
  where Self::Output: Sized {
    SliceRandom::partial_shuffle(self, &mut rand::rng(), amount)
  }

  #[inline]
  fn choose_mut_default(&mut self) -> Option<&mut Self::Output> {
    IndexedMutRandom::choose_mut(self, &mut rand::rng())
  }

  #[inline]
  fn choose_weighted_mut_default<F, B, X>(&mut self, weight: F) -> Result<&mut Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>
  {
    IndexedMutRandom::choose_weighted_mut(self, &mut rand::rng(), weight)
  }
}
