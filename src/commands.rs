mod connect_four;
mod feed;
mod general;
mod music_player;
mod roles;

use crate::prelude::*;
use crate::data::*;
use crate::handler::{MelodyCommand, MelodyContext};

pub use melody_framework::commands::{CommandMetaData, HelpLocalization, build_help_reply};
use serenity::model::colour::Color;
use serenity::model::permissions::Permissions;



const COMMANDS: &[fn() -> MelodyCommand] = &[
  help,
  self::general::ping,
  self::general::echo,
  self::general::troll,
  self::general::avatar,
  self::general::emoji_stats,
  self::general::ban_id,
  self::general::console,
  self::general::roll,
  self::feed::feeds,
  self::music_player::music_player,
  self::connect_four::connect_four,
  self::roles::role,
  self::roles::grant_roles,
  self::roles::join_roles
];

pub fn create_commands_list(state: &State) -> Vec<MelodyCommand> {
  melody_framework::commands::create_commands_list(COMMANDS, state)
}

#[poise::command(
  slash_command,
  description_localized("en-US", "Gets command help"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/help [command]"])
    .examples_localized("en-US", ["/help connect-four"])
)]
async fn help(
  ctx: MelodyContext<'_>,
  #[description_localized("en-US", "A specific command to get help for, otherwise returns the full command list")]
  argument: Option<String>
) -> MelodyResult {
  let core = Core::from(ctx);
  let embed_color = get_bot_color(&Core::from(ctx)).await;
  let locale = ctx.locale().unwrap_or("en-US");

  #[allow(deprecated)]
  let permissions = ctx.author_member().await
    // "Use Guild::member_permissions_in instead"? What? That function doesn't exist.
    .and_then(|member| member.permissions(&ctx).ok())
    .unwrap_or(Permissions::all());

  let commands = core.get::<MelodyFrameworkKey>().await.read_commands_owned().await;

  let categories = if let Some(guild_id) = ctx.guild_id() {
    core.operate_persist(async |persist| persist.get_guild_plugins(guild_id)).await
  } else {
    HashSet::new()
  };

  let footer_text = format!(
    "Melody v{version} - Deployed {deployed} - Rev {rev}",
    version = env!("CARGO_PKG_VERSION"),
    deployed = crate::BUILD_DATE,
    rev = crate::BUILD_GIT_HASH
  );

  let help_localization = HelpLocalization {
    footer: Some(footer_text.as_str()),
    ..HelpLocalization::default()
  };

  let reply = build_help_reply(
    argument.as_deref(), &commands, &categories, permissions,
    locale, help_localization, embed_color
  ).expect("unable to create help message");

  ctx.send(reply).await.context("failed to send reply")?;
  Ok(())
}

pub async fn get_bot_color(core: &Core) -> Color {
  core.operate_config(async |config| config.accent_color).await
    .or_else(|| core.cache.current_user().accent_colour)
    .unwrap_or(Color::BLURPLE)
}
