use chrono::{DateTime, Utc};
use feed::model::Entry;
use reqwest::Client;
use tokio::sync::{Notify, RwLock, RwLockWriteGuard, RwLockReadGuard};
use tokio::task::JoinHandle;
use tokio::time::{sleep_until, Instant};
use url::Url;

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use std::ops::{Deref, DerefMut};

use crate::model::{HasDateTime, ModelError};



#[async_trait::async_trait]
pub trait Context<M: Model>: Send + Sync + 'static {
  type Error: From<crate::model::ModelError<<M::Entry as TryFrom<Entry>>::Error>> + Send;

  fn client(&self) -> &Client;
  fn on_manager_error(&self, error: Self::Error);
  async fn on_new_entries(&self, identifier: &M::Identifier, entries: Vec<M::Entry>);
  async fn save_update_datetime(&self, identifier: &M::Identifier, update: DateTime<Utc>) -> Result<(), Self::Error>;
  async fn load_update_datetime(&self, identifier: &M::Identifier) -> Result<Option<DateTime<Utc>>, Self::Error>;
}

#[async_trait::async_trait]
impl<T, M: Model> Context<M> for Arc<T> where T: Context<M> {
  type Error = T::Error;

  #[inline]
  fn client(&self) -> &Client {
    T::client(self)
  }

  #[inline]
  fn on_manager_error(&self, error: Self::Error) {
    T::on_manager_error(self, error)
  }

  #[inline]
  async fn on_new_entries(&self, identifier: &M::Identifier, entries: Vec<M::Entry>) {
    T::on_new_entries(self, identifier, entries).await
  }

  #[inline]
  async fn save_update_datetime(&self, identifier: &M::Identifier, update: DateTime<Utc>) -> Result<(), Self::Error> {
    T::save_update_datetime(self, identifier, update).await
  }

  #[inline]
  async fn load_update_datetime(&self, identifier: &M::Identifier) -> Result<Option<DateTime<Utc>>, Self::Error> {
    T::load_update_datetime(self, identifier).await
  }
}

pub trait Model: Send + Sync + 'static {
  type Identifier: std::fmt::Debug + Clone + PartialEq + Send + Sync + 'static;
  type Entry: TryFrom<Entry> + HasDateTime + Send + Sync + 'static;

  fn url(&self, identifier: &Self::Identifier) -> reqwest::Result<Url>;
  fn delay(&self, queue_len: usize) -> Duration;

  #[allow(unused)]
  fn filter(&self, entry: &Self::Entry) -> bool {
    true
  }
}

impl<T> Model for Arc<T> where T: Model {
  type Identifier = T::Identifier;
  type Entry = T::Entry;

  #[inline]
  fn url(&self, identifier: &Self::Identifier) -> reqwest::Result<Url> {
    T::url(self, identifier)
  }

  #[inline]
  fn delay(&self, queue_len: usize) -> Duration {
    T::delay(self, queue_len)
  }

  #[inline]
  fn filter(&self, entry: &Self::Entry) -> bool {
    T::filter(self, entry)
  }
}



/// A handle to a periodic feed collection task. Wraps a [`Handle`] and a [`Context`].
#[derive(Debug)]
pub struct HandleWithContext<M: Model, C: Context<M>> {
  pub handle: Handle<M>,
  pub context: C
}

impl<M: Model, C: Context<M> + Clone> HandleWithContext<M, C> {
  pub fn new(model: M, context: C) -> Self {
    HandleWithContext { handle: Handle::new(model), context }
  }

  pub async fn get_queue(&self) -> VecDeque<M::Identifier> {
    self.handle.get_queue().await
  }

  pub async fn push_queue(&self, identifier: M::Identifier) {
    self.handle.push_queue(&self.context, identifier).await;
  }

  pub async fn extend_queue(&self, identifiers: impl IntoIterator<Item = M::Identifier>) {
    self.handle.extend_queue(&self.context, identifiers).await;
  }

  pub async fn replace_queue(&self, identifiers: impl IntoIterator<Item = M::Identifier>) {
    self.handle.replace_queue(&self.context, identifiers).await;
  }

  pub async fn remove_queue(&self, identifier: &M::Identifier) {
    self.handle.remove_queue(&self.context, identifier).await;
  }

  pub async fn retain_queue(&self, f: impl FnMut(&M::Identifier) -> bool) {
    self.handle.retain_queue(&self.context, f).await;
  }

  pub async fn clear_queue(&self) {
    self.handle.clear_queue(&self.context).await;
  }

  pub async fn modify_queue<F>(&self, f: F)
  where F: FnOnce(&mut VecDeque<M::Identifier>) {
    self.handle.modify_queue(&self.context, f).await;
  }

  pub async fn is_running(&self) -> bool {
    self.handle.is_running().await
  }

  pub async fn is_queue_empty(&self) -> bool {
    self.handle.is_queue_empty().await
  }

  pub async fn queue_len(&self) -> usize {
    self.handle.queue_len().await
  }
}

impl<M: Model, C: Context<M> + Clone> Clone for HandleWithContext<M, C> {
  fn clone(&self) -> Self {
    HandleWithContext {
      handle: self.handle.clone(),
      context: self.context.clone()
    }
  }
}



/// A handle to a periodic feed collection task with no [`Context`] attached.
#[derive(Debug)]
pub struct Handle<M: Model> {
  inner: Arc<HandleInner<M>>
}

impl<M: Model> Handle<M> {
  pub fn new(model: M) -> Self {
    Handle { inner: Arc::new(HandleInner::new(model)) }
  }

  pub async fn get_queue(&self) -> VecDeque<M::Identifier> {
    VecDeque::clone(&*self.inner.read_queue().await)
  }

  pub async fn push_queue<C: Context<M> + Clone>(&self, context: &C, identifier: M::Identifier) {
    self.modify_queue(context, |queue| queue.push_back(identifier)).await;
  }

  pub async fn extend_queue<C: Context<M> + Clone>(&self, context: &C, identifiers: impl IntoIterator<Item = M::Identifier>) {
    self.modify_queue(context, |queue| queue.extend(identifiers)).await;
  }

  pub async fn replace_queue<C: Context<M> + Clone>(&self, context: &C, identifiers: impl IntoIterator<Item = M::Identifier>) {
    self.modify_queue(context, |queue| {
      queue.clear();
      queue.extend(identifiers);
    }).await;
  }

  pub async fn remove_queue<C: Context<M> + Clone>(&self, context: &C, identifier: &M::Identifier) {
    self.modify_queue(context, |queue| {
      remove_by_in_queue(queue, identifier);
    }).await;
  }

  pub async fn retain_queue<C: Context<M> + Clone>(&self, context: &C, f: impl FnMut(&M::Identifier) -> bool) {
    self.modify_queue(context, |queue| queue.retain(f)).await;
  }

  pub async fn clear_queue<C: Context<M> + Clone>(&self, context: &C) {
    self.modify_queue(context, |queue| queue.clear()).await;
  }

  pub async fn modify_queue<C, F>(&self, context: &C, f: F)
  where C: Context<M> + Clone, F: FnOnce(&mut VecDeque<M::Identifier>) {
    let mut state_guard = self.inner.state.write().await;
    f(&mut state_guard.queue);

    if state_guard.queue.is_empty() {
      self.inner.interrupt.notify_waiters();
    } else if state_guard.join_handle.is_none() {
      let join_handle = tokio::task::spawn(self.clone().task_static(context.clone()));
      state_guard.join_handle = Some(join_handle);
    };
  }

  pub async fn abort(&self) {
    if let Some(join_handle) = &*self.inner.read_join_handle().await {
      join_handle.abort();
    };
  }

  pub async fn is_running(&self) -> bool {
    self.inner.read_join_handle().await.is_some()
  }

  pub async fn is_queue_empty(&self) -> bool {
    self.inner.read_queue().await.is_empty()
  }

  pub async fn queue_len(&self) -> usize {
    self.inner.read_queue().await.len()
  }

  async fn task_static<C: Context<M>>(self, context: C) {
    self.inner.task(&context).await.unwrap_or_else(|error| context.on_manager_error(error));
    self.inner.write_join_handle().await.take();
  }
}

impl<M: Model> Clone for Handle<M> {
  fn clone(&self) -> Self {
    Handle { inner: self.inner.clone() }
  }
}



#[derive(Debug)]
struct State<M: Model> {
  queue: VecDeque<M::Identifier>,
  join_handle: Option<JoinHandle<()>>
}

#[derive(Debug)]
struct HandleInner<M: Model> {
  model: M,
  interrupt: Notify,
  state: RwLock<State<M>>
}

impl<M: Model> HandleInner<M> {
  fn new(model: M) -> Self {
    HandleInner {
      model,
      interrupt: Notify::new(),
      state: RwLock::new(State {
        queue: VecDeque::new(),
        join_handle: None
      })
    }
  }

  async fn wait(&self, last_advance: Instant) -> bool {
    let deadline = last_advance + self.model.delay(self.read_queue().await.len());

    loop {
      match sleep_or_notify(deadline, &self.interrupt).await {
        SleepOrNotify::Slept => break true,
        SleepOrNotify::Notified if self.read_queue().await.is_empty() => break false,
        SleepOrNotify::Notified => ()
      };
    }
  }

  async fn rotate_queue(&self) -> Option<M::Identifier> {
    rotate_queue(&mut *self.write_queue().await)
  }

  async fn task<C: Context<M>>(&self, context: &C) -> Result<(), C::Error> {
    trace!("started feed task for model `{}`", std::any::type_name::<M>());

    let mut last_advance = Instant::now();
    while self.wait(last_advance).await && let Some(identifier) = self.rotate_queue().await {
      last_advance = Instant::now();

      let url = self.model.url(&identifier).map_err(ModelError::from)?;
      let last_update = context.load_update_datetime(&identifier).await?.unwrap_or(DateTime::UNIX_EPOCH);

      trace!("requesting feed entries for model `{}` and feed identifier `{:?}`", std::any::type_name::<M>(), identifier);
      let mut entries = crate::model::get_feed_entries::<M::Entry>(context.client(), url).await?;
      entries.retain(|entry| entry.datetime() > last_update && self.model.filter(entry));
      entries.sort_unstable_by_key(|entry| entry.datetime());

      if let Some(last_update) = entries.iter().map(|entry| entry.datetime()).max() {
        context.save_update_datetime(&identifier, last_update).await?;
        context.on_new_entries(&identifier, entries).await;
      };
    };

    Ok(())
  }

  #[inline]
  async fn read_queue(&self) -> impl Deref<Target = VecDeque<M::Identifier>> {
    RwLockReadGuard::map(self.state.read().await, |state| &state.queue)
  }

  #[inline]
  async fn write_queue(&self) -> impl DerefMut<Target = VecDeque<M::Identifier>> {
    RwLockWriteGuard::map(self.state.write().await, |state| &mut state.queue)
  }

  #[inline]
  async fn read_join_handle(&self) -> impl Deref<Target = Option<JoinHandle<()>>> {
    RwLockReadGuard::map(self.state.read().await, |state| &state.join_handle)
  }

  #[inline]
  async fn write_join_handle(&self) -> impl DerefMut<Target = Option<JoinHandle<()>>> {
    RwLockWriteGuard::map(self.state.write().await, |state| &mut state.join_handle)
  }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SleepOrNotify {
  Slept, Notified
}

async fn sleep_or_notify(deadline: Instant, notify: &Notify) -> SleepOrNotify {
  tokio::select!{
    () = notify.notified() => SleepOrNotify::Notified,
    () = sleep_until(deadline) => SleepOrNotify::Slept
  }
}

fn rotate_queue<T: Clone>(queue: &mut VecDeque<T>) -> Option<T> {
  queue.front().cloned().map(|value| {
    queue.rotate_left(1);
    value
  })
}

fn remove_by_in_queue<T: PartialEq>(queue: &mut VecDeque<T>, target: &T) -> Option<T> {
  if let Some(index) = queue.iter().position(|t| t == target) {
    queue.remove(index)
  } else {
    None
  }
}
