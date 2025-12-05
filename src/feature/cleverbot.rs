use crate::prelude::*;
use crate::data::Core;

pub use cleverbot::Error as CleverBotError;
pub use cleverbot_logs::Error as CleverBotLogError;
use cleverbot::{CleverBotAgent, CleverBotContext};
use cleverbot_logs::{CleverBotLogger, CleverBotLogEntry};
use chrono::Utc;
use melody_ratelimiter::RateLimiter;
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use tokio::sync::Mutex;

use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;



/// Mentions targeting the current user will be replaced with this
pub const CLEVERBOT_CANONICAL_NAME: &str = "CleverBot";

#[derive(Debug, Clone)]
pub struct CleverBotLoggerWrapper {
  ptr: Arc<Mutex<CleverBotLogger>>
}

impl CleverBotLoggerWrapper {
  pub async fn create() -> Result<Self, CleverBotLogError> {
    let logger = tokio::task::spawn_blocking(|| {
      CleverBotLogger::create(format!("./data/cleverbot.log"))
    }).await.unwrap()?;

    Ok(CleverBotLoggerWrapper {
      ptr: Arc::new(Mutex::new(logger))
    })
  }

  pub async fn log(
    self,
    channel_id: ChannelId,
    message: impl Into<String>,
    response: impl Into<String>
  ) -> Result<(), CleverBotLogError> {
    let message = message.into();
    let response = response.into();
    let guard = self.ptr.lock_owned().await;
    tokio::task::spawn_blocking(move || {
      guard.log(&CleverBotLogEntry {
        thread: channel_id.into(),
        time: Utc::now(),
        message,
        response
      })
    }).await.unwrap()
  }
}

#[derive(Debug, Clone)]
pub struct CleverBotWrapper {
  ratelimiter: RateLimiter<CleverBotManager>
}

impl CleverBotWrapper {
  pub fn new(delay: Duration) -> Self {
    CleverBotWrapper {
      ratelimiter: RateLimiter::new(CleverBotManager::new(), delay)
    }
  }

  pub async fn send(&self, channel: ChannelId, message: &str) -> Result<String, CleverBotError> {
    self.ratelimiter.get().await.send(channel, message).await
  }
}

#[derive(Debug)]
pub struct CleverBotManager {
  agent: CleverBotAgent,
  channels: HashMap<ChannelId, CleverBotContext>
}

impl CleverBotManager {
  pub fn new() -> Self {
    CleverBotManager {
      agent: CleverBotAgent::new(),
      channels: HashMap::new()
    }
  }

  pub async fn send(&mut self, channel: ChannelId, message: &str) -> Result<String, CleverBotError> {
    self.channels.entry(channel).or_default().send(&mut self.agent, message).await
  }
}

pub async fn send_reply(core: &Core, message: &Message, content: impl Into<String>) -> MelodyResult {
  // whether or not to notify the user that this message is from cleverbot
  let notify = core.operate_persist_commit(async |persist| {
    Ok(persist.cleverbot_notify(message.author.id))
  }).await?;

  let message_builder = CreateMessage::new()
    .allowed_mentions(Default::default())
    .embeds(if notify { vec![cleverbot_note_embed()] } else { Vec::new() })
    .reference_message(message)
    .content(content);
  message.channel_id.send_message(&core, message_builder)
    .await.context("failed to send cleverbot reply")?;

  Ok(())
}

fn cleverbot_note_embed() -> CreateEmbed {
  CreateEmbed::default()
    .title("Please note")
    .description("Melody's chatbot responses are from CleverBot. If you send messages too quickly, you'll be ratelimited.")
    .footer(CreateEmbedFooter::new("You're seeing this because it's your first time using this feature."))
}
