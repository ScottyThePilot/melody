use crate::data::{Core, ConfigRssTwitter, ConfigRssYouTube};

use chrono::{DateTime, Utc};
use melody_rss_feed::{FeedError, TwitterPost, YouTubeVideo};
use melody_rss_feed::reqwest::Client;
use rand::Rng;
use serenity::model::id::{ChannelId, GuildId};
use tokio::sync::Mutex;
use tokio::time::{sleep, sleep_until, Instant};
use tokio::task::JoinHandle;

use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;



#[allow(unused_variables)]
#[serenity::async_trait]
pub trait FeedEventHandler: Send + Sync + 'static {
  /// A feed was terminated, either due to error or due to reaching the max failures amount.
  async fn terminate(self: Arc<Self>, core: Core, feed: Feed) {}
  /// A feed failed due to an error.
  async fn failure(&self, core: Core, feed: Feed, error: FeedError, failures_left: usize) {}
  /// A YouTube feed has recieved a new video.
  async fn feed_youtube_video(&self, core: Core, channel: &str, video: YouTubeVideo) {}
  /// A Twitter feed has recieved a new post.
  async fn feed_twitter_post(&self, core: Core, handle: &str, post: TwitterPost) {}
}

pub type FeedWrapper = Arc<Mutex<FeedManager>>;

pub struct FeedManager {
  client: Client,
  handler: Arc<dyn FeedEventHandler>,
  tasks: HashMap<Feed, JoinHandle<()>>
}

impl FeedManager {
  pub fn new<H>(client: Client, handler: H) -> Self
  where H: FeedEventHandler {
    FeedManager {
      client,
      handler: Arc::new(handler),
      tasks: HashMap::new()
    }
  }

  pub async fn register(&mut self, core: &Core, feed: Feed) -> Option<&JoinHandle<()>> {
    use std::collections::hash_map::Entry;
    match self.tasks.entry(feed.clone()) {
      Entry::Vacant(entry) => feed.task(core, &self.client, &self.handler)
        .await.map(|task| &*entry.insert(task)),
      Entry::Occupied(entry) => Some(entry.into_mut())
    }
  }

  pub async fn respawn_all(&mut self, core: &Core) {
    for (feed, handle) in self.tasks.iter_mut() {
      if handle.is_finished() {
        if let Some(task) = feed.task(core, &self.client, &self.handler).await {
          std::mem::drop(std::mem::replace(handle, task).await);
        };
      };
    };
  }

  pub fn abort_all(&mut self) {
    for (feed, handle) in self.tasks.drain() {
      handle.abort();
      trace!("RSS Feed: aborted task for {feed}");
    };
  }

  pub fn abort(&mut self, feed: &Feed) -> bool {
    if let Some(handle) = self.tasks.remove(feed) {
      handle.abort();
      trace!("RSS Feed: aborted task for {feed}");
      true
    } else {
      false
    }
  }

  pub fn tasks(&self) -> impl Iterator<Item = (&Feed, bool)> {
    self.tasks.iter().map(|(feed, handle)| (feed, !handle.is_finished()))
  }
}

impl fmt::Debug for FeedManager {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_struct("FeedManager")
      .field("client", &self.client)
      .field("tasks", &self.tasks)
      .finish_non_exhaustive()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedState {
  pub last_update: DateTime<Utc>,
  pub guilds: HashMap<GuildId, ChannelId>
}

impl FeedState {
  pub fn new() -> Self {
    FeedState {
      last_update: Utc::now(),
      guilds: HashMap::new()
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Feed {
  YouTube {
    /// Channel ID, for example `UC7_YxT-KID8kRbqZo7MyscQ`.
    channel: String
  },
  Twitter {
    /// Twitter handle, for example `markiplier`.
    handle: String
  }
}

impl Feed {
  async fn task(&self, core: &Core, client: &Client, handler: &Arc<dyn FeedEventHandler>) -> Option<JoinHandle<()>> {
    core.operate_config(|config| match self {
      Feed::YouTube { channel } => config.rss.youtube.as_ref().map(|config| {
        tokio::spawn(youtube_task(
          core.clone(), client.clone(), handler.clone(),
          channel.clone(), config.clone()
        ))
      }),
      Feed::Twitter { handle } => config.rss.twitter.as_ref().map(|config| {
        tokio::spawn(twitter_task(
          core.clone(), client.clone(), handler.clone(),
          handle.clone(), config.clone()
        ))
      })
    }).await
  }
}

impl fmt::Display for Feed {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Feed::YouTube { channel } => write!(f, "www.youtube.com/channel/{channel}"),
      Feed::Twitter { handle } => write!(f, "twitter.com/{handle}")
    }
  }
}

impl PartialEq for Feed {
  #[inline]
  fn eq(&self, other: &Feed) -> bool {
    match (self, other) {
      (Feed::YouTube { channel: c0 }, Feed::YouTube { channel: c1 }) => *c0 == *c1,
      (Feed::Twitter { handle: h0 }, Feed::Twitter { handle: h1 }) => str::eq_ignore_ascii_case(h0, h1),
      _ => false
    }
  }
}

impl Eq for Feed {}

impl Hash for Feed {
  #[inline]
  fn hash<H: Hasher>(&self, state: &mut H) {
    std::mem::discriminant(self).hash(state);
    match self {
      Feed::YouTube { channel } => channel.hash(state),
      Feed::Twitter { handle } => handle.to_lowercase().hash(state)
    }
  }
}

async fn set_last_updated(core: &Core, feed: &Feed, last_update: DateTime<Utc>) {
  log_result!(core.operate_persist_commit(|persist| {
    if let Some(feed_state) = persist.feeds.get_mut(&feed) {
      feed_state.last_update = last_update;
    };

    Ok(())
  }).await);
}

const MAX_FAILURES: usize = 32;

async fn youtube_task(
  core: Core, client: Client, handler: Arc<dyn FeedEventHandler>,
  channel: String, config_rss_youtube: Arc<ConfigRssYouTube>
) {
  let feed = Feed::YouTube { channel: channel.to_owned() };
  trace!("RSS Feed (YouTube): task started for {feed}");
  sleep(random_delay(config_rss_youtube.interval)).await;

  let mut failures = 0;
  'outer: while failures < MAX_FAILURES {
    'inner: loop {
      let Some(last_update) = get_last_update(&core, &feed).await else { break 'outer };
      let url = config_rss_youtube.get_url(&channel);
      let deadline = Instant::now() + config_rss_youtube.interval;

      trace!("RSS Feed (YouTube): making fetch to {url}");
      let result = match melody_rss_feed::get_youtube_feed(&client, &url).await {
        Ok(mut videos) => Ok({
          // failures count is reset upon a success
          failures = 0;
          videos.retain(|entry| entry.time > last_update);
          videos.sort_unstable_by_key(|entry| entry.time);
          if let Some(time) = videos.iter().map(|entry| entry.time).max() {
            for video in videos.into_iter() {
              trace!("RSS Feed (YouTube): new video (time {time})");
              handler.feed_youtube_video(core.clone(), &channel, video).await;
            };

            Some(time)
          } else {
            None
          }
        }),
        Err(error) => {
          failures += 1;
          Err(error)
        }
      };

      match result {
        Ok(Some(last_update)) => {
          set_last_updated(&core, &feed, last_update).await;
          sleep_until(deadline).await;
        },
        Ok(None) => {
          sleep_until(deadline).await;
        },
        Err(error) => {
          let failures_left = MAX_FAILURES - failures;
          error!("RSS Feed (YouTube): failure at {url}: {error}");
          handler.failure(core.clone(), feed.clone(), error, failures_left).await;
          sleep_until(deadline).await;
          break 'inner;
        }
      };
    };
  };

  handler.terminate(core, feed).await;
}

async fn twitter_task(
  core: Core, client: Client, handler: Arc<dyn FeedEventHandler>,
  handle: String, config_rss_twitter: Arc<ConfigRssTwitter>
) {
  let feed = Feed::Twitter { handle: handle.to_owned() };
  trace!("RSS Feed (Twitter): task started for {feed}");
  sleep(random_delay(config_rss_twitter.interval)).await;

  let mut failures = 0;
  'outer: while failures < MAX_FAILURES {
    'inner: loop {
      let Some(last_update) = get_last_update(&core, &feed).await else { break 'outer };
      let url = config_rss_twitter.get_url(&handle);
      let deadline = Instant::now() + config_rss_twitter.interval;

      trace!("RSS Feed (Twitter): making fetch to {url}");
      let result = match melody_rss_feed::get_twitter_feed(&client, &url).await {
        Ok(mut posts) => Ok({
          // failures count is reset upon a success
          failures = 0;
          // when a tweet is a reqweet, the author will be that of the retweeted post
          posts.retain(|entry| entry.time > last_update && entry.author.eq_ignore_ascii_case(&handle));
          posts.sort_unstable_by_key(|entry| entry.time);
          if let Some(time) = posts.iter().map(|entry| entry.time).max() {
            for post in posts.into_iter().rev() {
              trace!("RSS Feed (Twitter): new post (time {time})");
              handler.feed_twitter_post(core.clone(), &handle, post).await;
            };

            Some(time)
          } else {
            None
          }
        }),
        Err(error) => Err(error)
      };

      match result {
        Ok(Some(last_update)) => {
          set_last_updated(&core, &feed, last_update).await;
          sleep_until(deadline).await;
        },
        Ok(None) => {
          sleep_until(deadline).await;
        },
        Err(error) => {
          let failures_left = MAX_FAILURES - failures;
          error!("RSS Feed (Twitter): failure at {url}: {error}");
          handler.failure(core.clone(), feed.clone(), error, failures_left).await;
          sleep_until(deadline).await;
          break 'inner;
        }
      };
    };
  };

  handler.terminate(core, feed).await;
}

fn random_delay(interval: Duration) -> Duration {
  let mut rng = rand::thread_rng();
  interval.mul_f64(rng.gen_range(0.0..1.0))
}

async fn get_last_update(core: &Core, feed: &Feed) -> Option<DateTime<Utc>> {
  core.operate_persist(|persist| {
    persist.feeds.get(&feed).map(|feed_state| feed_state.last_update)
  }).await
}
