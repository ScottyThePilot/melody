use crate::MelodyResult;
use crate::data::Core;
use crate::ratelimiter::RateLimiter;
use crate::utils::Contextualize;

pub use cleverbot::Error as CleverBotError;
use cleverbot::{CleverBotAgent, CleverBotContext};
use serenity::builder::CreateEmbed;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;

use std::collections::HashMap;
use std::time::Duration;



const DELAY: Duration = Duration::from_secs(3);

pub type CleverBotWrapper = RateLimiter<CleverBotManager>;

impl CleverBotWrapper {
  pub async fn send(&self, channel: ChannelId, message: &str) -> Result<String, CleverBotError> {
    self.get().await.send(channel, message).await
  }
}

impl From<CleverBotManager> for CleverBotWrapper {
  fn from(manager: CleverBotManager) -> Self {
    RateLimiter::new(manager, DELAY)
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

pub async fn send_reply(core: &Core, message: &Message, content: impl AsRef<str>) -> MelodyResult {
  // whether or not to notify the user that this message is from cleverbot
  let notify = core.operate_persist_commit(|persist| {
    Ok(persist.cleverbot_notify(message.author.id))
  }).await?;

  message.channel_id
    .send_message(&core, |builder| {
      // reply to the original message
      builder.reference_message(message).allowed_mentions(|mentions| {
        mentions.replied_user(true)
          .parse(serenity::builder::ParseValue::Everyone)
          .parse(serenity::builder::ParseValue::Users)
          .parse(serenity::builder::ParseValue::Roles)
      });

      if notify {
        builder.set_embed(cleverbot_note_embed());
      };

      builder.content(content.as_ref())
    })
    .await.context("failed to send cleverbot reply")?;

  Ok(())
}

fn cleverbot_note_embed() -> CreateEmbed {
  let mut embed = CreateEmbed::default();
  embed.title("Please note");
  embed.description("Melody's chatbot responses are from CleverBot. If you send messages too quickly, you'll be ratelimited.");
  embed.footer(|embed_footer| {
    embed_footer.text("You're seeing this because it's your first time using this feature.")
  });

  embed
}
