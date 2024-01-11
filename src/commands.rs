mod connect_four;
mod feed;
mod general;
mod role_playing;
mod roles;

use crate::MelodyResult;
use crate::blueprint::*;
use crate::data::*;
use crate::utils::Contextualize;

use itertools::Itertools;
use serenity::cache::Cache;
use serenity::model::id::GuildId;
use serenity::model::application::Command;
use serenity::model::colour::Color;

use std::collections::HashSet;



pub const COMMANDS: &[BlueprintCommand] = &[
  self::feed::FEEDS,
  self::general::PING,
  self::general::HELP,
  self::general::ECHO,
  self::general::TROLL,
  self::general::AVATAR,
  self::general::BANNER,
  self::general::EMOJI_STATS,
  self::connect_four::CONNECT_FOUR,
  self::role_playing::ROLL,
  self::roles::ROLE,
  self::roles::GRANT_ROLES,
  self::roles::JOIN_ROLES
];



pub async fn get_bot_color(core: &Core) -> Color {
  core.operate_config(|config| config.accent_color).await
    .or_else(|| core.cache.current_user().accent_colour)
    .unwrap_or(Color::BLURPLE)
}

pub async fn register_guild_commands(
  core: &Core, guild_id: GuildId, plugins: HashSet<String>
) -> MelodyResult {
  let guild_name = crate::utils::guild_name(&core, guild_id);
  let guild_commands = iter_exclusive_commands(COMMANDS)
    .filter(|blueprint| blueprint.is_enabled(&plugins))
    .collect::<Vec<BlueprintCommand>>();
  if guild_commands.is_empty() {
    info!("Clearing exclusive commands for guild {guild_name} ({guild_id})");
    guild_id.set_commands(&core, Vec::new())
      .await.context("failed to clear guild commands")?;
  } else {
    let commands_text = guild_commands.iter().map(|blueprint| blueprint.name).join(", ");
    info!("Registering exclusive commands ({commands_text}) for guild {guild_name} ({guild_id})");
    guild_id.set_commands(&core, build_commands(guild_commands))
      .await.context("failed to register guild-only commands")?;
  };

  Ok(())
}

pub async fn register_commands(core: &Core, guilds: &[GuildId]) -> MelodyResult {
  let default_commands_count = iter_default_commands(COMMANDS).count();
  let exclusive_commands_count = iter_exclusive_commands(COMMANDS).count();
  info!("Found {default_commands_count} commands, {exclusive_commands_count} exclusive commands");
  Command::set_global_commands(&core, build_commands(iter_default_commands(COMMANDS)))
    .await.context("failed to register commands")?;
  for &guild_id in guilds {
    let plugins = core.operate_persist(|persist| persist.get_guild_plugins(guild_id)).await;
    register_guild_commands(core, guild_id, plugins).await?;
  };

  Ok(())
}

pub fn iter_guilds<'a>(cache: impl AsRef<Cache> + 'a, guilds: &'a [GuildId])
-> impl Iterator<Item = (GuildId, String)> + 'a {
  guilds.into_iter().map(move |&guild_id| {
    (guild_id, crate::utils::guild_name(cache.as_ref(), guild_id))
  })
}
