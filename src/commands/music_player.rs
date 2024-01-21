use crate::{MelodyError, MelodyResult};
use crate::blueprint::*;
use crate::data::{Core, MusicPlayerKey};
use crate::feature::music_player::{MusicPlayer, QueueItem, AttachmentItem, YouTubeItem};
use crate::utils::youtube;

use serenity::model::mention::Mentionable;
use serenity::model::id::{ChannelId, GuildId, UserId};
use serenity::model::channel::Attachment;

use std::num::NonZeroUsize;
use std::sync::Arc;

pub const MUSIC_PLAYER: BlueprintCommand = blueprint_command! {
  name: "music-player",
  description: "Play audio from various sources in voice channels",
  info: [
    "Player functionality may be spotty or unreliable.",
    "If the player breaks or stops on a track indefinitely, command the bot to skip, leave the channel, and join the channel again.",
    "Issuing the stop command will clear the queue, the leave command will not."
  ],
  usage: [
    "/music-player play youtube <video-url>",
    "/music-player play attachment <attachment>",
    "/music-player join",
    "/music-player leave",
    "/music-player pause <true|false>",
    "/music-player loop <true|false>",
    "/music-player stop"
  ],
  examples: [
    "/music-player play youtube 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'",
    "/music-player join",
    "/music-player leave",
    "/music-player pause false",
    "/music-player loop true",
    "/music-player stop"
  ],
  plugin: "music-player",
  allow_in_dms: false,
  subcommands: [
    blueprint_subcommand! {
      name: "play",
      description: "Adds a track to the queue to be played in your voice channel",
      subcommands: [
        blueprint_subcommand! {
          name: "youtube",
          description: "Plays a YouTube video in your voice channel",
          arguments: [
            blueprint_argument!(String {
              name: "video-url",
              description: "The URL of the YouTube video to play",
              required: true,
              max_length: 256
            })
          ],
          function: music_player_play_youtube
        },
        blueprint_subcommand! {
          name: "youtube-playlist",
          description: "Plays a YouTube playlist in your voice channel",
          arguments: [
            blueprint_argument!(String {
              name: "playlist-url",
              description: "The URL of the YouTube playlist to play",
              required: true,
              max_length: 512
            }),
            blueprint_argument!(Boolean {
              name: "shuffle",
              description: "Whether to randomly shuffle the playlist or not",
              required: false
            })
          ],
          function: music_player_play_youtube_playlist
        },
        blueprint_subcommand! {
          name: "attachment",
          description: "Plays an audio file attachment in your voice channel",
          arguments: [
            blueprint_argument!(Attachment {
              name: "attachment",
              description: "The audio file attachment to play",
              required: true
            })
          ],
          function: music_player_play_attachment
        }
      ]
    },
    blueprint_subcommand! {
      name: "queue",
      description: "Commands relating to the music player queue",
      subcommands: [
        blueprint_subcommand! {
          name: "show",
          description: "Displays the full list of tracks in the queue",
          arguments: [
            blueprint_argument!(Integer {
              name: "page",
              description: "The page of the queue to be shown",
              required: false,
              min_value: 1
            })
          ],
          function: music_player_queue_show
        },
        blueprint_subcommand! {
          name: "clear",
          description: "Clears the queue, except for the currently playing track",
          arguments: [],
          function: music_player_queue_clear
        },
        blueprint_subcommand! {
          name: "remove",
          description: "Removes an item from the queue",
          arguments: [
            blueprint_argument!(Integer {
              name: "index",
              description: "The index of the item in the queue to remove",
              required: true,
              min_value: 1
            })
          ],
          function: music_player_queue_remove
        },
        blueprint_subcommand! {
          name: "shuffle",
          description: "Shuffles the queue, except for the currently playing track",
          arguments: [],
          function: music_player_queue_shuffle
        }
      ]
    },
    blueprint_subcommand! {
      name: "join",
      description: "Makes the bot join your voice channel",
      arguments: [],
      function: music_player_join
    },
    blueprint_subcommand! {
      name: "leave",
      description: "Makes the bot leave your voice channel",
      arguments: [],
      function: music_player_leave
    },
    blueprint_subcommand! {
      name: "pause",
      description: "Pauses (or unpauses) the music player",
      arguments: [
        blueprint_argument!(Boolean {
          name: "state",
          description: "Whether to pause (true) or unpause (false)",
          required: true
        })
      ],
      function: music_player_pause
    },
    blueprint_subcommand! {
      name: "loop",
      description: "Enables (or disables) looping on the music player queue",
      arguments: [
        blueprint_argument!(Boolean {
          name: "state",
          description: "Whether to enable looping (true) or disable looping (false)",
          required: true
        })
      ],
      function: music_player_loop
    },
    blueprint_subcommand! {
      name: "skip",
      description: "Skips the current song in the music player queue",
      arguments: [],
      function: music_player_skip
    },
    blueprint_subcommand! {
      name: "stop",
      description: "Stops the music player, clearing the queue",
      arguments: [],
      function: music_player_stop
    },
  ]
};

async fn music_player_play_youtube(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_id = args.interaction.user.id;
  let video_url = args.resolve_values::<&str>()?;

  send_response_result(&core, &args, {
    match ensure_in_channel(&core, guild_id, user_id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => match youtube::parse_video_url(video_url) {
        None => Err("Invalid YouTube link".to_owned()),
        Some(video_id) => Ok({
          let item = QueueItem::YouTube(YouTubeItem { id: video_id });
          let item_str = item.to_string();

          args.defer(&core).await?;

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

async fn music_player_play_youtube_playlist(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_id = args.interaction.user.id;
  let (playlist_url, shuffle) = args.resolve_values::<(&str, Option<bool>)>()?;
  let shuffle = shuffle.unwrap_or(false);

  send_response_result(&core, &args, {
    match ensure_in_channel(&core, guild_id, user_id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => match youtube::parse_playlist_url(playlist_url) {
        None => Err("Invalid YouTube link".to_owned()),
        Some(playlist_id) => Ok({
          args.defer(&core).await?;

          match youtube::get_playlist_info(music_player.ytdlp_path(), &playlist_id).await {
            Ok(playlist_info) => {
              let mut items = playlist_info.entries.into_iter()
                .map(|video_info| QueueItem::YouTube(YouTubeItem { id: video_info.id }))
                .collect::<Vec<QueueItem>>();
              let items_count = items.len();

              if shuffle {
                crate::utils::shuffle(&mut items);
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

async fn music_player_play_attachment(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let user_id = args.interaction.user.id;
  let attachment = args.resolve_values::<&Attachment>()?;
  let item = QueueItem::Attachment(AttachmentItem {
    id: attachment.id,
    filename: attachment.filename.clone(),
    filesize: attachment.size,
    url: attachment.url.clone()
  });
  let item_str = item.to_string();

  send_response_result(&core, &args, {
    match ensure_in_channel(&core, guild_id, user_id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => Ok({
        args.defer(&core).await?;

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

async fn music_player_queue_show(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  const PER_PAGE: usize = 10;
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let page = args.resolve_values::<Option<usize>>()?.unwrap_or(1) - 1;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        let queue_list = music_player.queue_list(guild_id).await;

        let page_start = (page * PER_PAGE) as usize;
        let entries = queue_list.into_iter()
          .enumerate().skip(page_start).take(PER_PAGE as usize)
          .map(|(i, queue_item)| match i == 0 {
            true => format!("Now Playing {queue_item}"),
            false => format!("`#{}` {queue_item}", i)
          })
          .collect::<Vec<String>>();
        if entries.is_empty() {
          "(Nothing is playing)".to_owned()
        } else {
          let mut response = entries.join("\n");
          response.push_str("(The queue will automatically loop)");
          response
        }
      })
    }
  }).await
}

async fn music_player_queue_clear(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        music_player.queue_clear(guild_id).await;

        "Cleared queue".to_owned()
      })
    }
  }).await
}

async fn music_player_queue_shuffle(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        music_player.queue_shuffle(guild_id).await;

        "Shuffled queue".to_owned()
      })
    }
  }).await
}

async fn music_player_queue_remove(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let index = args.resolve_values::<NonZeroUsize>()?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
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

async fn music_player_join(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(&core, &args, {
    match ensure_in_channel(&core, guild_id, args.interaction.user.id).await {
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

async fn music_player_leave(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
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

async fn music_player_pause(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let state = args.resolve_values::<bool>()?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
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

async fn music_player_loop(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let state = args.resolve_values::<bool>()?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
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

async fn music_player_skip(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
      Err(response) => Err(response),
      Ok((music_player, ..)) => Ok({
        args.defer(&core).await?;
        music_player.skip(&core, guild_id).await;

        "Skipped currently playing track".to_owned()
      })
    }
  }).await
}

async fn music_player_stop(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  send_response_result(&core, &args, {
    match ensure_in_same_channel(&core, guild_id, args.interaction.user.id).await {
      Err(response) => Err(response),
      Ok((music_player, channel_id)) => Ok({
        args.defer(&core).await?;
        match music_player.stop(&core, guild_id).await {
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
  let music_player = get_music_player(core).await?;
  if let Some(channel_id) = user_voice_channel(core, guild_id, user_id) {
    Ok((music_player, channel_id))
  } else {
    Err("You are not in a voice channel".to_owned())
  }
}

async fn ensure_in_same_channel(
  core: &Core, guild_id: GuildId, user_id: UserId
) -> Result<(Arc<MusicPlayer>, ChannelId), String> {
  let music_player = get_music_player(core).await?;
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

async fn send_response_result(
  core: &Core, args: &BlueprintCommandArgs,
  result: Result<String, String>
) -> MelodyResult {
  match result {
    Ok(response) => {
      BlueprintCommandResponse::new(response)
        .send(&core, &args).await?;
    },
    Err(response) => {
      BlueprintCommandResponse::new_ephemeral(response)
        .send(&core, &args).await?;
    }
  };

  Ok(())
}

fn user_voice_channel(core: &Core, guild_id: GuildId, user_id: UserId) -> Option<ChannelId> {
  let channel_id = core.cache.guild(guild_id)?.voice_states.get(&user_id)?.channel_id?;
  Some(channel_id)
}

async fn get_music_player(core: &Core) -> Result<Arc<MusicPlayer>, String> {
  core.get::<MusicPlayerKey>().await.ok_or_else(|| "Music player is not enabled".to_owned())
}
