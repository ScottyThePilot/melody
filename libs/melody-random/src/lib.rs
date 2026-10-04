pub extern crate rand;

use rand::distr::uniform::{SampleBorrow, SampleUniform};
use rand::distr::weighted::Weight;
use rand::seq::{IndexedMutRandom, IndexedRandom, SliceChooseIter, SliceRandom, WeightError};

use std::ops::DerefMut;



pub type RandomUtilsIter<'a, T> = SliceChooseIter<'a, <T as RandomUtils>::Slice, <T as RandomUtils>::Output>;

pub trait RandomUtils {
  type Slice: ?Sized;
  type Output: ?Sized;

  fn shuffle_default(&mut self);

  fn choose_default(&self) -> Option<&Self::Output>;

  fn choose_mut_default(&mut self) -> Option<&mut Self::Output>;

  fn choose_multiple_default(&self, amount: usize) -> RandomUtilsIter<'_, Self>
  where
    Self::Output: Sized;

  fn choose_multiple_array_default<const N: usize>(&self) -> Option<[Self::Output; N]>
  where
    Self::Output: Clone + Sized;

  fn choose_multiple_weighted_default<F, X>(&self, amount: usize, weight: F) -> Result<RandomUtilsIter<'_, Self>, WeightError>
  where
    Self::Output: Sized,
    F: Fn(&Self::Output) -> X,
    X: Into<f64>;

  fn choose_weighted_default<F, B, X>(&self, weight: F) -> Result<&Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>;

  fn choose_weighted_mut_default<F, B, X>(&mut self, weight: F) -> Result<&mut Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>;
}

impl<T> RandomUtils for T
where T: DerefMut, T::Target: RandomUtils {
  type Slice = <T::Target as RandomUtils>::Slice;
  type Output = <T::Target as RandomUtils>::Output;

  #[inline]
  fn shuffle_default(&mut self) {
    <T::Target>::shuffle_default(self)
  }

  #[inline]
  fn choose_default(&self) -> Option<&Self::Output> {
    <T::Target>::choose_default(self)
  }

  #[inline]
  fn choose_mut_default(&mut self) -> Option<&mut Self::Output> {
    <T::Target>::choose_mut_default(self)
  }

  #[inline]
  fn choose_multiple_default(&self, amount: usize) -> RandomUtilsIter<'_, Self>
  where Self::Output: Sized {
    <T::Target>::choose_multiple_default(self, amount)
  }

  #[inline]
  fn choose_multiple_array_default<const N: usize>(&self) -> Option<[Self::Output; N]>
  where Self::Output: Clone + Sized {
    <T::Target>::choose_multiple_array_default(self)
  }

  #[inline]
  fn choose_multiple_weighted_default<F, X>(&self, amount: usize, weight: F) -> Result<RandomUtilsIter<'_, Self>, WeightError>
  where Self::Output: Sized, F: Fn(&Self::Output) -> X, X: Into<f64> {
    <T::Target>::choose_multiple_weighted_default(self, amount, weight)
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
  fn choose_weighted_mut_default<F, B, X>(&mut self, weight: F) -> Result<&mut Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>
  {
    <T::Target>::choose_weighted_mut_default(self, weight)
  }
}

impl<T> RandomUtils for [T] {
  type Slice = Self;
  type Output = T;

  #[inline]
  fn shuffle_default(&mut self) {
    SliceRandom::shuffle(self, &mut rand::rng());
  }

  #[inline]
  fn choose_default(&self) -> Option<&Self::Output> {
    IndexedRandom::choose(self, &mut rand::rng())
  }

  #[inline]
  fn choose_mut_default(&mut self) -> Option<&mut Self::Output> {
    IndexedMutRandom::choose_mut(self, &mut rand::rng())
  }

  #[inline]
  fn choose_multiple_default(&self, amount: usize) -> SliceChooseIter<'_, Self, Self::Output>
  where Self::Output: Sized {
    IndexedRandom::choose_multiple(self, &mut rand::rng(), amount)
  }

  #[inline]
  fn choose_multiple_array_default<const N: usize>(&self) -> Option<[Self::Output; N]>
  where Self::Output: Clone + Sized {
    IndexedRandom::choose_multiple_array(self, &mut rand::rng())
  }

  #[inline]
  fn choose_multiple_weighted_default<F, X>(&self, amount: usize, weight: F) -> Result<SliceChooseIter<'_, Self, Self::Output>, WeightError>
  where Self::Output: Sized, F: Fn(&Self::Output) -> X, X: Into<f64> {
    IndexedRandom::choose_multiple_weighted(self, &mut rand::rng(), amount, weight)
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
  fn choose_weighted_mut_default<F, B, X>(&mut self, weight: F) -> Result<&mut Self::Output, WeightError>
  where
    F: Fn(&Self::Output) -> B,
    B: SampleBorrow<X>,
    X: SampleUniform + Weight + PartialOrd<X>
  {
    IndexedMutRandom::choose_weighted_mut(self, &mut rand::rng(), weight)
  }
}
