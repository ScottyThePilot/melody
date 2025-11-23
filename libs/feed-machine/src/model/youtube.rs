use chrono::{DateTime, Utc};
use feed::model::Entry;
use itertools::Itertools;
use url::Url;

use super::{HasDateTime, SchemaError};



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

impl TryFrom<Entry> for YouTubeVideo {
  type Error = SchemaError;

  fn try_from(entry: Entry) -> Result<Self, Self::Error> {
    let id = entry.id.strip_prefix("yt:video:").ok_or(SchemaError)?.to_owned();
    let title = entry.title.map(|text| text.content).ok_or(SchemaError)?;
    let author = entry.authors.into_iter().map(|person| person.name).join(", ");
    let link = entry.links.into_iter()
      .find_map(|link| Url::parse(&link.href).ok())
      .ok_or(SchemaError)?;
    let time = entry.published.ok_or(SchemaError)?;
    let media = entry.media.into_iter().next().ok_or(SchemaError)?;
    let description = media.description.map(|text| text.content).ok_or(SchemaError)?;
    let thumbnail = media.thumbnails.into_iter()
      .filter_map(|media_thumbnail| Url::parse(&media_thumbnail.image.uri).ok())
      .next().ok_or(SchemaError)?;
    Ok(YouTubeVideo { id, title, author, description, link, time, thumbnail })
  }
}

impl HasDateTime for YouTubeVideo {
  fn datetime(&self) -> DateTime<Utc> {
    self.time
  }
}
