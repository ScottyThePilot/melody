use crate::prelude::*;
use crate::feature::feed::{Feed, FeedState};
use crate::data::Core;
use crate::utils::{Timestamp, TimestampFormat, LazyRegex};
use super::{MelodyContext, CommandMetaData};

use chrono::{DateTime, Utc};
use serenity::model::id::{ChannelId, GuildId};
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
  Err(MelodyError::command_precondition_violation("root command"))
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

  // validate that channel id is a text channel

  let response = if let Some(feed) = feed_type.with_source(&feed_source) {
    let (was_replaced, is_disabled) = register_feed(&core, feed.clone(), guild_id, channel_id).await?;
    let mut response = format!("Successfully added feed for <https://{feed}> in {}", channel_id.mention());
    if was_replaced { response.push_str(", replacing an existing identical feed") };
    if is_disabled { response.push_str("\n(Please note that feeds of this type are currently disabled)") }
    response
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

  let response = if let Some(feed) = feed_type.with_source(&feed_source) {
    match unregister_feed(&core, &feed, guild_id).await? {
      Some(channel_id) => format!("Successfully removed feed for <https://{feed}> in {}", channel_id.mention()),
      None => "Found no such feed for this server".to_owned()
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
  let removed = unregister_guild_feeds(&core, guild_id).await?;

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
  let feeds = list_guild_feeds(&core, guild_id).await;

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

async fn register_feed(core: &Core, feed: Feed, guild_id: GuildId, channel_id: ChannelId) -> MelodyResult<(bool, bool)> {
  // Create a new entry in the persist file's feeds table
  let was_replaced = core.operate_persist_commit(|persist| Ok({
    persist.feeds.entry(feed.clone()).or_insert_with(FeedState::new)
      .guilds.insert(guild_id, channel_id).is_some()
  })).await?;

  // Ensure the feed has its task spawned (if possible)
  let feed_wrapper = core.state.feed.clone();
  let is_disabled = feed_wrapper.lock().await.register(&core, feed).await.is_none();

  Ok((was_replaced, is_disabled))
}

async fn unregister_feed(core: &Core, feed: &Feed, guild_id: GuildId) -> MelodyResult<Option<ChannelId>> {
  let (should_abort, result) = core.operate_persist_commit(|persist| {
    let (should_abort, result) = persist.feeds.get_mut(feed).map_or((false, None), |feed_state| {
      match feed_state.guilds.remove(&guild_id) {
        // the feed task will only be aborted when this guild was the only remaining guild
        Some(channel_id) => (feed_state.guilds.is_empty(), Some(channel_id)),
        None => (false, None)
      }
    });

    // when there are no guilds left, the feed entry should be removed entirely
    // this erases the last updated timestamp, preventing a situation where a previously registered feed is
    // registered again, and the bot floods the channel with events that had happened since the (erroneous) last update
    if should_abort { persist.feeds.remove(&feed); };
    Ok((should_abort, result))
  }).await?;

  if should_abort {
    let feed_wrapper = core.state.feed.clone();
    feed_wrapper.lock().await.abort(feed);
  };

  Ok(result)
}

async fn unregister_guild_feeds(core: &Core, guild_id: GuildId) -> MelodyResult<usize> {
  let removed = core.operate_persist_commit(|persist| {
    let mut removed = Vec::new();
    persist.feeds.retain(|feed, feed_state| {
      feed_state.guilds.remove(&guild_id);
      if feed_state.guilds.is_empty() {
        removed.push(feed.clone());
        false
      } else {
        true
      }
    });

    Ok(removed)
  }).await?;

  if !removed.is_empty() {
    let feed_wrapper = core.state.feed.clone();
    for feed in removed.iter() {
      feed_wrapper.lock().await.abort(feed);
    };
  };

  Ok(removed.len())
}

async fn list_guild_feeds(core: &Core, guild_id: GuildId) -> Vec<(Feed, ChannelId, DateTime<Utc>)> {
  core.operate_persist(|persist| {
    persist.feeds.iter().filter_map(|(feed, feed_state)| {
      feed_state.guilds.get(&guild_id).map(|&channel_id| {
        (feed.clone(), channel_id, feed_state.last_update)
      })
    }).collect()
  }).await
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
  pub fn with_source(self, source: &str) -> Option<Feed> {
    static RX_YOUTUBE_CHANNEL: LazyRegex = LazyRegex::new(r"^(UC[0-9A-Za-z_-]{21}[AQgw]{1})$");
    static RX_TWITTER_HANDLE: LazyRegex = LazyRegex::new(r"^@?([A-Za-z0-9_]{1,15})$");
    match self {
      Self::YouTube => {
        let source = RX_YOUTUBE_CHANNEL.captures(source)?.get(1)?.as_str();
        Some(Feed::YouTube { channel: source.to_owned() })
      },
      Self::Twitter => {
        let source = RX_TWITTER_HANDLE.captures(source)?.get(1)?.as_str();
        Some(Feed::Twitter { handle: source.to_owned() })
      }
    }
  }
}
