mod connect_four;
mod general;

use crate::MelodyResult;
use crate::blueprint::*;
use crate::data::*;
use crate::utils::Contextualize;

use itertools::Itertools;
use serenity::client::Context;
use serenity::model::id::GuildId;
use serenity::model::application::command::Command;
use serenity::utils::Color;



pub const APPLICATION_COMMANDS: &[BlueprintCommand] = &[
  self::general::PING,
  self::general::HELP,
  self::general::AVATAR,
  self::general::BANNER,
  self::connect_four::CONNECT_FOUR
];

pub async fn bot_color(ctx: &Context) -> Color {
  data_operate_config(ctx, |config| config.accent_color).await
    .or_else(|| ctx.cache.current_user_field(|me| me.accent_colour))
    .unwrap_or(serenity::utils::colours::branding::BLURPLE)
}

pub async fn register_commands(ctx: &Context, guilds: &[GuildId]) -> MelodyResult {
  let (exclusive_commands, default_commands) = APPLICATION_COMMANDS.into_iter().copied()
    .partition::<Vec<_>, _>(BlueprintCommand::is_exclusive);
  info!("Found {} commands, {} exclusive commands", default_commands.len(), exclusive_commands.len());
  Command::set_global_application_commands(&ctx, commands_builder(&default_commands))
    .await.context("failed to register commands")?;
  for &guild_id in guilds {
    let guild_name = ctx.cache.guild_field(guild_id, |guild| guild.name.clone())
      .unwrap_or_else(|| "Unknown".to_owned());
    info!("Discovered guild: {guild_name} ({guild_id})");

    if exclusive_commands.is_empty() { continue };
    let plugins = Persist::get_guild_plugins(&data_get_persist(ctx).await, guild_id).await;

    let guild_commands = exclusive_commands.iter().cloned()
      .filter(|&blueprint| blueprint.is_enabled(&plugins))
      .collect::<Vec<_>>();
    if guild_commands.is_empty() {
      info!("Clearing exclusive commands for guild {guild_name} ({guild_id}");
      guild_id.set_application_commands(&ctx, |builder| builder)
        .await.context("failed to clear guild commands")?;
    } else {
      let commands_text = guild_commands.iter().map(|blueprint| blueprint.name).join(", ");
      info!("Registering exclusive commands ({commands_text}) for guild {guild_name} ({guild_id})");
      guild_id.set_application_commands(&ctx, commands_builder(&guild_commands))
        .await.context("failed to register guild-only commands")?;
    };
  };

  Ok(())
}
