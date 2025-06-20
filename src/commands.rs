mod connect_four;
mod feed;
mod general;
mod music_player;
mod role_playing;
mod roles;

use crate::prelude::*;
use crate::data::*;
use crate::utils::{Blockify, Contextualize};

use poise::{Context as PoiseContext, Command as PoiseCommand};
use poise::reply::CreateReply;
use serenity::cache::Cache;
use serenity::model::id::GuildId;
use serenity::model::application::Command as SerenityCommand;
use serenity::model::colour::Color;
use serenity::model::permissions::Permissions;
use serenity::builder::{CreateEmbed, CreateEmbedFooter};

use std::collections::{HashSet, HashMap};
use std::sync::Arc;



const COMMANDS: &[fn() -> MelodyCommand] = &[
  help,
  self::general::ping,
  self::general::echo,
  self::general::troll,
  self::general::avatar,
  self::general::banner,
  self::general::emoji_stats,
  self::general::console,
  self::role_playing::roll,
  self::feed::feeds,
  self::music_player::music_player,
  self::connect_four::connect_four,
  self::roles::role,
  self::roles::grant_roles,
  self::roles::join_roles
];

pub fn create_commands_list() -> Vec<MelodyCommand> {
  COMMANDS.into_iter().map(|f| f()).collect()
}

//self::feed::feeds
//self::music_player::music_player
//self::connect_four::connect_four
//self::role_playing::roll
//self::roles::role
//self::roles::grant_roles
//self::roles::join_roles

pub type MelodyCommand = PoiseCommand<Arc<State>, MelodyError>;
pub type MelodyContext<'a> = PoiseContext<'a, Arc<State>, MelodyError>;

#[derive(Debug, Clone, Default)]
pub struct CommandState {
  info_localizations: Option<HashMap<String, String>>,
  usage_localizations: HashMap<String, Vec<String>>,
  examples_localizations: HashMap<String, Vec<String>>
}

impl CommandState {
  pub fn new() -> Self {
    CommandState::default()
  }

  pub fn info_localized(mut self, locale: impl Into<String>, contents: impl Into<String>) -> Self {
    self.info_localizations.get_or_insert_default().insert(locale.into(), contents.into());
    self
  }

  pub fn info_localized_concat(mut self, locale: impl Into<String>, contents: impl IntoIterator<Item = impl std::fmt::Display>) -> Self {
    let contents = contents.into_iter().join(" ");
    self.info_localizations.get_or_insert_default().insert(locale.into(), contents);
    self
  }

  pub fn usage_localized(mut self, locale: impl Into<String>, contents: impl IntoIterator<Item = impl Into<String>>) -> Self {
    let contents = contents.into_iter().map(Into::into).collect::<Vec<String>>();
    self.usage_localizations.insert(locale.into(), contents);
    self
  }

  pub fn examples_localized(mut self, locale: impl Into<String>, contents: impl IntoIterator<Item = impl Into<String>>) -> Self {
    let contents = contents.into_iter().map(Into::into).collect::<Vec<String>>();
    self.examples_localizations.insert(locale.into(), contents);
    self
  }
}

pub async fn register_guild_commands(core: &Core, commands: &[MelodyCommand], guild_id: GuildId, plugins: HashSet<String>) -> MelodyResult {
  let guild_name = crate::utils::guild_name(&core, guild_id);
  let guild_commands = iter_exclusive_commands(commands)
    .filter(|command| command.category.as_ref().map_or(true, |c| plugins.contains(c)))
    .collect::<Vec<&MelodyCommand>>();
  if guild_commands.is_empty() {
    info!("Clearing exclusive commands for guild {guild_name} ({guild_id})");
    guild_id.set_commands(&core, Vec::new())
      .await.context("failed to clear guild commands")?;
  } else {
    let plugins_text = plugins.iter().join(", ");
    info!("Registering exclusive commands for plugins ({plugins_text}) for guild {guild_name} ({guild_id})");
    guild_id.set_commands(&core, create_application_commands(guild_commands))
      .await.context("failed to register guild-only commands")?;
  };

  Ok(())
}

pub async fn register_commands(core: &Core, commands: &[MelodyCommand], guilds: &[GuildId]) -> MelodyResult {
  let default_commands_count = iter_default_commands(commands).count();
  let exclusive_commands_count = iter_exclusive_commands(commands).count();
  info!("Found {default_commands_count} commands, {exclusive_commands_count} exclusive commands");
  SerenityCommand::set_global_commands(&core, create_application_commands(iter_default_commands(commands)))
    .await.context("failed to register commands")?;
  for &guild_id in guilds {
    let plugins = core.operate_persist(|persist| persist.get_guild_plugins(guild_id)).await;
    register_guild_commands(core, commands, guild_id, plugins).await?;
  };

  Ok(())
}

pub fn iter_guilds<'a>(cache: impl AsRef<Cache> + 'a, guilds: &'a [GuildId])
-> impl Iterator<Item = (GuildId, String)> + 'a {
  guilds.into_iter().map(move |&guild_id| {
    (guild_id, crate::utils::guild_name(cache.as_ref(), guild_id))
  })
}

fn create_application_commands<'a>(commands: impl IntoIterator<Item = &'a MelodyCommand>) -> Vec<serenity::builder::CreateCommand> {
  fn recursively_add_context_menu_commands(builder: &mut Vec<serenity::builder::CreateCommand>, command: &MelodyCommand) {
    if let Some(context_menu_command) = command.create_as_context_menu_command() {
      builder.push(context_menu_command);
    };

    for subcommand in &command.subcommands {
      recursively_add_context_menu_commands(builder, subcommand);
    };
  }

  let mut commands_builder = Vec::new();
  for command in commands {
    if let Some(slash_command) = command.create_as_slash_command() {
      commands_builder.push(slash_command);
    };

    recursively_add_context_menu_commands(&mut commands_builder, command);
  };

  commands_builder
}

fn iter_default_commands(commands: &[MelodyCommand]) -> impl Iterator<Item = &MelodyCommand> + Clone {
  commands.iter().filter(|command| command.category.is_none())
}

fn iter_exclusive_commands(commands: &[MelodyCommand]) -> impl Iterator<Item = &MelodyCommand> + Clone {
  commands.iter().filter(|command| command.category.is_some())
}



#[poise::command(
  slash_command,
  description_localized("en-US", "Gets command help"),
  custom_data = CommandState::new()
    .usage_localized("en-US", ["/help [command]"])
    .examples_localized("en-US", ["/help connect-four"])
)]
async fn help(
  ctx: MelodyContext<'_>,
  #[description_localized("en-US", "A specific command to get help for, otherwise returns the full command list")]
  argument: Option<String>
) -> MelodyResult {
  let core = Core::from(ctx);
  let color = get_bot_color(&Core::from(ctx)).await;
  let locale = ctx.locale().unwrap_or("en-US");

  let framework = core.get::<MelodyFrameworkKey>().await.lock_owned().await;
  let reply = if let Some(argument) = argument {
    if let Some(command) = find_command(&framework.options().commands, locale, &argument) {
      CreateReply::default().ephemeral(true)
        .embed(command_embed(command, locale, color))
    } else {
      CreateReply::default().ephemeral(true)
        .content("That command does not exist")
    }
  } else {
    #[allow(deprecated)]
    let permissions = ctx.author_member().await
      // "Use Guild::member_permissions_in instead"? What? That function doesn't exist.
      .and_then(|member| member.permissions(&ctx).ok())
      .unwrap_or(Permissions::all());
    CreateReply::default().ephemeral(true)
      .embed(command_list_embed(&framework.options().commands, locale, permissions, color))
  };

  ctx.send(reply).await.context("failed to send reply")?;
  Ok(())
}

pub async fn get_bot_color(core: &Core) -> Color {
  core.operate_config(|config| config.accent_color).await
    .or_else(|| core.cache.current_user().accent_colour)
    .unwrap_or(Color::BLURPLE)
}

pub fn find_command<'c>(commands: &'c [MelodyCommand], locale: &str, argument: &str) -> Option<&'c MelodyCommand> {
  argument.split_whitespace()
    .try_fold((commands, None), |(commands, _), command_name| {
      commands.iter()
        .find(|command| {
          command.name_localizations.get(locale)
            .unwrap_or(&command.name)
            .eq_ignore_ascii_case(command_name)
        })
        .map(|command| {
          (command.subcommands.as_slice(), Some(command))
        })
    })
    .and_then(|(_, command)| command)
}

pub fn command_embed(command: &MelodyCommand, locale: &str, color: Color) -> CreateEmbed {
  let command_state = command.custom_data.downcast_ref::<CommandState>()
    .expect("command custom data was not of type CommandState");

  let name = command.name_localizations.get(locale).unwrap_or(&command.name);
  let description = command.description_localizations.get(locale)
    .or_else(|| command.description.as_ref())
    .map_or("(none)", String::as_str);
  let info = command_state.info_localizations.as_ref()
    .and_then(|info| info.get(locale));
  let usage = command_state.usage_localizations.get(locale)
    .map(|usage| usage.iter().map(Blockify::new).join("\n"))
    .unwrap_or_else(|| "(none)".to_owned());
  let examples = command_state.examples_localizations.get(locale)
    .map(|examples| examples.iter().map(Blockify::new).join("\n"))
    .unwrap_or_else(|| "(none)".to_owned());

  let permissions_user = command.required_permissions | command.default_member_permissions;
  let permissions_bot = command.required_bot_permissions;

  let mut fields = Vec::new();

  if let Some(info) = info {
    fields.push(("Info", info.clone(), false));
  };

  fields.push(("Usage", usage, false));
  fields.push(("Examples", examples, false));

  if !permissions_user.is_empty() {
    fields.push(("Required User Permissions", permissions_user.to_string(), false));
  };

  if !permissions_bot.is_empty() {
    fields.push(("Required Bot Permissions", permissions_bot.to_string(), false));
  };

  fields.push(("Allowable Where?", {
    str::to_owned(match (command.dm_only, command.guild_only) {
      (true, true) => "Nowhere",
      (true, false) => "Only in DMs",
      (false, true) => "Only in Guilds",
      (false, false) => "Anywhere"
    })
  }, false));

  CreateEmbed::default()
    .title(crate::utils::to_words(name)).color(color)
    .description(description).fields(fields)
    .footer(CreateEmbedFooter::new(format!("Melody v{}", env!("CARGO_PKG_VERSION"))))
}

pub fn command_list_embed(commands: &[MelodyCommand], locale: &str, permissions: Permissions, color: Color) -> CreateEmbed {
  let mut commands = commands.iter().collect::<Vec<&MelodyCommand>>();

  commands.sort_by(|a, b| {
    let [a, b] = [a, b].map(|command| {
      command.name_localizations.get(locale).unwrap_or(&command.name)
    });

    String::cmp(a, b)
  });

  let body = commands.into_iter()
    .filter(|command| permissions.contains(command.required_permissions | command.default_member_permissions))
    .map(|command| {
      let name = command.name_localizations.get(locale).unwrap_or(&command.name);
      let description = command.description_localizations.get(locale)
        .or_else(|| command.description.as_ref());

      match description {
        Some(description) => format!("`/{name}`: *{description}*"),
        None => format!("`/{name}`")
      }
    })
    .join("\n");

  CreateEmbed::default()
    .title("Command Help").color(color)
    .description("Below is a list of commands, each with a short description of what they do")
    .field("Command List", body, false)
    .footer(CreateEmbedFooter::new(format!("Melody v{}", env!("CARGO_PKG_VERSION"))))
}
