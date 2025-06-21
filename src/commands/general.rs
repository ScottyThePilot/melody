use crate::prelude::*;
use crate::data::Core;
use crate::feature::dice_roll::Roll;
use crate::utils::Blockify;
use super::{MelodyContext, CommandMetaData};

use chrono::{Utc, Duration};
use log::Level;
use rand::Rng;
use poise::reply::CreateReply;
use serenity::model::guild::Member;
use serenity::model::timestamp::Timestamp;
use serenity::utils::{content_safe, ContentSafeOptions};



const FUNNY_CHANCE: f64 = 0.01;

#[poise::command(
  slash_command,
  name_localized("en-US", "ping"),
  description_localized("en-US", "Gets a basic response from the bot"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/ping"])
    .examples_localized("en-US", ["/ping"])
)]
pub async fn ping(ctx: MelodyContext<'_>) -> MelodyResult {
  let response = if rand::thread_rng().gen_bool(FUNNY_CHANCE) { "Pog" } else { "Pong" };
  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  name_localized("en-US", "echo"),
  description_localized("en-US", "Makes the bot repeat something"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/echo <text>"])
    .examples_localized("en-US", ["/echo 'hello world'"])
)]
pub async fn echo(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "text")]
  #[description_localized("en-US", "The text to be repeated back")]
  #[max_length = 1000]
  text: String
) -> MelodyResult {
  let response = content_safe(&ctx, &text, &ContentSafeOptions::default().clean_user(false), &[]);
  info!("Echoing message (original): {text:?}");
  info!("Echoing message (filtered): {response:?}");
  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  name_localized("en-US", "troll"),
  description_localized("en-US", "Conducts epic trollage"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/troll"])
    .examples_localized("en-US", ["/troll"])
)]
pub async fn troll(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "victim")]
  #[description_localized("en-US", "The user to be trolled, if so desired")]
  victim: Option<Member>
) -> MelodyResult {
  let mut perpetrator = ctx.author_member().await
    .ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?
    .into_owned();

  let time = Timestamp::from(Utc::now() + Duration::seconds(10));
  let response = match victim {
    Some(mut victim) => {
      if victim.user.id == ctx.cache().current_user().id {
        let time = Timestamp::from(Utc::now() + Duration::seconds(30));
        match perpetrator.disable_communication_until_datetime(&ctx, time).await.log_error() {
          Some(()) => "Impossible. Heresy. Unspeakable. Heresy. Heresy. Silence.".to_owned(),
          None => "Not happening, buddy.".to_owned()
        }
      } else if perpetrator.user.id == victim.user.id {
        match perpetrator.disable_communication_until_datetime(&ctx, time).await.log_error() {
          Some(()) => format!("{} has successfully trolled themselves.", perpetrator.user.id.mention()),
          None => "Sorry, I don't have permission to do that.".to_owned()
        }
      } else {
        if rand::thread_rng().gen_bool(FUNNY_CHANCE) {
          match victim.disable_communication_until_datetime(&ctx, time).await.log_error() {
            Some(()) => format!("{} has successfully trolled {}.", perpetrator.mention(), victim.mention()),
            None => "Sorry, even though you succeeded, I don't have permission to do that.".to_owned()
          }
        } else {
          match perpetrator.disable_communication_until_datetime(&ctx, time).await.log_error() {
            Some(()) => format!("{}'s attempt at trollage was a royal failure.", perpetrator.mention()),
            None => "Sorry, I don't have permission to do that.".to_owned()
          }
        }
      }
    },
    None => {
      match perpetrator.disable_communication_until_datetime(&ctx, time).await.log_error() {
        Some(()) => format!("{} has been trolled.", perpetrator.user.id.mention()),
        None => "Sorry, I cannot do that.".to_owned()
      }
    }
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  name_localized("en-US", "avatar"),
  description_localized("en-US", "Gets another user's avatar"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/avatar [user] [get-global-avatar]"])
    .examples_localized("en-US", ["/avatar @Nanachi", "/avatar @Nanachi false"])
)]
pub async fn avatar(
  ctx: MelodyContext<'_>,
  #[rename = "user"]
  #[name_localized("en-US", "user")]
  #[description_localized("en-US", "The user whose avatar should be retrieved, defaults to the caller if not set")]
  member: Option<Member>,
  #[rename = "get-global-avatar"]
  #[name_localized("en-US", "get-global-avatar")]
  #[description_localized("en-US", "Whether to get this user's global avatar instead of their server avatar")]
  get_global: Option<bool>
) -> MelodyResult {
  let member = member
    .map(std::borrow::Cow::Owned)
    .or(ctx.author_member().await)
    .ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let get_global = get_global.unwrap_or(false);

  let response = if get_global { member.user.face() } else { member.face() };
  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  name_localized("en-US", "banner"),
  description_localized("en-US", "Gets another user's banner"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/banner [user]"])
    .examples_localized("en-US", ["/banner @Nanachi"])
)]
pub async fn banner(
  ctx: MelodyContext<'_>,
  #[rename = "user"]
  #[name_localized("en-US", "user")]
  #[description_localized("en-US", "The user whose banner should be retrieved, defaults to the caller if not set")]
  member: Option<Member>
) -> MelodyResult {
  let member = member
    .map(std::borrow::Cow::Owned)
    .or(ctx.author_member().await)
    .ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let response = member.user.banner_url()
    .unwrap_or_else(|| "Failed to get that user's banner".to_owned());
  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "emoji-stats",
  name_localized("en-US", "emoji-stats"),
  description_localized("en-US", "Gets usage statistics of emojis for this server"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/emoji-stats [page]"])
    .examples_localized("en-US", ["/emoji-stats 3"])
)]
pub async fn emoji_stats(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "page")]
  #[description_localized("en-US", "The page of results to display (results are grouped 20 at a time)")]
  #[min = 1]
  #[max = 65536]
  page: Option<usize>
) -> MelodyResult {
  const PER_PAGE: usize = 20;

  let core = Core::from(ctx);

  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let page = page.unwrap_or(1) - 1;

  let emoji_statistics = core.operate_persist_guild(guild_id, |persist_guild| {
    core.cache.guild(guild_id).map(|guild| {
      persist_guild.emoji_stats.get_emoji_uses(|emoji_id| guild.emojis.get(&emoji_id))
    }).ok_or(MelodyError::command_cache_failure("guild"))
  }).await?;

  let page_start = page * PER_PAGE;
  let entries = emoji_statistics.into_iter()
    .enumerate().skip(page_start).take(PER_PAGE as usize)
    .map(|(i, (emoji, count))| format!("`#{}` {emoji} ({count} times)", i + 1))
    .collect::<Vec<String>>();

  let response = match entries.is_empty() {
    true => "(No results)".to_owned(),
    false => entries.join("\n")
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  owners_only, dm_only,
  name_localized("en-US", "console"),
  description_localized("en-US", "Execute an internal command"),
  custom_data = CommandMetaData::new()
    .info_localized("en-US", "This command is only usable by the bot owner.")
)]
pub async fn console(
  ctx: MelodyContext<'_>,
  #[rename = "internal-command-text"]
  #[name_localized("en-US", "internal-command-text")]
  #[description_localized("en-US", "The internal command to be executed")]
  #[max_length = 1000]
  internal_command_text: String,
) -> MelodyResult {
  ctx.defer_ephemeral().await.context("failed to defer response")?;

  info!("External console input: {internal_command_text:?}");
  let mut input_agent = crate::handler::InputAgent::new(Core::from(ctx));
  let result = input_agent.line(internal_command_text).await;
  if let Err(err) = result {
    input_agent.error(err.to_string());
  };

  if input_agent.output().is_empty() {
    input_agent.trace("(no output)");
  };

  let mut output = Vec::new();
  for (level, line) in input_agent.into_output() {
    let level_prefix = match level {
      Level::Error => "[ERROR]: ",
      Level::Warn => "[WARN]: ",
      Level::Info => "[INFO]: ",
      Level::Debug => "[DEBUG]: ",
      Level::Trace => "[TRACE]: ",
    };

    output.push(format!("{level_prefix}{line}"));
  };

  fn output_length(output: &[String]) -> usize {
    #[allow(unstable_name_collisions)]
    output.iter().map(String::len)
      .intersperse("\n".len())
      .sum::<usize>()
  }

  let mut truncated = false;
  while output_length(&mut output) + 6 + 32 > 2000 {
    output.pop();
    truncated = true;
  };

  let output_full = output.into_iter()
    .chain(truncated.then(|| "...".to_owned()))
    .join("\n");

  let response = format!("```\n{output_full}\n```");
  let response = content_safe(&ctx, &response, &ContentSafeOptions::default().clean_user(false), &[]);
  let reply = CreateReply::default().ephemeral(true).content(response);
  ctx.send(reply).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  description_localized("en-US", "Rolls a configurable dice"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/roll <dice notation>"])
    .examples_localized("en-US", [
      "/roll '3d20'",
      "/roll 'd20 + 2 advantage'",
      "/roll 'd20 - 2 disadvantage'",
      "/roll '6d6 max 2'",
      "/roll '6d6 min 2'",
      "/roll '2d8 + 3'",
      "/roll 'coin'"
    ])
)]
pub async fn roll(
  ctx: MelodyContext<'_>,
  #[description_localized("en-US", "The notation for the dice roll to be made")]
  #[max_length = 1000]
  notation: String
) -> MelodyResult {
  let response = match notation.parse::<Roll>() {
    Ok(roll) => {
      let roll_message = roll.execute().to_string();
      if roll_message.len() > 2000 {
        "The resulting message was too long to send...".to_owned()
      } else {
        roll_message
      }
    },
    Err(error) => {
      Blockify(format_args!("Error: {error}")).to_string()
    }
  };

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}
