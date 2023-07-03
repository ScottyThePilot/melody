use crate::{MelodyError, MelodyResult};
use crate::blueprint::*;
use crate::data::Core;
use crate::utils::Loggable;

use chrono::{Utc, Duration};
use rand::Rng;
use serenity::model::id::EmojiId;
use serenity::model::mention::Mentionable;
use serenity::model::permissions::Permissions;
use serenity::model::timestamp::Timestamp;
use serenity::model::user::User;
use serenity::utils::{content_safe, ContentSafeOptions};



pub(super) const PING: BlueprintCommand = blueprint_command! {
  name: "ping",
  description: "Gets a basic response from the bot",
  usage: ["/ping"],
  examples: ["/ping"],
  allow_in_dms: true,
  arguments: [],
  function: ping
};

#[command_attr::hook]
async fn ping(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let response = if rand::thread_rng().gen_bool(0.01) { "Pog" } else { "Pong" };
  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

pub(super) const HELP: BlueprintCommand = blueprint_command! {
  name: "help",
  description: "Gets command help",
  usage: ["/help [command]"],
  examples: ["/help", "/help connect-four"],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(String {
      name: "command",
      description: "A specific command to get help for, otherwise returns the command list",
      required: false,
      choices: [
        ("ping", "ping"),
        ("help", "help"),
        ("echo", "echo"),
        ("troll", "troll"),
        ("avatar", "avatar"),
        ("banner", "banner"),
        ("connect-four", "connect-four"),
        ("emoji_stats", "emoji_stats"),
        ("roll", "roll")
      ]
    })
  ],
  function: help
};

#[command_attr::hook]
async fn help(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let color = super::bot_color(&core).await;
  let response = match resolve_arguments::<Option<String>>(args.option_values)? {
    // User provided a command, return help for that command
    Some(command_name) => match find_command(super::APPLICATION_COMMANDS, &command_name) {
      Some(command) => BlueprintCommandResponse::with_ephemeral_embeds(vec![command_embed(command, color)]),
      None => BlueprintCommandResponse::new_ephemeral("That command does not exist")
    },
    // User provided no command, return command list
    None => {
      let permissions = args.interaction.member.as_ref()
        .and_then(|member| member.permissions)
        .unwrap_or(Permissions::all());
      BlueprintCommandResponse::with_ephemeral_embeds(vec![
        command_list_embed(super::APPLICATION_COMMANDS, permissions, color)
      ])
    }
  };

  response.send(&core, &args.interaction).await
}

pub(super) const ECHO: BlueprintCommand = blueprint_command! {
  name: "echo",
  description: "Makes the bot repeat something",
  usage: ["/echo <text>"],
  examples: ["/echo 'hello world'"],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(String {
      name: "text",
      description: "The text to be repeated back",
      required: true,
      max_length: 1000
    })
  ],
  function: echo
};

#[command_attr::hook]
async fn echo(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let text = resolve_arguments::<String>(args.option_values)?;
  let text_filtered = content_safe(&core.cache, &text, &ContentSafeOptions::default().clean_user(false), &[]);
  info!("Echoing message: {text:?}");
  info!("Echoing message (filtered): {text_filtered:?}");
  BlueprintCommandResponse::new(text_filtered)
    .send(&core, &args.interaction).await
}

pub(super) const TROLL: BlueprintCommand = blueprint_command! {
  name: "troll",
  description: "Conducts epic trollage",
  usage: ["/troll"],
  examples: ["/troll"],
  allow_in_dms: false,
  arguments: [
    blueprint_argument!(User {
      name: "victim",
      description: "The user to be trolled, if so desired",
      required: false
    })
  ],
  function: troll
};

#[command_attr::hook]
async fn troll(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let mut member = args.interaction.member.clone().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let target = resolve_arguments::<Option<User>>(args.option_values)?;
  let time = Timestamp::from(Utc::now() + Duration::seconds(10));
  let response = match target {
    Some(target) => {
      if target.id == core.cache.current_user_id() {
        // Shadows `time`, changing it to be 30 seconds instead of 10.
        let time = Timestamp::from(Utc::now() + Duration::seconds(30));
        match member.disable_communication_until_datetime(&core, time).await.log_some() {
          Some(()) => "Impossible. Heresy. Unspeakable. Heresy. Heresy. Silence.".to_owned(),
          None => "Not happening, buddy.".to_owned()
        }
      } else {
        if rand::thread_rng().gen_bool(0.01) {
          let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
          let mut target_member = core.cache.member(guild_id, target.id)
            .ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
          match target_member.disable_communication_until_datetime(&core, time).await.log_some() {
            Some(()) => format!("{} has successfully trolled {}.", member.user.id.mention(), target.id.mention()),
            None => "Sorry, even though you succeeded, I don't have permission to do that.".to_owned()
          }
        } else {
          match member.disable_communication_until_datetime(&core, time).await.log_some() {
            Some(()) => format!("{}'s attempt at trollage was a royal failure.", member.user.id.mention()),
            None => "Sorry, I cannot do that.".to_owned()
          }
        }
      }
    },
    None => {
      match member.disable_communication_until_datetime(&core, time).await.log_some() {
        Some(()) => format!("{} has been trolled.", member.user.id.mention()),
        None => "Sorry, I cannot do that.".to_owned()
      }
    }
  };

  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

pub const AVATAR: BlueprintCommand = blueprint_command! {
  name: "avatar",
  description: "Gets another user's avatar",
  usage: ["/avatar [user]"],
  examples: ["/avatar", "/avatar @Nanachi"],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(User {
      name: "user",
      description: "The user whose avatar should be retrieved, defaults to the caller if not set",
      required: false
    })
  ],
  function: avatar
};

#[command_attr::hook]
async fn avatar(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let user = resolve_arguments::<Option<User>>(args.option_values)?
    .unwrap_or_else(|| args.interaction.user.clone());
  let response = match user.avatar_url() {
    Some(avatar_url) => BlueprintCommandResponse::new_ephemeral(avatar_url),
    None => BlueprintCommandResponse::new_ephemeral("Failed to get that user's avatar")
  };

  response.send(&core, &args.interaction).await
}

pub const BANNER: BlueprintCommand = blueprint_command! {
  name: "banner",
  description: "Gets another user's banner",
  usage: ["/banner [user]"],
  examples: ["/banner", "/banner @Nanachi"],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(User {
      name: "user",
      description: "The user whose banner should be retrieved, defaults to the caller if not set",
      required: false
    })
  ],
  function: banner
};

#[command_attr::hook]
async fn banner(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let user = resolve_arguments::<Option<User>>(args.option_values)?
    .unwrap_or_else(|| args.interaction.user.clone());
  let response = match user.banner_url() {
    Some(banner_url) => BlueprintCommandResponse::new_ephemeral(banner_url),
    None => BlueprintCommandResponse::new_ephemeral("Failed to get that user's banner")
  };

  response.send(&core, &args.interaction).await
}

pub const EMOJI_STATS: BlueprintCommand = blueprint_command! {
  name: "emoji-stats",
  description: "Gets usage statistics of emojis for this server",
  usage: ["/emoji-stats [page]"],
  examples: ["/emoji-stats", "/emoji-stats 3"],
  allow_in_dms: false,
  arguments: [
    blueprint_argument!(Integer {
      name: "page",
      description: "The page of results to display (results are grouped 10 at a time)",
      min_value: 1
    })
  ],
  function: emoji_stats
};

#[command_attr::hook]
async fn emoji_stats(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  const PER_PAGE: u64 = 10;
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let page = resolve_arguments::<Option<u64>>(args.option_values)?.unwrap_or(1) - 1;

  let emoji_statistics = core.operate_persist_guild(guild_id, |persist_guild| {
    core.cache.guild_field(guild_id, |guild| {
      persist_guild.get_emoji_uses(|emoji_id| guild.emojis.get(&emoji_id))
    }).ok_or(MelodyError::COMMAND_NOT_IN_GUILD)
  }).await?;

  let page_start = (page * PER_PAGE) as usize;
  let entries = emoji_statistics.into_iter()
    .enumerate().skip(page_start).take(PER_PAGE as usize)
    .map(|(i, (emoji, animated, count))| {
      let emoji_mention = (emoji, animated).mention();
      format!("`#{}` {emoji_mention} ({count} times)", i + page_start + 1)
    })
    .collect::<Vec<String>>();
  let response = match entries.is_empty() {
    true => BlueprintCommandResponse::new("(No results)".to_owned()),
    false => BlueprintCommandResponse::new(entries.join("\n"))
  };

  response.send(&core, &args.interaction).await
}
