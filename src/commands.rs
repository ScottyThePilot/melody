mod connect_four;
mod general;

use crate::MelodyResult;
use crate::blueprint::*;
use crate::data::*;
use crate::utils::Contextualize;

use itertools::Itertools;
use serenity::cache::Cache;
use serenity::model::id::GuildId;
use serenity::model::application::command::Command;
use serenity::utils::Color;

use std::collections::HashSet;



pub const APPLICATION_COMMANDS: &[BlueprintCommand] = &[
  self::general::PING,
  self::general::HELP,
  self::general::AVATAR,
  self::general::BANNER,
  self::connect_four::CONNECT_FOUR
];

pub async fn bot_color(core: &Core) -> Color {
  core.operate_config(|config| config.accent_color).await
    .or_else(|| core.cache.current_user_field(|me| me.accent_colour))
    .unwrap_or(serenity::utils::colours::branding::BLURPLE)
}

pub async fn register_guild_commands(
  core: &Core, guild_id: GuildId, plugins: HashSet<String>
) -> MelodyResult {
  let guild_name = core.cache.guild_field(guild_id, |guild| guild.name.clone())
    .unwrap_or_else(|| "Unknown".to_owned());
  register_guild_commands_verbose(core, guild_id, &guild_name, exclusive_commands(), plugins).await
}

pub async fn register_guild_commands_verbose(
  core: &Core, guild_id: GuildId, guild_name: &str,
  exclusive_commands: impl IntoIterator<Item = &BlueprintCommand>,
  plugins: HashSet<String>
) -> MelodyResult {
  let guild_commands = exclusive_commands.into_iter().cloned()
    .filter(|&blueprint| blueprint.is_enabled(&plugins))
    .collect::<Vec<_>>();
  if guild_commands.is_empty() {
    info!("Clearing exclusive commands for guild {guild_name} ({guild_id})");
    guild_id.set_application_commands(&core, |builder| builder)
      .await.context("failed to clear guild commands")?;
  } else {
    let commands_text = guild_commands.iter().map(|blueprint| blueprint.name).join(", ");
    info!("Registering exclusive commands ({commands_text}) for guild {guild_name} ({guild_id})");
    guild_id.set_application_commands(&core, commands_builder(&guild_commands))
      .await.context("failed to register guild-only commands")?;
  };

  Ok(())
}

pub async fn register_commands(core: &Core, guilds: &[GuildId]) -> MelodyResult {
  let (exclusive_commands, default_commands) = partition_application_commands();
  info!("Found {} commands, {} exclusive commands", default_commands.len(), exclusive_commands.len());
  Command::set_global_application_commands(&core, commands_builder(&default_commands))
    .await.context("failed to register commands")?;
  for (guild_id, guild_name) in iter_guilds(&core, guilds) {
    let plugins = core.operate_persist(|persist| persist.get_guild_plugins(guild_id)).await;
    register_guild_commands_verbose(core, guild_id, &guild_name, &exclusive_commands, plugins).await?;
  };

  Ok(())
}

pub fn iter_guilds<'a>(cache: impl AsRef<Cache> + 'a, guilds: &'a [GuildId])
-> impl Iterator<Item = (GuildId, String)> + 'a {
  guilds.into_iter().map(move |&guild_id| {
    let guild_name = cache.as_ref().guild_field(guild_id, |guild| guild.name.clone());
    (guild_id, guild_name.unwrap_or_else(|| "Unknown".to_owned()))
  })
}

pub fn partition_application_commands() -> (Vec<BlueprintCommand>, Vec<BlueprintCommand>) {
  APPLICATION_COMMANDS.into_iter().copied().partition(|blueprint| blueprint.is_exclusive())
}

pub fn exclusive_commands() -> impl Iterator<Item = &'static BlueprintCommand> {
  APPLICATION_COMMANDS.into_iter().filter(|blueprint| blueprint.is_exclusive())
}
