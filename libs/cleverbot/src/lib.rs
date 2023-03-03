extern crate chrono;
extern crate md5;
extern crate percent_encoding;
extern crate reqwest;
#[macro_use]
extern crate thiserror;

use chrono::{DateTime, Utc};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_encode};
use reqwest::{Client, Proxy, Error as ReqwestError};

use std::collections::VecDeque;

const USER_AGENT: &str = "Opera/9.48 (Windows NT 6.0; sl-SI) Presto/2.11.249 Version/10.00";



/// Facilitates communication with CleverBot by imitating the website's functionality.
/// A single instance of this should be treated as if it were a single instance of your browser.
pub struct CleverBotAgent {
  pub data: Option<CleverBotData>,
  pub client: Client
}

impl CleverBotAgent {
  pub fn new() -> Self {
    let client = Client::builder()
      .user_agent(USER_AGENT)
      .build().unwrap();
    Self::with_client(client)
  }

  pub fn with_proxy(proxy: Proxy, user_agent: impl AsRef<str>) -> Self {
    let client = Client::builder()
      .user_agent(user_agent.as_ref())
      .proxy(proxy)
      .build().unwrap();
    Self::with_client(client)
  }

  pub fn with_client(client: Client) -> Self {
    CleverBotAgent { data: None, client }
  }

  async fn init(&mut self) -> Result<(&mut CleverBotData, &Client), Error> {
    let now = Utc::now();
    Result::map(match self.data {
      Some(ref mut data) => data.ensure_valid(&self.client, now).await.map(|()| data),
      None => CleverBotData::request(&self.client, now).await.map(|data| self.data.insert(data))
    }, |data| (data, &self.client))
  }
}

impl Default for CleverBotAgent {
  fn default() -> Self {
    Self::new()
  }
}



#[derive(Debug, Clone)]
struct CleverBotPreviousData {
  cbsid: String,
  xai: String,
  last_reply: String
}

impl CleverBotPreviousData {
  fn new(response: &str) -> Option<Self> {
    let mut contents = response.split('\r');
    let reply = contents.next()?.to_owned();
    let cbsid = contents.next()?.to_owned();
    let xai = format!("{},{}", &cbsid[0..3], contents.next()?);
    Some(CleverBotPreviousData { cbsid, xai, last_reply: reply })
  }
}

#[derive(Debug, Clone)]
pub struct CleverBotData {
  xvis: String,
  previous: Option<CleverBotPreviousData>,
  max_age: i64,
  last_update: DateTime<Utc>
}

impl CleverBotData {
  fn new(cookie: &str, last_update: DateTime<Utc>) -> Option<Self> {
    let mut contents = cookie.split(';');
    let contents = (contents.next()?, contents.next(), contents.next());
    let max_age = contents.2.and_then(parse_max_age).unwrap_or(86400000);
    Some(CleverBotData {
      xvis: contents.0.to_owned(),
      previous: None,
      max_age, last_update
    })
  }

  pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
    now.signed_duration_since(self.last_update).num_milliseconds() > self.max_age
  }

  async fn ensure_valid(&mut self, client: &Client, now: DateTime<Utc>) -> Result<(), Error> {
    if self.is_expired(now) {
      Self::request(client, now).await.map(|agent| *self = agent)
    } else {
      Ok(())
    }
  }

  async fn request(client: &Client, now: DateTime<Utc>) -> Result<Self, Error> {
    let date = now.format("%Y%m%d");
    let url = format!("https://www.cleverbot.com/extras/conversation-social-min.js?{date}");
    let response = client.get(url).send().await?;
    let cookie = response.headers().get("set-cookie")
      .and_then(|value| value.to_str().ok())
      .ok_or(Error::MissingCookie)?;
    CleverBotData::new(cookie, now)
      .ok_or_else(|| Error::InvalidCookie(cookie.to_owned()))
  }
}

fn parse_max_age(max_age: &str) -> Option<i64> {
  max_age.strip_prefix("Max-Age=")?.parse::<i64>().ok()
}

/// Saves the user's conversation history for convenience.
pub struct CleverBotContext {
  pub history: VecDeque<String>,
  pub history_size_limit: usize
}

impl CleverBotContext {
  pub fn new() -> Self {
    Self::with_size_limit(32)
  }

  pub fn with_size_limit(history_size_limit: usize) -> Self {
    CleverBotContext {
      history: VecDeque::new(),
      history_size_limit
    }
  }

  pub async fn send(&mut self, agent: &mut CleverBotAgent, message: &str) -> Result<String, Error> {
    let reply = send(agent, self.history.make_contiguous(), message).await?;
    self.history.push_back(message.to_owned());
    self.history.push_back(reply.clone());
    while self.history.len() > self.history_size_limit {
      self.history.pop_front();
    };

    Ok(reply)
  }
}

impl Default for CleverBotContext {
  fn default() -> Self {
    CleverBotContext::new()
  }
}

pub async fn send(agent: &mut CleverBotAgent, history: &[String], message: &str) -> Result<String, Error> {
  let (agent_data, agent_client) = agent.init().await?;

  let mut payload = String::new();
  payload.push_str(&format!("stimulus={}&", escape(message.as_bytes())));
  for (i, history_message) in history.iter().rev().enumerate() {
    payload.push_str(&format!("vText{}={}", i + 2, escape(history_message.as_bytes())))
  };

  payload.push_str("cb_settings_scripting=no&islearning=1&icognoid=wsf&icognocheck=");
  payload.push_str(&format!("{:x}", md5::compute(&payload[7..33])));

  let mut url = "https://www.cleverbot.com/webservicemin?uc=UseOfficialCleverbotAPI".to_owned();
  if let Some(CleverBotPreviousData { cbsid, xai, last_reply }) = &agent_data.previous {
    let last_reply = encode_uri_component(last_reply.as_bytes());
    let stimulus = encode_uri_component(message.as_bytes());
    url.push_str(&format!("&out={last_reply}&in={stimulus}&bot=c&cbsid={cbsid}&xai={xai}"));
    url.push_str("&ns=2&al=&dl=&flag=&user=&mode=1&alt=0&reac=&emo=&sou=website&xed=&");
  };

  let response = agent_client.post(url)
    .header("Cookie", format!("{}; _cbsid=-1", agent_data.xvis))
    .header("enctype", "text/plain")
    .body(payload)
    .send().await?;
  let response_text = response.text().await?;
  let previous_data = CleverBotPreviousData::new(&response_text)
    .ok_or_else(|| Error::InvalidResponse(response_text))?;
  let reply = previous_data.last_reply.clone();
  agent_data.previous = Some(previous_data);
  Ok(reply)
}

macro_rules! ascii_set {
  ($set:expr, $method:ident, [$($char:literal),*]) => ($set$(.$method($char))*);
}

fn encode_uri_component(input: &[u8]) -> impl std::fmt::Display + '_ {
  const ASCII_SET: AsciiSet = ascii_set!(NON_ALPHANUMERIC, remove, [
    b'!', b'\'', b'(', b')', b'*', b'-', b'.', b'_', b'~'
  ]);

  percent_encode(input, &ASCII_SET)
}

fn escape(input: &[u8]) -> impl std::fmt::Display + '_ {
  const ASCII_SET: AsciiSet = ascii_set!(NON_ALPHANUMERIC, remove, [
    b'*', b'+', b'-', b'.', b'/', b'@', b'_'
  ]);

  percent_encode(input, &ASCII_SET)
}

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  ReqwestError(#[from] ReqwestError),
  #[error("invalid cookie: {0:?}")]
  InvalidCookie(String),
  #[error("missing cookie")]
  MissingCookie,
  #[error("invalid response: {0:?}")]
  InvalidResponse(String)
}
