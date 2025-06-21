use super::{MelodyCommand};

use itertools::Itertools;
use poise::reply::CreateReply;
use serenity::Error as SerenityError;
use serenity::builder::{CreateCommand, CreateEmbed, CreateEmbedFooter};
use serenity::cache::Cache;
use serenity::http::Http;
use serenity::model::id::GuildId;
use serenity::model::application::Command as SerenityCommand;
use serenity::model::Permissions;
use serenity::model::colour::Color;

use std::fmt;
use std::collections::{HashMap, HashSet};



#[derive(Debug, Clone, Default)]
pub struct CommandMetaData {
  info_localizations: Option<HashMap<String, String>>,
  usage_localizations: HashMap<String, Vec<String>>,
  examples_localizations: HashMap<String, Vec<String>>
}

impl CommandMetaData {
  pub fn new() -> Self {
    CommandMetaData::default()
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

  pub const fn get_info_localizations(&self) -> &Option<HashMap<String, String>> {
    &self.info_localizations
  }

  pub const fn get_usage_localizations(&self) -> &HashMap<String, Vec<String>> {
    &self.usage_localizations
  }

  pub const fn get_examples_localizations(&self) -> &HashMap<String, Vec<String>> {
    &self.examples_localizations
  }
}

fn create_application_commands<'a, S: 'a, E: 'a>(commands: impl IntoIterator<Item = &'a MelodyCommand<S, E>>) -> Vec<CreateCommand> {
  fn recursively_add_context_menu_commands<S, E>(builder: &mut Vec<CreateCommand>, command: &MelodyCommand<S, E>) {
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

pub async fn register_guild_commands<S, E>(
  cache_http: &(impl AsRef<Http> + AsRef<Cache>),
  commands: &[MelodyCommand<S, E>],
  guild_id: GuildId,
  categories: HashSet<String>
) -> Result<(), SerenityError> {
  let guild_name = guild_name(cache_http, guild_id);
  let guild_commands = iter_exclusive_commands(commands)
    .filter(|command| command.category.as_deref().map_or(true, |category| categories.contains(category)))
    .collect::<Vec<&MelodyCommand<S, E>>>();
  if guild_commands.is_empty() {
    info!("Clearing exclusive commands for guild {guild_name} ({guild_id})");
    guild_id.set_commands(cache_http, Vec::new()).await?;
  } else {
    let categories_text = categories.iter().join(", ");
    info!("Registering exclusive commands for categories ({categories_text}) for guild {guild_name} ({guild_id})");
    guild_id.set_commands(cache_http, create_application_commands(guild_commands)).await?;
  };

  Ok(())
}

pub async fn register_commands<S, E>(
  cache_http: &(impl AsRef<Http> + AsRef<Cache>),
  commands: &[MelodyCommand<S, E>],
  guilds: impl IntoIterator<Item = (GuildId, HashSet<String>)>,
) -> Result<(), SerenityError> {
  let common_commands_count = iter_common_commands(commands).count();
  let exclusive_commands_count = iter_exclusive_commands(commands).count();
  info!("Found {common_commands_count} commands, {exclusive_commands_count} exclusive commands");

  let commands_list = create_application_commands(iter_common_commands(commands));
  SerenityCommand::set_global_commands(cache_http, commands_list).await?;
  for (guild_id, categories) in guilds {
    register_guild_commands(cache_http, commands, guild_id, categories).await?;
  };

  Ok(())
}

pub fn iter_common_commands<S, E>(commands: &[MelodyCommand<S, E>])
-> impl Iterator<Item = &MelodyCommand<S, E>> + DoubleEndedIterator + Clone {
  commands.iter().filter(|command| command.category.is_none())
}

pub fn iter_exclusive_commands<S, E>(commands: &[MelodyCommand<S, E>])
-> impl Iterator<Item = &MelodyCommand<S, E>> + DoubleEndedIterator + Clone {
  commands.iter().filter(|command| command.category.is_some())
}

#[derive(Debug, Clone, Copy)]
pub struct HelpLocalization<'t> {
  pub none: &'t str,
  pub fallback_message: &'t str,
  pub title_command_list: &'t str,
  pub heading_command_list: &'t str,
  pub description_command_list: &'t str,
  pub heading_info: &'t str,
  pub heading_usage: &'t str,
  pub heading_examples: &'t str,
  pub heading_required_user_permissions: &'t str,
  pub heading_required_bot_permissions: &'t str,
  pub heading_allowable_where: &'t str,
  pub content_allowable_where_nowhere: &'t str,
  pub content_allowable_where_dm_only: &'t str,
  pub content_allowable_where_guild_only: &'t str,
  pub content_allowable_where_anywhere: &'t str,
  pub footer: Option<&'t str>
}

impl Default for HelpLocalization<'static> {
  fn default() -> Self {
    HelpLocalization {
      none: "(none)",
      fallback_message: "That command does not exist",
      title_command_list: "Command Help",
      heading_command_list: "Command List",
      description_command_list: "Below is a list of commands, each with a short description of what they do",
      heading_info: "Info",
      heading_usage: "Usage",
      heading_examples: "Examples",
      heading_required_user_permissions: "Required User Permissions",
      heading_required_bot_permissions: "Required Bot Permissions",
      heading_allowable_where: "Allowable Where?",
      content_allowable_where_nowhere: "Nowhere",
      content_allowable_where_dm_only: "Only in DMs",
      content_allowable_where_guild_only: "Only in Guilds",
      content_allowable_where_anywhere: "Anywhere",
      footer: None
    }
  }
}

pub fn find_command<'a, S, E>(commands: &'a [MelodyCommand<S, E>], locale: &str, argument: &str) -> Option<&'a MelodyCommand<S, E>> {
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

pub fn build_help_reply<S, E>(
  argument: Option<&str>,
  commands: &[MelodyCommand<S, E>],
  categories: &HashSet<String>,
  permissions: Permissions,
  locale: &str,
  text: HelpLocalization,
  embed_color: Color
) -> Option<CreateReply> {
  let reply_builder = CreateReply::default().ephemeral(true);

  let embed_builder = CreateEmbed::default().color(embed_color);
  let embed_builder = if let Some(footer_text) = text.footer {
    embed_builder.footer(CreateEmbedFooter::new(footer_text))
  } else {
    embed_builder
  };

  if let Some(argument) = argument {
    let command = find_command(commands, locale, argument)
      .filter(|command| permissions.contains(command.required_permissions | command.default_member_permissions))
      .filter(|command| command.category.as_deref().map_or(true, |category| categories.contains(category)));
    if let Some(command) = command {
      build_command_help_embed(command, locale, text, embed_builder)
        .map(|embed_builder| reply_builder.embed(embed_builder))
    } else {
      Some(reply_builder.content(text.fallback_message))
    }
  } else {
    build_command_list_help_embed(commands, categories, permissions, locale, text, embed_builder)
      .map(|embed_builder| reply_builder.embed(embed_builder))
  }
}

pub fn build_command_list_help_embed<S, E>(
  commands: &[MelodyCommand<S, E>],
  categories: &HashSet<String>,
  permissions: Permissions,
  locale: &str,
  text: HelpLocalization,
  embed_builder: CreateEmbed
) -> Option<CreateEmbed> {
  let mut commands = commands.iter()
    .filter(|command| permissions.contains(command.required_permissions | command.default_member_permissions))
    .filter(|command| command.category.as_deref().map_or(true, |category| categories.contains(category)))
    .map(|command| (command.name_localizations.get(locale).unwrap_or(&command.name), command))
    .collect::<Vec<(&String, &MelodyCommand<S, E>)>>();
  commands.sort_by(|a, b| String::cmp(a.0, b.0));

  let body = commands.into_iter()
    .map(|(name, command)| {
      let name = name.as_str();
      let description = command.description_localizations.get(locale)
        .or_else(|| command.description.as_ref())
        .map(String::as_str);
      Commandify { name, description }
    })
    .join("\n");

  let embed_builder = embed_builder
    .title(text.title_command_list)
    .description(text.description_command_list)
    .field(text.heading_command_list, body, false);
  Some(embed_builder)
}

pub fn build_command_help_embed<S, E>(
  command: &MelodyCommand<S, E>,
  locale: &str,
  text: HelpLocalization,
  embed_builder: CreateEmbed
) -> Option<CreateEmbed> {
  let command_state = command.custom_data.downcast_ref::<CommandMetaData>()?;

  let name = command.name_localizations.get(locale).unwrap_or(&command.name);
  let description = command.description_localizations.get(locale)
    .or_else(|| command.description.as_ref())
    .map_or(text.none, String::as_str);
  let info = command_state.info_localizations.as_ref()
    .and_then(|info| info.get(locale));
  let usage = command_state.usage_localizations.get(locale)
    .map(|usage| usage.iter().map(Blockify).join("\n"))
    .unwrap_or_else(|| text.none.to_owned());
  let examples = command_state.examples_localizations.get(locale)
    .map(|examples| examples.iter().map(Blockify).join("\n"))
    .unwrap_or_else(|| text.none.to_owned());

  let permissions_user = command.required_permissions | command.default_member_permissions;
  let permissions_bot = command.required_bot_permissions;

  let mut fields = Vec::new();

  if let Some(info) = info {
    fields.push((text.heading_info, info.clone(), false));
  };

  fields.push((text.heading_usage, usage, false));
  fields.push((text.heading_examples, examples, false));

  if !permissions_user.is_empty() {
    fields.push((text.heading_required_user_permissions, permissions_user.to_string(), false));
  };

  if !permissions_bot.is_empty() {
    fields.push((text.heading_required_bot_permissions, permissions_bot.to_string(), false));
  };

  let content_allowable_where = match (command.dm_only, command.guild_only) {
    (true, true) => text.content_allowable_where_nowhere,
    (true, false) => text.content_allowable_where_dm_only,
    (false, true) => text.content_allowable_where_guild_only,
    (false, false) => text.content_allowable_where_anywhere
  };

  fields.push((text.heading_allowable_where, content_allowable_where.to_owned(), false));

  let embed_builder = embed_builder
    .title(to_words(name))
    .description(description)
    .fields(fields);
  Some(embed_builder)
}



#[derive(Debug, Clone, Copy)]
struct Commandify<'t> {
  name: &'t str,
  description: Option<&'t str>
}

impl<'t> fmt::Display for Commandify<'t> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let name = self.name;
    match self.description {
      Some(description) => write!(f, "/`{name}`: *{description}*"),
      None => write!(f, "/`{name}`")
    }
  }
}

#[derive(Debug, Clone, Copy)]
struct Blockify<S>(S);

impl<S: fmt::Display> fmt::Display for Blockify<S> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "`{}`", self.0)
  }
}

fn guild_name(cache: impl AsRef<Cache>, guild_id: GuildId) -> String {
  cache.as_ref().guild(guild_id)
    .map_or_else(|| "Unknown".to_owned(), |guild| guild.name.clone())
}

fn capitalize(s: impl AsRef<str>) -> String {
  let mut chars = s.as_ref().chars();
  chars.next().map_or_else(String::new, |first| {
    first.to_uppercase()
      .chain(chars.map(|c| c.to_ascii_lowercase()))
      .collect()
  })
}

fn to_words(s: impl AsRef<str>) -> String {
  s.as_ref().split(&['-', '_']).map(capitalize).join(" ")
}
