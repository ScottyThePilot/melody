use crate::handler::MelodyHandler;

use indexmap::IndexMap;

use std::any::{Any, TypeId};
use std::fmt;



pub trait MelodyFeatureKey<S, E>: Any + Send + Sync
where S: Send + Sync, E: Send + Sync {
  type Value: MelodyFeature<S, E>;
}

pub trait MelodyFeature<S, E>: MelodyHandler<S, E> + Any + fmt::Debug
where S: Send + Sync, E: Send + Sync {}

impl<S, E, T> MelodyFeature<S, E> for T
where
  T: MelodyHandler<S, E> + Any + fmt::Debug,
  S: Send + Sync, E: Send + Sync
{}

impl<S, E> dyn MelodyFeature<S, E>
where S: Send + Sync + 'static, E: Send + Sync + 'static {
  #[inline]
  pub fn is<T: MelodyFeature<S, E>>(&self) -> bool {
    <dyn Any>::is::<T>(self)
  }

  #[inline]
  pub fn downcast_ref<T: MelodyFeature<S, E>>(&self) -> Option<&T> {
    <dyn Any>::downcast_ref::<T>(self)
  }

  #[inline]
  pub fn downcast_mut<T: MelodyFeature<S, E>>(&mut self) -> Option<&mut T> {
    <dyn Any>::downcast_mut::<T>(self)
  }

  #[inline]
  pub fn downcast<T: MelodyFeature<S, E>>(self: Box<Self>) -> Option<Box<T>> {
    <Box<dyn Any>>::downcast::<T>(self).ok()
  }
}



pub struct MelodyFeatureContainer<S, E> {
  map: IndexMap<TypeId, Box<dyn MelodyFeature<S, E>>>
}

impl<S, E> MelodyFeatureContainer<S, E>
where S: Send + Sync, E: Send + Sync {
  pub fn new() -> Self {
    MelodyFeatureContainer { map: IndexMap::new() }
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.map.len()
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  #[inline]
  pub fn clear(&mut self) {
    self.map.clear()
  }

  #[inline]
  pub fn contains_key<T: MelodyFeatureKey<S, E>>(&self) -> bool
  where T::Value: MelodyFeature<S, E> {
    self.map.contains_key(&TypeId::of::<T>())
  }

  #[inline]
  pub fn insert<T: MelodyFeatureKey<S, E>>(&mut self, value: T::Value)
  where T::Value: MelodyFeature<S, E> {
    self.map.insert(TypeId::of::<T>(), Box::new(value));
  }

  #[inline]
  pub fn get<T: MelodyFeatureKey<S, E>>(&self) -> Option<&T::Value>
  where T::Value: MelodyFeature<S, E>, S: 'static, E: 'static {
    self.map.get(&TypeId::of::<T>()).map(|value| value.downcast_ref().unwrap())
  }

  #[inline]
  pub fn get_mut<T: MelodyFeatureKey<S, E>>(&mut self) -> Option<&mut T::Value>
  where T::Value: MelodyFeature<S, E>, S: 'static, E: 'static {
    self.map.get_mut(&TypeId::of::<T>()).map(|value| value.downcast_mut().unwrap())
  }

  #[inline]
  pub fn remove<T: MelodyFeatureKey<S, E>>(&mut self) -> Option<T::Value>
  where T::Value: MelodyFeature<S, E>, S: 'static, E: 'static {
    self.map.shift_remove(&TypeId::of::<T>()).map(|value| *value.downcast().unwrap())
  }

  pub fn values(&self) -> impl Iterator<Item = &dyn MelodyFeature<S, E>> {
    self.map.values().map(|x| &**x)
  }

  pub fn into_values(self) -> impl Iterator<Item = Box<dyn MelodyFeature<S, E>>> {
    self.map.into_values()
  }
}

impl<S, E> fmt::Debug for MelodyFeatureContainer<S, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MelodyFeatureContainer")
      .field("map", &self.map)
      .finish()
  }
}
