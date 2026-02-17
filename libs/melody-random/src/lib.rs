pub extern crate rand;
pub extern crate rand_chacha;
pub extern crate rand_xoshiro;

use rand::{CryptoRng, RngCore, TryRngCore, SeedableRng};
use rand::distr::uniform::{SampleBorrow, SampleUniform};
use rand::distr::weighted::Weight;
use rand::seq::{IndexedMutRandom, IndexedRandom, SliceChooseIter, SliceRandom, WeightError};
use rand::rngs::{OsRng, ReseedingRng};
use rand_chacha::{ChaCha12Core, ChaCha12Rng};
use rand_xoshiro::Xoshiro256PlusPlus;

use std::ops::DerefMut;
use std::cell::RefCell;
use std::thread::LocalKey;



pub type SecureThreadRng = ThreadRng<SecureRng>;
pub type SecureReseedingThreadRng = ThreadRng<SecureReseedingRng>;

const _: () = assert_is_threadsafe::<ThreadRng<SecureRng>>();
const _: () = assert_is_threadsafe::<ThreadRng<SecureReseedingRng>>();

#[macro_export]
macro_rules! impl_thread_local {
  (for $Type:ty, $expr:expr) => (
    impl $crate::ThreadLocal for $Type {
      fn get_key() -> &'static $crate::LocalKeyCell<Self> {
        std::thread_local!{
          static KEY: std::cell::RefCell<$Type> = {
            std::cell::RefCell::new($expr)
          };
        }

        &KEY
      }
    }
  );
}

pub type LocalKeyCell<T> = LocalKey<RefCell<T>>;

pub trait ThreadLocal: Sized + 'static {
  fn get_key() -> &'static LocalKeyCell<Self>;
}

pub struct ThreadRng<T: 'static> {
  inner: &'static LocalKeyCell<T>
}

impl<T: ThreadLocal> ThreadRng<T> {
  pub fn new() -> Self {
    ThreadRng { inner: T::get_key() }
  }
}

impl<T: ThreadLocal + RngCore> RngCore for ThreadRng<T> {
  #[inline(always)]
  fn next_u32(&mut self) -> u32 {
    self.inner.with_borrow_mut(RngCore::next_u32)
  }

  #[inline(always)]
  fn next_u64(&mut self) -> u64 {
    self.inner.with_borrow_mut(RngCore::next_u64)
  }

  fn fill_bytes(&mut self, dst: &mut [u8]) {
    self.inner.with_borrow_mut(|inner| inner.fill_bytes(dst))
  }
}

impl<T: ThreadLocal + CryptoRng> CryptoRng for ThreadRng<T> {}



const _: () = assert_is_threadsafe::<SecureRng>();
const _: () = assert_is_crypto_rng::<ChaCha12Rng>();

pub type SecureRngSeed = <ChaCha12Rng as SeedableRng>::Seed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecureRng {
  inner: ChaCha12Rng
}

impl SecureRng {
  pub fn new() -> Self {
    Self::from_os_rng()
  }
}

impl SeedableRng for SecureRng {
  type Seed = SecureRngSeed;

  fn from_seed(seed: Self::Seed) -> Self {
    SecureRng { inner: ChaCha12Rng::from_seed(seed) }
  }
}

impl RngCore for SecureRng {
  #[inline(always)]
  fn next_u32(&mut self) -> u32 {
    self.inner.next_u32()
  }

  #[inline(always)]
  fn next_u64(&mut self) -> u64 {
    self.inner.next_u64()
  }

  fn fill_bytes(&mut self, dst: &mut [u8]) {
    self.inner.fill_bytes(dst);
  }
}

impl CryptoRng for SecureRng {}

impl_thread_local!(for SecureRng, SecureRng::new());



const _: () = assert_is_threadsafe::<SecureReseedingRng>();
const _: () = assert_is_crypto_rng::<ReseedingRng<ChaCha12Core, OsRng>>();

pub type SecureReseedingRngError = <OsRng as TryRngCore>::Error;

#[derive(Debug, Clone)]
pub struct SecureReseedingRng {
  inner: ReseedingRng<ChaCha12Core, OsRng>
}

impl SecureReseedingRng {
  pub const THRESHOLD: u64 = 1024 * 64;

  pub fn new() -> Result<Self, SecureReseedingRngError> {
    Self::with_threshold(Self::THRESHOLD)
  }

  pub fn with_threshold(threshold: u64) -> Result<Self, SecureReseedingRngError> {
    Ok(SecureReseedingRng { inner: ReseedingRng::new(threshold, OsRng)? })
  }

  pub fn reseed(&mut self) -> Result<(), SecureReseedingRngError> {
    self.inner.reseed()
  }
}

impl RngCore for SecureReseedingRng {
  #[inline(always)]
  fn next_u32(&mut self) -> u32 {
    self.inner.next_u32()
  }

  #[inline(always)]
  fn next_u64(&mut self) -> u64 {
    self.inner.next_u64()
  }

  fn fill_bytes(&mut self, dst: &mut [u8]) {
    self.inner.fill_bytes(dst);
  }
}

impl CryptoRng for SecureReseedingRng {}

impl_thread_local!(for SecureReseedingRng, {
  SecureReseedingRng::new().unwrap()
});



const _: () = assert_is_threadsafe::<SmallRng>();

pub type SmallRngSeed = <Xoshiro256PlusPlus as SeedableRng>::Seed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmallRng {
  inner: Xoshiro256PlusPlus
}

impl SmallRng {
  pub fn new() -> Self {
    SmallRng::from_rng(&mut rand::rng())
  }
}

impl SeedableRng for SmallRng {
  type Seed = SmallRngSeed;

  fn from_seed(seed: Self::Seed) -> Self {
    SmallRng { inner: Xoshiro256PlusPlus::from_seed(seed) }
  }

  fn seed_from_u64(state: u64) -> Self {
    SmallRng { inner: Xoshiro256PlusPlus::seed_from_u64(state) }
  }
}

impl RngCore for SmallRng {
  #[inline(always)]
  fn next_u32(&mut self) -> u32 {
    self.inner.next_u32()
  }

  #[inline(always)]
  fn next_u64(&mut self) -> u64 {
    self.inner.next_u64()
  }

  fn fill_bytes(&mut self, dst: &mut [u8]) {
    self.inner.fill_bytes(dst);
  }
}



const fn assert_is_threadsafe<T: Send + Sync + 'static>() {}
const fn assert_is_crypto_rng<T: CryptoRng>() {}



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
