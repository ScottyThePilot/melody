extern crate feed;
extern crate mime;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate thiserror;

pub extern crate chrono;
pub extern crate reqwest;
pub extern crate url;

use chrono::{DateTime, Utc};
use mime::Mime;
use feed::model::{Feed, Entry, MediaObject};
use feed::parser::{Builder as ParserBuilder, ParseFeedError};
use reqwest::{Client, Error as ReqwestError};
use url::Url;



#[derive(Debug, Error)]
pub enum FeedError {
  #[error("feed parsing error: {0}")]
  ParseFeed(#[from] ParseFeedError),
  #[error("feed request error: {0}")]
  Reqwest(#[from] ReqwestError),
  #[error("invalid feed entry at index {0}")]
  InvalidFeedEntry(usize)
}

pub async fn get_feed(client: &Client, url: &str) -> Result<Vec<FeedEntry>, FeedError> {
  get_feed_inner::<FeedEntry>(client, url).await
}

pub async fn get_youtube_feed(client: &Client, url: &str) -> Result<Vec<YouTubeVideo>, FeedError> {
  get_feed_inner::<YouTubeVideo>(client, url).await
}

pub async fn get_twitter_feed(client: &Client, url: &str) -> Result<Vec<TwitterPost>, FeedError> {
  get_feed_inner::<TwitterPost>(client, url).await
}

async fn get_feed_inner<E: FromFeedEntry>(client: &Client, url: &str) -> Result<Vec<E>, FeedError> {
  let payload = client.get(url).send().await?.bytes().await?;
  let feed = ParserBuilder::new().base_uri(Some(url)).build().parse(payload.as_ref())?;
  let feed_entries = decompose_feed(feed).map_err(FeedError::InvalidFeedEntry)?;
  Ok(feed_entries)
}

fn decompose_feed<E: FromFeedEntry>(feed: Feed) -> Result<Vec<E>, usize> {
  feed.entries.into_iter().enumerate()
    .map(|(i, entry)| FromFeedEntry::from_entry(entry).ok_or(i))
    .collect()
}

trait FromFeedEntry: Sized {
  fn from_entry(entry: Entry) -> Option<Self>;
}

impl FromFeedEntry for FeedEntry {
  fn from_entry(entry: Entry) -> Option<Self> {
    FeedEntry::from_entry(entry)
  }
}

impl FromFeedEntry for YouTubeVideo {
  fn from_entry(entry: Entry) -> Option<Self> {
    FeedEntry::from_entry(entry).and_then(YouTubeVideo::from_feed_entry)
  }
}

impl FromFeedEntry for TwitterPost {
  fn from_entry(entry: Entry) -> Option<Self> {
    FeedEntry::from_entry(entry).and_then(TwitterPost::from_feed_entry)
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeedEntry {
  pub id: String,
  pub title: Option<String>,
  pub content: Option<String>,
  pub summary: Option<String>,
  pub author: Option<String>,
  pub link: Option<Url>,
  pub time: DateTime<Utc>,
  pub category: Option<String>,
  pub media: Vec<FeedMedia>
}

impl FeedEntry {
  fn from_entry(entry: Entry) -> Option<Self> {
    let id = entry.id;
    let title = entry.title.map(|text| text.content);
    let content = entry.content.and_then(|content| content.body);
    let summary = entry.summary.map(|text| text.content);
    let author = entry.authors.into_iter()
      .next().map(|person| person.name);
    let link = entry.links.into_iter()
      .find_map(|link| Url::parse(&link.href).ok());
    let time = entry.published?;
    let category = entry.categories.into_iter()
      .next().map(|category| category.term);
    let media = entry.media.into_iter()
      .filter_map(FeedMedia::from_media_object)
      .collect::<Vec<FeedMedia>>();
    Some(FeedEntry { id, title, content, summary, author, link, time, category, media })
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeedMedia {
  pub title: Option<String>,
  pub description: Option<String>,
  pub content_type: FeedContentType,
  pub link: Option<Url>,
  pub thumbnails: Vec<Url>
}

impl FeedMedia {
  fn from_media_object(media_object: MediaObject) -> Option<Self> {
    let title = media_object.title.map(|text| text.content);
    let description = media_object.description.map(|text| text.content);
    let (content_type, link) = media_object.content.into_iter()
      .find_map(|media_content| {
        media_content.content_type
          .and_then(FeedContentType::from_mime)
          .zip(media_content.url)
      })
      .unzip();
    let content_type = content_type?;
    let thumbnails = media_object.thumbnails.into_iter()
      .filter_map(|media_thumbnail| Url::parse(&media_thumbnail.image.uri).ok())
      .collect::<Vec<Url>>();
    Some(FeedMedia { title, description, content_type, link, thumbnails })
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum FeedContentType {
  Audio,
  Video,
  Image,
  Application,
  Text,
  Font
}

impl FeedContentType {
  const VARIANTS: &'static [(&'static str, Self)] = &[
    ("audio", Self::Audio),
    ("video", Self::Video),
    ("image", Self::Image),
    ("application", Self::Application),
    ("text", Self::Text),
    ("font", Self::Font)
  ];

  fn from_mime(mime: Mime) -> Option<Self> {
    let t = mime.type_().as_str();
    Self::VARIANTS.into_iter().find_map(|&(name, out)| {
      (t == name).then_some(out)
    })
  }

  pub fn into_str(self) -> &'static str {
    match self {
      Self::Audio => "audio",
      Self::Video => "video",
      Self::Image => "image",
      Self::Application => "application",
      Self::Text => "text",
      Self::Font => "font"
    }
  }
}

#[derive(Debug, Clone)]
pub struct YouTubeVideo {
  pub id: String,
  pub title: String,
  pub author: String,
  pub description: String,
  pub link: Url,
  pub time: DateTime<Utc>,
  pub thumbnail: Url
}

impl YouTubeVideo {
  fn from_feed_entry(feed_entry: FeedEntry) -> Option<Self> {
    let id = feed_entry.id.strip_prefix("yt:video:")?.to_owned();
    let title = feed_entry.title?;
    let author = feed_entry.author?;
    let mut link = feed_entry.link?;
    link.set_host(Some("www.youtube.com")).ok()?;
    let time = feed_entry.time;
    let feed_media = feed_entry.media.into_iter().next()?;
    let description = feed_media.description?;
    let thumbnail = feed_media.thumbnails.into_iter().next()?;
    Some(YouTubeVideo { id, title, author, description, link, time, thumbnail })
  }
}

#[derive(Debug, Clone)]
pub struct TwitterPost {
  pub title: String,
  pub author: String,
  pub link: Url,
  pub time: DateTime<Utc>
}

impl TwitterPost {
  fn from_feed_entry(feed_entry: FeedEntry) -> Option<Self> {
    let title = feed_entry.title?;
    let author = feed_entry.author?.trim_start_matches('@').to_owned();
    let mut link = feed_entry.link?;
    link.set_host(Some("twitter.com")).ok()?;
    link.set_fragment(None);
    let time = feed_entry.time;
    Some(TwitterPost { title, author, link, time })
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TwitterPostType {
  Original,
  Retweet
}
