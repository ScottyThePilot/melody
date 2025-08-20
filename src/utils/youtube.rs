use chrono::{DateTime, Utc, TimeDelta};
use cacheable::{CacheAsync, CacheableAsync};
use reqwest::Client as HttpClient;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::de::DeserializeOwned;
use singlefile_formats::data::json_serde::original as serde_json;
use songbird::input::{AuxMetadata, AudioStream, AudioStreamError, Compose, HttpRequest, Input};
use songbird::input::core::io::MediaSource;
use tokio::process::Command;
use url::Url;

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct YtDlp {
  program: Arc<Path>
}

impl YtDlp {
  pub fn new(program: impl Into<Arc<Path>>) -> Self {
    YtDlp { program: program.into() }
  }

  async fn run(&self, args: impl IntoIterator<Item = &str>) -> Result<Vec<u8>, YtDlpError> {
    Command::new(self.program.as_ref()).args(args).output().await
      .map_err(|err| YtDlpError::Io(err, self.program.to_path_buf()))
      .and_then(|output| match output.status.success() {
        true => Ok(output.stdout),
        false => Err(YtDlpError::Program(String::from_utf8_lossy(&output.stderr).into_owned()))
      })
  }

  async fn run_json<T: DeserializeOwned>(&self, args: impl IntoIterator<Item = &str>) -> Result<T, YtDlpError> {
    self.run(std::iter::once("-J").chain(args)).await.and_then(|output_json| {
      serde_json::from_slice::<T>(&output_json).map_err(YtDlpError::Json)
    })
  }

  pub async fn update(&self, update_to: &str) -> Result<String, YtDlpError> {
    self.run(["--update-to", update_to]).await
      .map(|output| String::from_utf8_lossy(&output).into_owned())
  }

  pub async fn get_video_info(&self, video_id: &str) -> Result<VideoInfo, YtDlpError> {
    assert!(is_id_str(video_id, 16), "video id {video_id:?} was invalid");
    let url = display_video_url(video_id).to_string();
    self.run_json([url.as_str(), "-f", "ba[abr>0][vcodec=none]/best", "--no-playlist"]).await
  }

  pub async fn get_playlist_info(&self, playlist_id: &str) -> Result<PlaylistInfo, YtDlpError> {
    assert!(is_id_str(playlist_id, 40), "playlist id {playlist_id:?} was invalid");
    let url = display_playlist_url(playlist_id).to_string();
    self.run_json([url.as_str(), "--compat-options", "no-youtube-unavailable-videos", "--yes-playlist"]).await
  }
}

#[derive(Debug, Error)]
pub enum YtDlpError {
  #[error("{0}: {1}")]
  Io(std::io::Error, PathBuf),
  #[error("program error: {0}")]
  Program(String),
  #[error(transparent)]
  Json(serde_json::Error)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoInfo {
  pub id: String,
  pub title: String,
  pub thumbnail: String,
  pub description: String,
  pub duration: f64,
  pub webpage_url: String,
  #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
  pub timestamp: Option<DateTime<Utc>>,
  pub album: Option<String>,
  pub artist: Option<String>,
  pub track: Option<String>,
  pub channel: Option<String>,
  pub channel_id: Option<String>,
  pub channel_url: Option<String>,
  pub uploader: Option<String>,
  pub uploader_id: Option<String>,
  pub uploader_url: Option<String>,
  pub filesize: Option<u64>,
  pub http_headers: Option<HashMap<String, String>>,
  pub url: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistVideoInfo {
  pub id: String
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlaylistInfo {
  pub id: String,
  pub title: Option<String>,
  pub channel: Option<String>,
  pub uploader: Option<String>,
  pub entries: Vec<PlaylistVideoInfo>,
  pub webpage_url: Option<String>
}

impl VideoInfo {
  pub fn into_aux_metadata(self) -> AuxMetadata {
    AuxMetadata {
      track: self.track,
      artist: self.artist.or(self.uploader),
      album: self.album,
      date: None,
      channels: Some(2),
      channel: self.channel,
      duration: Some(Duration::from_secs_f64(self.duration)),
      sample_rate: Some(48000),
      source_url: Some(self.webpage_url),
      title: Some(self.title),
      thumbnail: Some(self.thumbnail),
      ..AuxMetadata::default()
    }
  }
}

#[derive(Debug, Clone)]
struct VideoInfoExtended {
  video_info: VideoInfo,
  timestamp: DateTime<Utc>
}

#[serenity::async_trait]
impl CacheableAsync for VideoInfoExtended {
  type Args<'a> = (&'a YtDlp, &'a str);
  type Error = YtDlpError;

  async fn operation((yt_dlp, video_id): Self::Args<'_>) -> Result<Self, Self::Error> {
    let video_info = yt_dlp.get_video_info(video_id).await?;
    let timestamp = Utc::now();
    Ok(VideoInfoExtended { video_info, timestamp })
  }

  fn is_invalid(&self, _args: &Self::Args<'_>) -> bool {
    Utc::now().signed_duration_since(self.timestamp) > TimeDelta::seconds(3)
  }
}

#[derive(Debug, Clone)]
pub struct YtDlpSource {
  yt_dlp: YtDlp,
  video_id: String,
  http_client: HttpClient,
  video_info_cache: CacheAsync<VideoInfoExtended>
}

impl YtDlpSource {
  pub fn new(yt_dlp: YtDlp, video_id: impl Into<String>, http_client: HttpClient) -> Self {
    YtDlpSource {
      yt_dlp,
      video_id: video_id.into(),
      http_client,
      video_info_cache: CacheAsync::new()
    }
  }

  pub async fn get_video_info(&mut self) -> Result<&VideoInfo, YtDlpError> {
    self.video_info_cache.try_get_or_init((&self.yt_dlp, &self.video_id)).await
      .map(|video_info_extended| &video_info_extended.video_info)
  }

  pub async fn get_metadata(&mut self) -> Result<AuxMetadata, AudioStreamError> {
    self.get_video_info().await
      .map(|video_info| video_info.clone().into_aux_metadata())
      .map_err(|err| AudioStreamError::Fail(Box::new(err)))
  }
}

impl From<YtDlpSource> for Input {
  fn from(value: YtDlpSource) -> Self {
    Input::Lazy(Box::new(value))
  }
}

#[serenity::async_trait]
impl Compose for YtDlpSource {
  fn create(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
    Err(AudioStreamError::Unsupported)
  }

  async fn create_async(&mut self) -> Result<AudioStream<Box<dyn MediaSource>>, AudioStreamError> {
    let http_client = self.http_client.clone();
    let video_info = self.get_video_info().await
      .map_err(|err| AudioStreamError::Fail(Box::new(err)))?;

    let mut headers = HeaderMap::default();
    if let Some(video_headers) = &video_info.http_headers {
      headers.extend(video_headers.iter().filter_map(|(k, v)| {
        let header_name = HeaderName::from_bytes(k.as_bytes()).ok()?;
        let header_value = HeaderValue::from_str(v).ok()?;
        Some((header_name, header_value))
      }));
    };

    let mut req = HttpRequest {
      client: http_client,
      request: video_info.url.clone(),
      content_length: video_info.filesize,
      headers
    };

    req.create_async().await
  }

  fn should_create_async(&self) -> bool {
    true
  }

  async fn aux_metadata(&mut self) -> Result<AuxMetadata, AudioStreamError> {
    self.get_metadata().await
  }
}

fn is_id_str(s: &str, l: usize) -> bool {
  s.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') && s.len() < l
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayVideoUrl<'a>(&'a str);

impl<'a> fmt::Display for DisplayVideoUrl<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "https://www.youtube.com/watch?v={}", self.0)
  }
}

#[inline]
pub fn display_video_url(id: &str) -> DisplayVideoUrl {
  DisplayVideoUrl(id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisplayPlaylistUrl<'a>(&'a str);

impl<'a> fmt::Display for DisplayPlaylistUrl<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "https://www.youtube.com/playlist?list={}", self.0)
  }
}

#[inline]
pub fn display_playlist_url(id: &str) -> DisplayPlaylistUrl {
  DisplayPlaylistUrl(id)
}

pub fn parse_video_url(url: &str) -> Option<String> {
  let url = Url::parse(url).ok()?;
  let ("http" | "https") = url.scheme() else { return None };

  let domain = url.domain()?;
  if domain.ends_with("youtube.com") {
    url.query_pairs()
      .find_map(|(k, v)| (k == "v" && is_id_str(&v, 16)).then_some(v))
      .map(std::borrow::Cow::into_owned)
      .or_else(|| {
        let path = url.path();
        ["/v/", "/embed/", "/"].into_iter()
          .filter_map(|s| path.strip_prefix(s))
          .find(|&p| is_id_str(p, 16))
          .map(str::to_owned)
      })
  } else if domain == "youtu.be" {
    url.path().strip_prefix("/")
      .filter(|&p| is_id_str(p, 16))
      .map(str::to_owned)
  } else {
    None
  }
}

pub fn parse_playlist_url(url: &str) -> Option<String> {
  let url = Url::parse(url).ok()?;
  let ("http" | "https") = url.scheme() else { return None };

  let domain = url.domain()?;
  if domain.ends_with("youtube.com") || domain == "youtu.be" {
    url.query_pairs()
      .find_map(|(k, v)| (k == "list" && is_id_str(&v, 40)).then_some(v))
      .map(std::borrow::Cow::into_owned)
  } else {
    None
  }
}
