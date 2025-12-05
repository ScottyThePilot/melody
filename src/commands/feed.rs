use crate::prelude::*;
use crate::feature::feed::{FeedIdentifier, FeedIdentifierTwitter, FeedIdentifierYouTube, RegisterFeedResult, UnregisterFeedResult};
use crate::data::Core;
use crate::utils::{Timestamp, TimestampFormat, LazyRegex};
use super::{MelodyContext, CommandMetaData};

use serenity::model::id::ChannelId;
use poise::macros::ChoiceParameter;



#[poise::command(
  slash_command,
  guild_only,
  subcommands(
    "feeds_add",
    "feeds_remove",
    "feeds_remove_all",
    "feeds_list"
  ),
  category = "feed",
  name_localized("en-US", "feeds"),
  description_localized("en-US", "Set up channels to recieve posts from social media websites or from RSS feeds"),
  default_member_permissions = "MANAGE_WEBHOOKS",
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", [
      "/feeds add <'youtube'|'twitter'> <feed-source>",
      "/feeds remove <'youtube'|'twitter'> <feed-source>",
      "/feeds remove-all",
      "/feeds list"
    ])
    .examples_localized("en-US", [
      "/feeds add youtube UC7_YxT-KID8kRbqZo7MyscQ",
      "/feeds add twitter markiplier",
      "/feeds remove twitter elonmusk",
      "/feeds remove-all",
      "/feeds list"
    ])
)]
pub async fn feeds(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::COMMAND_PRECONDITION_VIOLATION_ROOT_COMMAND)
}

#[poise::command(
  slash_command,
  guild_only,
  category = "feed",
  rename = "add",
  name_localized("en-US", "add"),
  description_localized("en-US", "Adds a feed to this server"),
  default_member_permissions = "MANAGE_WEBHOOKS",
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", [
      "/feeds add <'youtube'|'twitter'> <feed-source>"
    ])
    .examples_localized("en-US", [
      "/feeds add youtube UC7_YxT-KID8kRbqZo7MyscQ",
      "/feeds add twitter markiplier"
    ])
)]
async fn feeds_add(
  ctx: MelodyContext<'_>,
  #[rename = "feed-type"]
  #[name_localized("en-US", "feed-type")]
  #[description_localized("en-US", "The type of feed to add, YouTube or Twitter")]
  feed_type: FeedType,
  #[rename = "feed-source"]
  #[name_localized("en-US", "feed-source")]
  #[description_localized("en-US", "For YouTube feeds, the channel ID, for Twitter feeds, the account's handle")]
  #[max_length = 64]
  feed_source: String,
  #[rename = "channel"]
  #[name_localized("en-US", "channel")]
  #[description_localized("en-US", "Which channel messages for this feed should be sent")]
  #[channel_types("Text")]
  channel_id: Option<ChannelId>
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let channel_id = channel_id.unwrap_or(ctx.channel_id());

  let response = if let Some(feed_identifier) = feed_type.with_source(&feed_source) {
    if channel_is_text_based(ctx, channel_id).ok_or(MelodyError::COMMAND_NOT_IN_GUILD)? {
      match core.feed().await.register_feed(feed_identifier.clone(), guild_id, channel_id).await? {
        RegisterFeedResult::FeedChannelRegistered => {
          format!("Successfully added feed for <https://{feed_identifier}> in {}", channel_id.mention())
        },
        RegisterFeedResult::FeedChannelReplaced(old_channel_id) => {
          format!("Successfully updated feed channel for <https://{feed_identifier}> from {} to {}", old_channel_id.mention(), channel_id.mention())
        },
        RegisterFeedResult::FeedNotEnabled => {
          "Feeds of this type are disabled".to_owned()
        }
      }
    } else {
      format!("Channel {} is not text-based", channel_id.mention())
    }
  } else {
    "Failed to parse feed source".to_owned()
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  category = "feed",
  rename = "remove",
  name_localized("en-US", "remove"),
  description_localized("en-US", "Removes a feed from this server"),
  default_member_permissions = "MANAGE_WEBHOOKS",
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", [
      "/feeds remove <'youtube'|'twitter'> <feed-source>"
    ])
    .examples_localized("en-US", [
      "/feeds remove twitter elonmusk",
      "/feeds remove-all"
    ])
)]
async fn feeds_remove(
  ctx: MelodyContext<'_>,
  #[rename = "feed-type"]
  #[name_localized("en-US", "feed-type")]
  #[description_localized("en-US", "The type of feed to remove, YouTube or Twitter")]
  feed_type: FeedType,
  #[rename = "feed-source"]
  #[name_localized("en-US", "feed-source")]
  #[description_localized("en-US", "For YouTube feeds, the channel ID, for Twitter feeds, the account's handle")]
  #[max_length = 64]
  feed_source: String
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let response = if let Some(feed_identifier) = feed_type.with_source(&feed_source) {
    match core.feed().await.unregister_feed(&feed_identifier, guild_id).await? {
      UnregisterFeedResult::FeedUnregistered(channel_id) | UnregisterFeedResult::FeedChannelUnregistered(channel_id) => {
        format!("Successfully removed feed for <https://{feed_identifier}> in {}", channel_id.mention())
      },
      UnregisterFeedResult::FeedNotRegistered | UnregisterFeedResult::FeedChannelNotRegistered => {
        "Found no such feed for this server".to_owned()
      }
    }
  } else {
    "Failed to parse feed source".to_owned()
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  category = "feed",
  rename = "remove-all",
  name_localized("en-US", "remove-all"),
  description_localized("en-US", "Removes all feeds from this server"),
  default_member_permissions = "MANAGE_WEBHOOKS",
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/feeds remove-all"])
    .examples_localized("en-US", ["/feeds remove-all"])
)]
async fn feeds_remove_all(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let removed = core.feed().await.unregister_guild_feeds(guild_id).await?;

  let response = match removed {
    0 => "No feeds to remove".to_owned(),
    r => format!("Successfully removed {r} feeds")
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  category = "feed",
  rename = "list",
  name_localized("en-US", "list"),
  description_localized("en-US", "Lists all feeds in this server"),
  default_member_permissions = "MANAGE_WEBHOOKS",
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/feeds list"])
    .examples_localized("en-US", ["/feeds list"])
)]
async fn feeds_list(
  ctx: MelodyContext<'_>
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let feeds = core.feed().await.get_guild_feeds(guild_id).await;

  let response = if feeds.is_empty() {
    "No feeds to show".to_owned()
  } else {
    feeds.into_iter()
      .map(|(feed, channel_id, last_update)| {
        let last_update = Timestamp::new(last_update, TimestampFormat::ShortDateTime);
        format!("<https://{feed}> for {}, last entry {last_update}", channel_id.mention())
      })
      .join("\n")
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ChoiceParameter, Serialize, Deserialize)]
pub enum FeedType {
  #[name = "YouTube"]
  #[name_localized("en-US", "YouTube")]
  YouTube,
  #[name = "Twitter"]
  #[name_localized("en-US", "Twitter")]
  Twitter
}

impl FeedType {
  pub fn with_source(self, source: &str) -> Option<FeedIdentifier> {
    static RX_YOUTUBE_CHANNEL: LazyRegex = LazyRegex::new(r"^(UC[0-9A-Za-z_-]{21}[AQgw]{1})$");
    static RX_TWITTER_HANDLE: LazyRegex = LazyRegex::new(r"^@?([A-Za-z0-9_]{1,15})$");
    match self {
      Self::YouTube => {
        let source = RX_YOUTUBE_CHANNEL.captures(source)?.get(1)?.as_str();
        Some(FeedIdentifier::YouTube(FeedIdentifierYouTube { channel: source.to_owned() }))
      },
      Self::Twitter => {
        let source = RX_TWITTER_HANDLE.captures(source)?.get(1)?.as_str();
        Some(FeedIdentifier::Twitter(FeedIdentifierTwitter { handle: source.to_owned() }))
      }
    }
  }
}

fn channel_is_text_based(ctx: MelodyContext<'_>, channel_id: ChannelId) -> Option<bool> {
  ctx.guild().map(|guild| guild.channels.get(&channel_id).is_some_and(|channel| channel.is_text_based()))
}
