use chrono::{DateTime, Utc};
use feed::model::Entry;
use itertools::Itertools;
use url::Url;

use super::{HasDateTime, SchemaError};



#[derive(Debug, Clone)]
pub struct TwitterPost {
  pub title: String,
  pub author: String,
  pub link: Url,
  pub time: DateTime<Utc>
}

impl TryFrom<Entry> for TwitterPost {
  type Error = SchemaError;

  fn try_from(entry: Entry) -> Result<Self, Self::Error> {
    let title = entry.title.map(|text| text.content).ok_or(SchemaError)?;
    let author = entry.authors.into_iter().map(|person| person.name).join(", ");
    let mut link = entry.links.into_iter()
      .find_map(|link| Url::parse(&link.href).ok())
      .ok_or(SchemaError)?;
    link.set_host(Some("twitter.com")).ok().ok_or(SchemaError)?;
    link.set_fragment(None);
    let time = entry.published.ok_or(SchemaError)?;
    Ok(TwitterPost { title, author, link, time })
  }
}

impl HasDateTime for TwitterPost {
  fn datetime(&self) -> DateTime<Utc> {
    self.time
  }
}
