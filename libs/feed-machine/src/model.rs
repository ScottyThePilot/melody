pub mod twitter;
pub mod youtube;

use chrono::{DateTime, Utc};
use feed::parser::{Builder as ParserBuilder, ParseFeedError};
use reqwest::{Client, Error as ReqwestError};
use itertools::Itertools;

pub use mediatype::{MediaType, MediaTypeBuf, MediaTypeError};
pub use feed::model::{
  Category, Content, Entry, Feed, FeedType, Generator, Image, Link,
  MediaCommunity, MediaContent, MediaCredit,
  MediaObject, MediaRating, MediaText, MediaThumbnail,
  Person, Text
};
pub use url::Url;

use std::convert::Infallible;



#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[error("failed to perform a conversion")]
pub struct SchemaError;

#[derive(Debug, Error)]
pub enum ModelError<E = Infallible> {
  #[error("feed parsing error: {0}")]
  ParseFeedError(#[from] ParseFeedError),
  #[error("feed request error: {0}")]
  ReqwestError(#[from] ReqwestError),
  #[error("failed to convert feed entry: {0}")]
  ConvertFeedEntry(E)
}

impl ModelError {
  pub fn with<E>(self) -> ModelError<E> {
    match self {
      Self::ParseFeedError(error) => ModelError::ParseFeedError(error),
      Self::ReqwestError(error) => ModelError::ReqwestError(error),
      Self::ConvertFeedEntry(error) => match error {}
    }
  }
}

pub async fn get_feed(client: &Client, url: Url) -> Result<Feed, ModelError> {
  let payload = client.get(url.clone()).send().await?.bytes().await?;
  let parser = ParserBuilder::new().base_uri(Some(&url)).build();
  let feed = parser.parse(payload.as_ref())?;
  Ok(feed)
}

pub async fn get_feed_entries<E: TryFrom<Entry>>(client: &Client, url: Url) -> Result<Vec<E>, ModelError<E::Error>> {
  get_feed(client, url).await.map_err(ModelError::with)
    .and_then(|feed| convert_feed_entries(feed).map_err(ModelError::ConvertFeedEntry))
}

pub fn convert_feed_entries<E: TryFrom<Entry>>(feed: Feed) -> Result<Vec<E>, E::Error> {
  feed.entries.into_iter().map(E::try_from).collect::<Result<Vec<E>, E::Error>>()
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EntryMinimal {
  pub id: String,
  pub title: Option<String>,
  pub content: Option<String>,
  pub summary: Option<String>,
  pub author: Option<String>,
  pub link: Option<Url>,
  pub time: DateTime<Utc>,
  pub category: Option<String>,
  pub media: Vec<MediaMinimal>
}

impl TryFrom<Entry> for EntryMinimal {
  type Error = SchemaError;

  fn try_from(entry: Entry) -> Result<Self, Self::Error> {
    let id = entry.id;
    let title = entry.title.map(|text| text.content);
    let content = entry.content.and_then(|content| content.body);
    let summary = entry.summary.map(|text| text.content);
    let author = join(entry.authors.into_iter().map(|person| person.name));
    let link = entry.links.into_iter()
      .find_map(|link| Url::parse(&link.href).ok());
    let time = entry.published.ok_or(SchemaError)?;
    let category = entry.categories.into_iter()
      .next().map(|category| category.term);
    let media = entry.media.into_iter()
      .map(MediaMinimal::try_from)
      .collect::<Result<Vec<MediaMinimal>, _>>()?;
    Ok(EntryMinimal {
      id, title, content, summary,
      author, link, time, category, media
    })
  }
}

impl HasDateTime for EntryMinimal {
  fn datetime(&self) -> DateTime<Utc> {
    self.time
  }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MediaMinimal {
  pub title: Option<String>,
  pub description: Option<String>,
  pub content_type: ContentType,
  pub link: Option<Url>,
  pub thumbnails: Vec<Url>
}

impl TryFrom<MediaObject> for MediaMinimal {
  type Error = SchemaError;

  fn try_from(media_object: MediaObject) -> Result<Self, Self::Error> {
    let title = media_object.title.map(|text| text.content);
    let description = media_object.description.map(|text| text.content);
    let (content_type, link) = media_object.content.into_iter()
      .find_map(|media_content| {
        media_content.content_type.as_ref()
          .map(MediaTypeBuf::to_ref)
          .and_then(|content_type| content_type.try_into().ok())
          .map(|content_type| (content_type, media_content.url))
      })
      .ok_or(SchemaError)?;
    let thumbnails = media_object.thumbnails.into_iter()
      .filter_map(|media_thumbnail| Url::parse(&media_thumbnail.image.uri).ok())
      .collect::<Vec<Url>>();
    Ok(MediaMinimal { title, description, content_type, link, thumbnails })
  }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ContentType {
  Audio,
  Video,
  Image,
  Application,
  Text,
  Font
}

impl ContentType {
  const VARIANTS: &'static [Self] = &[
    Self::Audio,
    Self::Video,
    Self::Image,
    Self::Application,
    Self::Text,
    Self::Font
  ];

  pub fn to_str(self) -> &'static str {
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

impl TryFrom<MediaType<'_>> for ContentType {
  type Error = SchemaError;

  fn try_from(mediatype: MediaType<'_>) -> Result<Self, Self::Error> {
    let ty = mediatype.essence().ty;
    Self::VARIANTS.into_iter()
      .find_map(|&variant| {
        (ty == variant.to_str()).then_some(variant)
      })
      .ok_or(SchemaError)
  }
}



fn join(iter: impl IntoIterator<Item = String>) -> Option<String> {
  let string = iter.into_iter().join(", ");
  if string.is_empty() { None } else { Some(string) }
}



pub trait HasDateTime {
  fn datetime(&self) -> DateTime<Utc>;
}

impl HasDateTime for DateTime<Utc> {
  #[inline]
  fn datetime(&self) -> DateTime<Utc> {
    *self
  }
}

macro_rules! impl_has_datetime_deref {
  ($T:ident, $Type:ty) => (
    impl<$T> HasDateTime for $Type where $T: HasDateTime {
      #[inline]
      fn datetime(&self) -> DateTime<Utc> {
        T::datetime(self)
      }
    }
  );
}

impl_has_datetime_deref!(T, &T);
impl_has_datetime_deref!(T, &mut T);
impl_has_datetime_deref!(T, Box<T>);
impl_has_datetime_deref!(T, std::sync::Arc<T>);
impl_has_datetime_deref!(T, std::rc::Rc<T>);
