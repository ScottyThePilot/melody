use crate::{MelodyError, MelodyResult};
use crate::blueprint::*;
use crate::feature::feed::{Feed, FeedState};
use crate::data::Core;
use crate::utils::{Timestamp, TimestampFormat};

use chrono::{DateTime, Utc};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::model::Permissions;
use serenity::model::id::{ChannelId, GuildId};
use serenity::model::mention::Mentionable;



pub const FEEDS: BlueprintCommand = blueprint_command! {
  name: "feeds",
  description: "Set up channels to recieve posts from social media websites or from RSS feeds",
  usage: [
    "/feeds add <'youtube'|'twitter'> <feed-source>",
    "/feeds remove <'youtube'|'twitter'> <feed-source>",
    "/feeds remove-all",
    "/feeds list"
  ],
  examples: [
    "/feeds add youtube UC7_YxT-KID8kRbqZo7MyscQ",
    "/feeds add twitter markiplier",
    "/feeds remove twitter elonmusk",
    "/feeds remove-all",
    "/feeds list"
  ],
  plugin: "feed",
  allow_in_dms: false,
  default_permissions: Permissions::MANAGE_WEBHOOKS,
  subcommands: [
    blueprint_subcommand! {
      name: "add",
      description: "Adds a feed to this server",
      arguments: [
        FEED_SOURCE_ARGUMENT,
        blueprint_argument!(String {
          name: "feed-source",
          description: "For YouTube feeds, the channel ID, for Twitter feeds, the account's handle",
          required: true,
          max_length: 64
        }),
        blueprint_argument!(Channel {
          name: "channel",
          description: "Which channel messages for this feed should be sent",
          required: false
        })
      ],
      function: feeds_add
    },
    blueprint_subcommand! {
      name: "remove",
      description: "Removes a feed from this server",
      arguments: [
        FEED_SOURCE_ARGUMENT,
        blueprint_argument!(String {
          name: "feed-source",
          description: "For YouTube feeds, the channel ID, for Twitter feeds, the account's handle",
          required: true,
          max_length: 64
        })
      ],
      function: feeds_remove
    },
    blueprint_subcommand! {
      name: "remove-all",
      description: "Removes all feeds from this server",
      arguments: [],
      function: feeds_remove_all
    },
    blueprint_subcommand! {
      name: "list",
      description: "Lists all feeds in this server",
      arguments: [],
      function: feeds_list
    }
  ]
};

async fn feeds_add(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let (feed_type, feed_source, channel_id) = args.resolve_values::<(String, String, Option<ChannelId>)>()?;
  let feed_type = FeedType::from_str(&feed_type).ok_or(MelodyError::COMMAND_INVALID_ARGUMENTS_STRUCTURE)?;
  let channel_id = channel_id.unwrap_or(args.interaction.channel_id);

  let response = if let Some(feed) = feed_type.with_source(&feed_source) {
    let (was_replaced, is_disabled) = register_feed(&core, feed.clone(), guild_id, channel_id).await?;
    let mut response = format!("Successfully added feed for <https://{feed}> in {}", channel_id.mention());
    if was_replaced { response.push_str(", replacing an existing identical feed") };
    if is_disabled { response.push_str("\n(Please note that feeds of this type are currently disabled)") }
    response
  } else {
    "Failed to parse feed source".to_owned()
  };

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

async fn feeds_remove(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let (feed_type, feed_source) = args.resolve_values::<(String, String)>()?;
  let feed_type = FeedType::from_str(&feed_type).ok_or(MelodyError::COMMAND_INVALID_ARGUMENTS_STRUCTURE)?;

  let response = if let Some(feed) = feed_type.with_source(&feed_source) {
    match unregister_feed(&core, &feed, guild_id).await? {
      Some(channel_id) => format!("Successfully removed feed for <https://{feed}> in {}", channel_id.mention()),
      None => "Found no such feed for this server".to_owned()
    }
  } else {
    "Failed to parse feed source".to_owned()
  };

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

async fn feeds_remove_all(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let removed = unregister_guild_feeds(&core, guild_id).await?;

  let response = match removed {
    0 => "No feeds to remove".to_owned(),
    r => format!("Successfully removed {r} feeds")
  };

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

async fn feeds_list(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
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

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

async fn register_feed(core: &Core, feed: Feed, guild_id: GuildId, channel_id: ChannelId) -> MelodyResult<(bool, bool)> {
  // Create a new entry in the persist file's feeds table
  let was_replaced = core.operate_persist_commit(|persist| Ok({
    persist.feeds.entry(feed.clone()).or_insert_with(FeedState::new)
      .guilds.insert(guild_id, channel_id).is_some()
  })).await?;

  // Ensure the feed has its task spawned (if possible)
  let feed_wrapper = core.get::<crate::data::FeedKey>().await;
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
    operate!(core, operate_lock::<crate::data::FeedKey>, |feed_manager| {
      feed_manager.abort(feed);
    });
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
    operate!(core, operate_lock::<crate::data::FeedKey>, |feed_manager| {
      for feed in removed.iter() {
        feed_manager.abort(feed);
      };
    });
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

const FEED_SOURCE_ARGUMENT: BlueprintOption = blueprint_argument!(String {
  name: "feed-type",
  description: "What type this feed should be",
  required: true,
  choices: [
    ("twitter", "twitter"),
    ("youtube", "youtube")
  ]
});

#[repr(u8)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FeedType {
  YouTube,
  Twitter
}

impl FeedType {
  pub fn from_str(s: &str) -> Option<Self> {
    match s {
      "youtube" => Some(Self::YouTube),
      "twitter" => Some(Self::Twitter),
      _ => None
    }
  }

  pub fn with_source(self, source: &str) -> Option<Feed> {
    static RX_YOUTUBE_CHANNEL: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(UC[0-9A-Za-z_-]{21}[AQgw]{1})$").unwrap());
    static RX_TWITTER_HANDLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^@?([A-Za-z0-9_]{1,15})$").unwrap());
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
