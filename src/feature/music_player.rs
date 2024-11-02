use crate::data::Core;
use crate::utils::youtube::{self, YtDlpSource};

use reqwest::Client as HttpClient;
use serenity::model::id::{AttachmentId, ChannelId, GuildId};
use songbird::{Call, Songbird, SongbirdKey};
use songbird::events::{Event, EventContext, EventHandler, TrackEvent};
use songbird::tracks::{TrackHandle, PlayMode};
use songbird::input::{Input, HttpRequest};
use songbird::error::JoinError;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::collections::vec_deque::VecDeque;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::path::{Path, PathBuf};



#[derive(Debug)]
pub struct MusicPlayer {
  ytdlp_path: PathBuf,
  http_client: HttpClient,
  guilds: Mutex<HashMap<GuildId, Arc<Mutex<QueueBundle>>>>
}

impl MusicPlayer {
  pub fn new(ytdlp_path: PathBuf, http_client: HttpClient) -> Self {
    MusicPlayer { ytdlp_path, http_client, guilds: Mutex::new(HashMap::new()) }
  }

  pub async fn join(&self, core: &Core, guild_id: GuildId, channel_id: ChannelId) -> Result<(), JoinError> {
    let songbird = core.get::<SongbirdKey>().await;
    let session = self.join_and_deafen(&songbird, guild_id, channel_id).await?;
    session.start_playing().await;
    Ok(())
  }

  pub async fn leave(&self, core: &Core, guild_id: GuildId) -> Result<(), JoinError> {
    let songbird = core.get::<SongbirdKey>().await;
    let queue_bundle = self.get_guild_queue_bundle(guild_id).await;
    songbird.remove(guild_id).await?;
    queue_bundle.lock().await.track.take();
    Ok(())
  }

  pub async fn play(&self, core: &Core, guild_id: GuildId, channel_id: ChannelId, items: Vec<QueueItem>) -> Result<(), JoinError> {
    let songbird = core.get::<SongbirdKey>().await;
    let session = self.join_and_deafen(&songbird, guild_id, channel_id).await?;
    session.start_playing_or_append(items).await;
    Ok(())
  }

  pub async fn kill(&self, core: &Core, guild_id: GuildId) -> Result<(), JoinError> {
    let songbird = core.get::<SongbirdKey>().await;
    songbird.remove(guild_id).await?;
    self.remove_guild_queue_bundle(guild_id).await;
    Ok(())
  }

  pub async fn stop(&self, core: &Core, guild_id: GuildId) {
    let songbird = core.get::<SongbirdKey>().await;
    if let Some(session) = self.current_session(&songbird, guild_id).await {
      session.queue_bundle.lock().await.queue.clear();
      session.stop_playing().await;
    };
  }

  pub async fn skip(&self, core: &Core, guild_id: GuildId) {
    let songbird = core.get::<SongbirdKey>().await;
    if let Some(session) = self.current_session(&songbird, guild_id).await {
      session.queue_bundle.lock().await.queue.advance();
      session.start_playing().await;
    };
  }

  pub async fn current_channel(&self, core: &Core, guild_id: GuildId) -> Option<ChannelId> {
    let songbird = core.get::<SongbirdKey>().await;
    match songbird.get(guild_id) {
      Some(call) => call.lock().await.current_channel()
        .map(|channel_id| ChannelId::from(channel_id.0)),
      None => None
    }
  }

  pub async fn set_loop(&self, guild_id: GuildId, state: bool) {
    let queue_bundle = self.get_guild_queue_bundle(guild_id).await;
    queue_bundle.lock().await.queue.looped = state;
  }

  pub async fn set_pause(&self, guild_id: GuildId, state: bool) {
    let queue_bundle = self.get_guild_queue_bundle(guild_id).await;
    if let Some(track) = &queue_bundle.lock().await.track {
      let result = if state { track.pause() } else { track.play() };
      if let Err(err) = result {
        error!("failed to pause/unpause track: {err}");
      };
    };
  }

  pub async fn queue_clear_keep_one(&self, guild_id: GuildId) {
    self.queue_manipulate(guild_id, |queue| queue.clear_keep_one()).await
  }

  pub async fn queue_shuffle(&self, guild_id: GuildId) {
    self.queue_manipulate(guild_id, |queue| queue.shuffle()).await
  }

  pub async fn queue_remove(&self, guild_id: GuildId, index: NonZeroUsize) -> Option<QueueItem> {
    self.queue_manipulate(guild_id, |queue| queue.remove(index)).await
  }

  pub async fn queue_list(&self, guild_id: GuildId) -> (Vec<QueueItem>, bool) {
    self.queue_manipulate(guild_id, |queue| (queue.to_vec(), queue.looped)).await
  }

  async fn queue_manipulate<F, R>(&self, guild_id: GuildId, f: F) -> R
  where F: FnOnce(&mut Queue) -> R {
    let queue_bundle = self.get_guild_queue_bundle(guild_id).await;
    let mut queue_bundle_handle = queue_bundle.lock().await;
    f(&mut queue_bundle_handle.queue)
  }

  async fn current_session(&self, songbird: &Arc<Songbird>, guild_id: GuildId) -> Option<Session> {
    let call = songbird.get(guild_id)?;
    let queue_bundle = self.get_guild_queue_bundle(guild_id).await;
    Some(Session {
      http_client: self.http_client.clone(),
      ytdlp_path: self.ytdlp_path.clone(),
      call, queue_bundle
    })
  }

  async fn join_and_deafen(&self, songbird: &Arc<Songbird>, guild_id: GuildId, channel_id: ChannelId) -> Result<Session, JoinError> {
    let call = join_and_deafen(&songbird, guild_id, channel_id).await?;
    let queue_bundle = self.get_guild_queue_bundle(guild_id).await;
    Ok(Session {
      http_client: self.http_client.clone(),
      ytdlp_path: self.ytdlp_path.clone(),
      call, queue_bundle
    })
  }

  async fn get_guild_queue_bundle(&self, guild_id: GuildId) -> Arc<Mutex<QueueBundle>> {
    self.guilds.lock().await.entry(guild_id).or_default().clone()
  }

  async fn remove_guild_queue_bundle(&self, guild_id: GuildId) -> Option<Arc<Mutex<QueueBundle>>> {
    self.guilds.lock().await.remove(&guild_id)
  }

  pub fn ytdlp_path(&self) -> &Path {
    &self.ytdlp_path
  }
}

async fn join_and_deafen(
  songbird: &Arc<Songbird>, guild_id: GuildId, channel_id: ChannelId
) -> Result<Arc<Mutex<Call>>, JoinError> {
  let call = songbird.join(guild_id, channel_id).await?;
  let mut call_handle = call.lock().await;
  if !call_handle.is_deaf() {
    call_handle.deafen(true).await?;
  };

  std::mem::drop(call_handle);
  Ok(call)
}

#[derive(Debug, Clone)]
struct Session {
  ytdlp_path: PathBuf,
  http_client: HttpClient,
  queue_bundle: Arc<Mutex<QueueBundle>>,
  call: Arc<Mutex<Call>>
}

impl Session {
  async fn stop_playing(&self) {
    let mut queue_bundle = self.queue_bundle.lock().await;
    let mut call = self.call.lock().await;
    call.stop();
    queue_bundle.track = None;
  }

  /// Start playing the first item in the queue, stopping any other tracks that might be playing, internally.
  async fn start_playing(&self) {
    let mut queue_bundle = self.queue_bundle.lock().await;
    let mut call = self.call.lock().await;
    if let Some(current_item) = queue_bundle.queue.get_current() {
      let input = current_item.to_input(self.http_client.clone(), self.ytdlp_path.clone());
      let track_handle = self.play(&mut call, input);
      queue_bundle.track = Some(track_handle);
    } else {
      queue_bundle.track = None;
    };
  }

  /// Adds the item to the end of the queue if a track is playing, otherwise causes it to start playing.
  async fn start_playing_or_append(&self, items: Vec<QueueItem>) {
    let mut queue_bundle = self.queue_bundle.lock().await;
    let mut call = self.call.lock().await;
    queue_bundle.queue.append(items);
    if let (None, Some(current_item)) = (&queue_bundle.track, queue_bundle.queue.get_current()) {
      let input = current_item.to_input(self.http_client.clone(), self.ytdlp_path.clone());
      let track_handle = self.play(&mut call, input);
      queue_bundle.track = Some(track_handle);
    };
  }

  fn play(&self, call: &mut Call, input: Input) -> TrackHandle {
    let track_handle = call.play_only_input(input);
    track_handle.add_event(Event::Track(TrackEvent::End), OnTrackEnd(self.clone())).unwrap();
    track_handle.add_event(Event::Track(TrackEvent::Error), OnTrackError(self.clone())).unwrap();
    track_handle
  }
}

#[derive(Debug, Clone)]
struct OnTrackEnd(Session);

#[serenity::async_trait]
impl EventHandler for OnTrackEnd {
  async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
    if let &EventContext::Track(&[(state, ..)]) = ctx {
      if let PlayMode::End = state.playing {
        self.0.queue_bundle.lock().await.queue.advance();
        self.0.start_playing().await;
      };
    };

    None
  }
}

#[derive(Debug, Clone)]
struct OnTrackError(Session);

#[serenity::async_trait]
impl EventHandler for OnTrackError {
  async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
    if let &EventContext::Track(&[(state, ..)]) = ctx {
      if let PlayMode::Errored(ref err) = state.playing {
        self.0.queue_bundle.lock().await.queue.advance();
        self.0.start_playing().await;
        error!("track errror: {err}");
      };
    };

    None
  }
}

#[derive(Debug, Default)]
struct QueueBundle {
  queue: Queue,
  track: Option<TrackHandle>
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Queue {
  /// Position 0 in the queue is special and cannot be cleared, as it is the currently playing track.
  contents: VecDeque<QueueItem>,
  looped: bool
}

impl Queue {
  fn clear(&mut self) {
    self.contents.clear();
  }

  fn clear_keep_one(&mut self) {
    self.contents.truncate(1);
  }

  fn shuffle(&mut self) {
    match self.contents.make_contiguous().split_first_mut() {
      Some((_, &mut [])) | None => (),
      Some((_, contents)) => crate::utils::shuffle(contents)
    };
  }

  fn advance(&mut self) {
    if let Some(item) = self.contents.pop_front() {
      if self.looped {
        self.contents.push_back(item);
      };
    };
  }

  fn append(&mut self, items: impl IntoIterator<Item = QueueItem>) {
    self.contents.extend(items);
  }

  fn remove(&mut self, index: NonZeroUsize) -> Option<QueueItem> {
    self.contents.remove(index.get())
  }

  fn get_current(&self) -> Option<&QueueItem> {
    self.contents.front()
  }

  fn to_vec(&self) -> Vec<QueueItem> {
    self.contents.iter().cloned().collect()
  }
}

impl Default for Queue {
  fn default() -> Self {
    Queue {
      contents: VecDeque::new(),
      looped: false
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueItem {
  YouTube(YouTubeItem),
  Attachment(AttachmentItem)
}

impl QueueItem {
  fn to_input(&self, http_client: HttpClient, ytdlp_path: PathBuf) -> Input {
    match self {
      QueueItem::YouTube(item) => item.to_input(http_client, ytdlp_path).into(),
      QueueItem::Attachment(item) => item.to_input(http_client).into()
    }
  }
}

impl fmt::Display for QueueItem {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      QueueItem::YouTube(item) => fmt::Display::fmt(item, f),
      QueueItem::Attachment(item) => fmt::Display::fmt(item, f)
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YouTubeItem {
  pub id: String
}

impl YouTubeItem {
  fn to_input(&self, http_client: HttpClient, ytdlp_path: PathBuf) -> YtDlpSource {
    YtDlpSource::new(ytdlp_path, self.id.clone(), http_client)
  }
}

impl fmt::Display for YouTubeItem {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "<{}>", youtube::display_video_url(&self.id))
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentItem {
  pub id: AttachmentId,
  pub filename: String,
  pub filesize: u32,
  pub url: String
}

impl AttachmentItem {
  fn to_input(&self, http_client: HttpClient) -> HttpRequest {
    HttpRequest::new(http_client, self.url.clone())
  }
}

impl fmt::Display for AttachmentItem {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "`{}`", self.filename)
  }
}
