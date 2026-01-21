use crate::prelude::*;
use crate::data::{Core, State};
use crate::feature::music_player::{MusicPlayer, QueueItem, AttachmentItem, YouTubeItem};
use crate::utils::youtube;
use super::{MelodyContext, CommandMetaData};

use serenity::model::id::{ChannelId, GuildId, UserId};
use serenity::model::channel::Attachment;
use melody_framework::commands::CommandConditionFunction;

use std::sync::Arc;



#[poise::command(
  slash_command,
  guild_only,
  subcommands(
    "music_player_play",
    "music_player_queue",
    "music_player_join",
    "music_player_leave",
    "music_player_pause",
    "music_player_loop",
    "music_player_skip",
    "music_player_stop",
    "music_player_kill"
  ),
  category = "music-player",
  rename = "music-player",
  name_localized("en-US", "music-player"),
  description_localized("en-US", "Play audio from various sources in voice channels"),
  custom_data = CommandMetaData::new()
    .info_localized_concat("en-US", [
      "Player functionality may be spotty or unreliable.",
      "If the player breaks or stops on a track indefinitely, command the bot to skip, leave the channel, and join the channel again.",
      "Issuing the stop command will clear the queue, the leave command will not."
    ])
    .usage_localized("en-US", [
      "/music-player play youtube <video-url>",
      "/music-player play youtube-playlist <playlist-url>",
      "/music-player play attachment <attachment>",
      "/music-player queue show [page]",
      "/music-player queue clear",
      "/music-player queue remove <index>",
      "/music-player queue shuffle",
      "/music-player join",
      "/music-player leave",
      "/music-player pause <true|false>",
      "/music-player loop <true|false>",
      "/music-player skip",
      "/music-player stop",
      "/music-player kill"
    ])
    .examples_localized("en-US", [
      "/music-player play youtube 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'",
      "/music-player play youtube-playlist 'https://www.youtube.com/playlist?list=OLAK5uy_kZx-hTxk_EfczAhOP3eQT-kJlBsH7NJXs'",
      "/music-player queue show",
      "/music-player queue clear",
      "/music-player queue remove 3",
      "/music-player queue shuffle",
      "/music-player join",
      "/music-player leave",
      "/music-player pause false",
      "/music-player loop true",
      "/music-player skip",
      "/music-player stop",
      "/music-player kill"
    ])
    .condition(CommandConditionFunction::new_downcast(|state: Option<&State>| {
      state.is_some_and(|state| state.music_player.is_some())
    }))
)]
pub async fn music_player(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::COMMAND_PRECONDITION_VIOLATION_ROOT_COMMAND)
}

#[poise::command(
  slash_command,
  guild_only,
  subcommands(
    "music_player_play_youtube",
    "music_player_play_youtube_playlist",
    "music_player_play_attachment"
  ),
  category = "music-player",
  rename = "play",
  name_localized("en-US", "play"),
  description_localized("en-US", "Adds a track to the queue to be played in your voice channel"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", [
      "/music-player play youtube <video-url>",
      "/music-player play youtube-playlist <playlist-url>",
      "/music-player play attachment <attachment>"
    ])
    .examples_localized("en-US", [
      "/music-player play youtube 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'",
      "/music-player play youtube-playlist 'https://www.youtube.com/playlist?list=OLAK5uy_kZx-hTxk_EfczAhOP3eQT-kJlBsH7NJXs'"
    ])
)]
async fn music_player_play(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::COMMAND_PRECONDITION_VIOLATION_ROOT_COMMAND)
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "youtube",
  name_localized("en-US", "youtube"),
  description_localized("en-US", "Plays a YouTube video in your voice channel"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", [
      "/music-player play youtube <video-url>"
    ])
    .examples_localized("en-US", [
      "/music-player play youtube 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'"
    ])
)]
async fn music_player_play_youtube(
  ctx: MelodyContext<'_>,
  #[rename = "video-url"]
  #[name_localized("en-US", "video-url")]
  #[description_localized("en-US", "The URL of the YouTube video to play")]
  #[max_length = 256]
  video_url: String
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_id = ctx.author().id;

  send_response_result(ctx, {
    match ensure_in_channel(&core, guild_id, user_id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => match youtube::parse_video_url(&video_url) {
        None => Err("Invalid YouTube link".to_owned()),
        Some(video_id) => Ok({
          let item = QueueItem::YouTube(YouTubeItem { id: video_id });
          let item_str = item.to_string();

          ctx.defer().await.context("failed to defer response")?;

          match music_player.play(&core, guild_id, channel_id, vec![item]).await {
            Ok(()) => format!("Added video {item_str} to queue"),
            Err(err) => {
              error!("failed to connect to channel: {err}");
              "Failed to connect to channel".to_owned()
            }
          }
        })
      }
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "youtube-playlist",
  name_localized("en-US", "youtube-playlist"),
  description_localized("en-US", "Plays a YouTube playlist in your voice channel"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", [
      "/music-player play youtube-playlist <playlist-url>"
    ])
    .examples_localized("en-US", [
      "/music-player play youtube-playlist 'https://www.youtube.com/playlist?list=OLAK5uy_kZx-hTxk_EfczAhOP3eQT-kJlBsH7NJXs'"
    ])
)]
async fn music_player_play_youtube_playlist(
  ctx: MelodyContext<'_>,
  #[rename = "playlist-url"]
  #[name_localized("en-US", "playlist-url")]
  #[description_localized("en-US", "The URL of the YouTube playlist to play")]
  #[max_length = 1000]
  playlist_url: String,
  #[name_localized("en-US", "internal-command-text")]
  #[description_localized("en-US", "Whether to randomly shuffle the playlist or not")]
  #[max_length = 1000]
  shuffle: Option<bool>
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_id = ctx.author().id;
  let shuffle = shuffle.unwrap_or(false);

  send_response_result(ctx, {
    match ensure_in_channel(&core, guild_id, user_id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => match youtube::parse_playlist_url(&playlist_url) {
        None => Err("Invalid YouTube link".to_owned()),
        Some(playlist_id) => Ok({
          ctx.defer().await.context("failed to defer response")?;

          match music_player.yt_dlp().get_playlist_info(&playlist_id).await {
            Ok(playlist_info) => {
              let mut items = playlist_info.entries.into_iter()
                .map(|video_info| QueueItem::YouTube(YouTubeItem { id: video_info.id }))
                .collect::<Vec<QueueItem>>();
              let items_count = items.len();

              if shuffle {
                items.shuffle_default();
              };

              match music_player.play(&core, guild_id, channel_id, items).await {
                Ok(()) => format!("Added playlist of {items_count} videos to queue"),
                Err(err) => {
                  error!("failed to connect to channel: {err}");
                  "Failed to connect to channel".to_owned()
                }
              }
            },
            Err(err) => {
              error!("failed to retrieve playlist: {err}");
              "Failed to retrieve playlist".to_owned()
            }
          }
        })
      }
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "attachment",
  name_localized("en-US", "attachment"),
  description_localized("en-US", "Plays an audio file attachment in your voice channel"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player play attachment <attachment>"])
)]
async fn music_player_play_attachment(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "attachment")]
  #[description_localized("en-US", "The audio file attachment to play")]
  attachment: Attachment
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_id = ctx.author().id;
  let item = QueueItem::Attachment(AttachmentItem {
    id: attachment.id,
    filename: attachment.filename.clone(),
    filesize: attachment.size,
    url: attachment.url.clone()
  });
  let item_str = item.to_string();

  send_response_result(ctx, {
    match ensure_in_channel(&core, guild_id, user_id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => Ok({
        ctx.defer().await.context("failed to defer response")?;

        match music_player.play(&core, guild_id, channel_id, vec![item]).await {
          Ok(()) => format!("Added attachment {item_str} to queue"),
          Err(err) => {
            error!("failed to connect to channel: {err}");
            "Failed to connect to channel".to_owned()
          }
        }

      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  subcommands(
    "music_player_queue_show",
    "music_player_queue_clear",
    "music_player_queue_shuffle",
    "music_player_queue_remove"
  ),
  category = "music-player",
  rename = "queue",
  name_localized("en-US", "queue"),
  description_localized("en-US", "Commands relating to the music player queue"),
  custom_data = CommandMetaData::new()
    .info_localized_concat("en-US", [
      "Player functionality may be spotty or unreliable.",
      "If the player breaks or stops on a track indefinitely, command the bot to skip, leave the channel, and join the channel again.",
      "Issuing the stop command will clear the queue, the leave command will not."
    ])
    .usage_localized("en-US", [
      "/music-player queue show [page]",
      "/music-player queue clear",
      "/music-player queue remove <index>",
      "/music-player queue shuffle"
    ])
    .examples_localized("en-US", [
      "/music-player queue show",
      "/music-player queue clear",
      "/music-player queue remove 3",
      "/music-player queue shuffle"
    ])
)]
async fn music_player_queue(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::COMMAND_PRECONDITION_VIOLATION_ROOT_COMMAND)
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "show",
  name_localized("en-US", "show"),
  description_localized("en-US", "Displays the full list of tracks in the queue"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player queue show [page]"])
    .examples_localized("en-US", ["/music-player queue show"])
)]
async fn music_player_queue_show(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "page")]
  #[description_localized("en-US", "The page of the queue to display (results are grouped 10 at a time)")]
  #[min = 1]
  #[max = 65536]
  page: Option<usize>
) -> MelodyResult {
  const PER_PAGE: usize = 10;

  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let page = page.unwrap_or(1) - 1;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        let (queue_list, queue_looped) = music_player.queue_list(guild_id).await;

        let page_start = page * PER_PAGE;
        let entries = queue_list.into_iter()
          .enumerate().skip(page_start).take(PER_PAGE)
          .map(|(i, queue_item)| match i == 0 {
            true => format!("Now Playing {queue_item}"),
            false => format!("`#{}` {queue_item}", i)
          })
          .collect::<Vec<String>>();
        if entries.is_empty() {
          "(Nothing is playing)".to_owned()
        } else {
          let mut response = entries.join("\n");
          if queue_looped {
            response.push_str("\n(The queue will automatically loop)");
          };

          response
        }
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "clear",
  name_localized("en-US", "clear"),
  description_localized("en-US", "Clears the queue, except for the currently playing track"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player queue clear"])
    .examples_localized("en-US", ["/music-player queue clear"])
)]
async fn music_player_queue_clear(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        music_player.queue_clear_keep_one(guild_id).await;

        "Cleared the queue".to_owned()
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "shuffle",
  name_localized("en-US", "shuffle"),
  description_localized("en-US", "Shuffles the queue, except for the currently playing track"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player queue shuffle"])
    .examples_localized("en-US", ["/music-player queue shuffle"])
)]
async fn music_player_queue_shuffle(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        music_player.queue_shuffle(guild_id).await;

        "Shuffled the queue".to_owned()
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "remove",
  name_localized("en-US", "remove"),
  description_localized("en-US", "Removes an item from the queue"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player queue remove <index>"])
    .examples_localized("en-US", ["/music-player queue remove 3"])
)]
async fn music_player_queue_remove(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "index")]
  #[description_localized("en-US", "The index of the item in the queue to remove")]
  #[min = 1]
  #[max = 65536]
  index: Option<usize>
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let index = zusize::new(index.unwrap_or(1))
    .ok_or(MelodyError::COMMAND_PRECONDITION_VIOLATION_ARGUMENTS)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        match music_player.queue_remove(guild_id, index).await {
          Some(item) => format!("Removed item {item} from position {index} in queue"),
          None => format!("No item at position {index} in queue")
        }
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "join",
  name_localized("en-US", "join"),
  description_localized("en-US", "Makes the bot join your voice channel"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player join"])
    .examples_localized("en-US", ["/music-player join"])
)]
async fn music_player_join(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => Ok({
        match music_player.join(&core, guild_id, channel_id).await {
          Ok(()) => format!("Joined channel {}", channel_id.mention()),
          Err(err) => {
            error!("failed to join channel: {err}");
            "Failed to join channel".to_owned()
          }
        }
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "leave",
  name_localized("en-US", "leave"),
  description_localized("en-US", "Makes the bot leave your voice channel (does not clear the queue)"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player leave"])
    .examples_localized("en-US", ["/music-player leave"])
)]
async fn music_player_leave(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => Ok({
        match music_player.leave(&core, guild_id).await {
          Ok(()) => format!("Left channel {}", channel_id.mention()),
          Err(err) => {
            error!("failed to leave channel: {err}");
            "Failed to leave channel".to_owned()
          }
        }
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "pause",
  name_localized("en-US", "pause"),
  description_localized("en-US", "Pauses (or unpauses) the music player"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player pause <true|false>"])
    .examples_localized("en-US", ["/music-player pause false"])
)]
async fn music_player_pause(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "state")]
  #[description_localized("en-US", "Whether to pause (true) or unpause (false)")]
  state: bool
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        music_player.set_pause(guild_id, state).await;
        match state {
          true => "Paused the current track".to_owned(),
          false => "Unpaused the current track".to_owned()
        }
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "loop",
  name_localized("en-US", "loop"),
  description_localized("en-US", "Enables (or disables) looping on the music player queue"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player loop <true|false>"])
    .examples_localized("en-US", ["/music-player loop true"])
)]
async fn music_player_loop(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "state")]
  #[description_localized("en-US", "Whether to enable looping (true) or disable looping (false)")]
  state: bool
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        music_player.set_loop(guild_id, state).await;
        match state {
          true => "Enabled queue looping".to_owned(),
          false => "Disabled queue looping".to_owned()
        }
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "skip",
  name_localized("en-US", "skip"),
  description_localized("en-US", "Skips the current song in the music player queue"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player skip"])
    .examples_localized("en-US", ["/music-player skip"])
)]
async fn music_player_skip(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        ctx.defer().await.context("failed to defer response")?;
        music_player.skip(&core, guild_id).await;

        "Skipped currently playing track".to_owned()
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "stop",
  name_localized("en-US", "stop"),
  description_localized("en-US", "Stops the music player, clearing the queue"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player stop"])
    .examples_localized("en-US", ["/music-player stop"])
)]
async fn music_player_stop(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        ctx.defer().await.context("failed to defer response")?;
        music_player.stop(&core, guild_id).await;

        "Stopped the player and cleared the queue".to_owned()
      })
    }
  }).await
}

#[poise::command(
  slash_command,
  guild_only,
  category = "music-player",
  rename = "kill",
  name_localized("en-US", "kill"),
  description_localized("en-US", "Makes the bot leave your voice channel, stopping the music player and clearing the queue"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/music-player kill"])
    .examples_localized("en-US", ["/music-player kill"])
)]
async fn music_player_kill(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(ctx, {
    match ensure_in_same_channel(&core, guild_id, ctx.author().id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => Ok({
        ctx.defer().await.context("failed to defer response")?;
        match music_player.kill(&core, guild_id).await {
          Ok(()) => format!("Left channel {} and cleared the queue", channel_id.mention()),
          Err(err) => {
            error!("failed to leave channel: {err}");
            "Failed to leave channel".to_owned()
          }
        }
      })
    }
  }).await
}

async fn ensure_in_channel(
  core: &Core, guild_id: GuildId, user_id: UserId
) -> Result<(Arc<MusicPlayer>, ChannelId), String> {
  let music_player = get_music_player(core)?;
  if let Some(channel_id) = user_voice_channel(core, guild_id, user_id) {
    Ok((music_player, channel_id))
  } else {
    Err("You are not in a voice channel".to_owned())
  }
}

async fn ensure_in_same_channel(
  core: &Core, guild_id: GuildId, user_id: UserId
) -> Result<(Arc<MusicPlayer>, ChannelId), String> {
  let music_player = get_music_player(core)?;
  if let Some(channel_id) = user_voice_channel(core, guild_id, user_id) {
    if let Some(bot_channel_id) = music_player.current_channel(core, guild_id).await {
      if channel_id == bot_channel_id {
        Ok((music_player, channel_id))
      } else {
        Err("We are not in the same voice channel".to_owned())
      }
    } else {
      Err("I am not in a voice channel".to_owned())
    }
  } else {
    Err("You are not in a voice channel".to_owned())
  }
}

async fn send_response_result(ctx: MelodyContext<'_>, result: Result<String, String>) -> MelodyResult {
  let response = match result {
    Ok(response) => response,
    Err(response) => response
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

fn user_voice_channel(core: &Core, guild_id: GuildId, user_id: UserId) -> Option<ChannelId> {
  let channel_id = core.cache.guild(guild_id)?.voice_states.get(&user_id)?.channel_id?;
  Some(channel_id)
}

fn get_music_player(core: &Core) -> Result<Arc<MusicPlayer>, String> {
  core.state.music_player.clone().ok_or_else(|| "Music player is not enabled".to_owned())
}
