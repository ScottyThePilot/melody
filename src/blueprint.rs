use crate::prelude::*;
use crate::data::Core;
use crate::utils::{Blockify, Contextualize};

use itertools::Itertools;
use melody_flag::Flag;
use serenity::builder::{
  CreateCommand,
  CreateCommandOption,
  CreateEmbed,
  CreateEmbedFooter,
  CreateInteractionResponse,
  CreateInteractionResponseMessage,
  EditInteractionResponse
};
use serenity::http::CacheHttp;
use serenity::model::application::{
  CommandData,
  CommandDataOption,
  CommandDataOptionValue,
  CommandDataResolved,
  CommandInteraction,
  CommandOptionType,
  CommandType,
  InteractionContext
};
use serenity::model::user::User;
use serenity::model::channel::{Attachment, ChannelType, PartialChannel};
use serenity::model::guild::{PartialMember, Role};
use serenity::model::colour::Color;
use serenity::model::id::{UserId, ChannelId, RoleId, AttachmentId};
use serenity::model::permissions::Permissions;
pub use serenity::futures::future::BoxFuture;

use std::collections::HashSet;
use std::fmt;
use std::num::{NonZeroU64, NonZeroU32, NonZeroUsize};
use std::str::FromStr;



pub fn find_command(commands: &'static [BlueprintCommand], name: &str) -> Option<&'static BlueprintCommand> {
  commands.into_iter().find(|command| command.name == name)
}

pub fn find_subcommand(subcommands: &'static [BlueprintSubcommand], name: &str) -> Option<&'static BlueprintSubcommand> {
  subcommands.into_iter().find(|subcommand| subcommand.name == name)
}

pub fn iter_default_commands(commands: &'static [BlueprintCommand]) -> impl Iterator<Item = BlueprintCommand> + Clone {
  commands.into_iter().copied().filter(|blueprint| !blueprint.is_exclusive())
}

pub fn iter_exclusive_commands(commands: &'static [BlueprintCommand]) -> impl Iterator<Item = BlueprintCommand> + Clone {
  commands.into_iter().copied().filter(|blueprint| blueprint.is_exclusive())
}

pub fn command_embed(command: &'static BlueprintCommand, color: Color) -> CreateEmbed {
  CreateEmbed::default()
    .title(crate::utils::kebab_case_to_words(command.name))
    .description(command.description)
    .color(color)
    .field("Info", command.stringify_info(), false)
    .field("Usage", command.stringify_usage(), false)
    .field("Examples", command.stringify_examples(), false)
    .field("Required Permissions", command.stringify_permissions(), false)
    .field("Allowable Where?", command.context.to_string(), false)
    .footer(CreateEmbedFooter::new(format!("Melody v{}", env!("CARGO_PKG_VERSION"))))
}

pub fn command_list_embed(commands: &'static [BlueprintCommand], permissions: Permissions, color: Color) -> CreateEmbed {
  let mut commands = commands.to_owned();
  commands.sort_by_key(|command| command.name);
  let body = commands.into_iter()
    .filter(|command| permissions.contains(command.default_permissions.unwrap_or(Permissions::empty())))
    .map(|command| format!("`/{}`: *{}*", command.name, command.description))
    .join("\n");

  CreateEmbed::default()
    .title("Command Help")
    .description("Below is a list of commands, each with a short description of what they do.")
    .color(color)
    .field("Command List", body, false)
    .footer(CreateEmbedFooter::new(format!("Melody v{}", env!("CARGO_PKG_VERSION"))))
}

pub fn build_commands(commands: impl IntoIterator<Item = BlueprintCommand>) -> Vec<CreateCommand> {
  commands.into_iter().map(BlueprintCommand::into_command_builder).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum BlueprintCommandContext {
  #[default]
  Anywhere,
  OnlyInGuild,
  OnlyInDm
}

impl BlueprintCommandContext {
  const fn contexts(self) -> &'static [InteractionContext] {
    match self {
      Self::Anywhere => &[
        InteractionContext::Guild,
        InteractionContext::BotDm,
        InteractionContext::PrivateChannel
      ],
      Self::OnlyInGuild => &[
        InteractionContext::Guild
      ],
      Self::OnlyInDm => &[
        InteractionContext::BotDm,
        InteractionContext::PrivateChannel
      ]
    }
  }
}

impl fmt::Display for BlueprintCommandContext {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Self::Anywhere => "Anywhere",
      Self::OnlyInGuild => "Only in Guilds",
      Self::OnlyInDm => "Only in DMs"
    })
  }
}

#[derive(Debug, Clone, Copy)]
pub struct BlueprintCommand {
  pub name: &'static str,
  pub description: &'static str,
  pub info: Option<&'static [&'static str]>,
  pub usage: Option<&'static [&'static str]>,
  pub examples: Option<&'static [&'static str]>,
  pub plugin: Option<&'static str>,
  pub command_type: CommandType,
  pub context: BlueprintCommandContext,
  pub default_permissions: Option<Permissions>,
  pub root: BlueprintRoot
}

impl BlueprintCommand {
  pub fn into_command_builder(self) -> CreateCommand {
    let builder = CreateCommand::new(self.name)
      .description(self.description)
      .kind(self.command_type)
      .contexts(self.context.contexts().to_owned())
      .set_options(self.root.into_options_builders().1);
    if let Some(permissions) = self.default_permissions {
      builder.default_member_permissions(permissions)
    } else {
      builder
    }
  }

  pub fn is_enabled(&self, plugins: &HashSet<String>) -> bool {
    match self.plugin {
      Some(plugin) => plugins.contains(plugin),
      None => true
    }
  }

  pub fn is_exclusive(&self) -> bool {
    self.plugin.is_some()
  }

  fn stringify_info(self) -> String {
    self.info.map_or_else(|| "(none)".to_owned(), |info| info.into_iter().join(" "))
  }

  fn stringify_usage(self) -> String {
    self.usage.map_or_else(|| "(none)".to_owned(), |usage| {
      usage.into_iter().map(Blockify::new).join("\n")
    })
  }

  fn stringify_examples(self) -> String {
    self.examples.map_or_else(|| "(none)".to_owned(), |example| {
      example.into_iter().map(Blockify::new).join("\n")
    })
  }

  fn stringify_permissions(self) -> String {
    match self.default_permissions {
      None => "Everyone".to_owned(),
      Some(p) if p.is_empty() => "Everyone".to_owned(),
      Some(p) => p.to_string()
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct BlueprintSubcommand {
  pub name: &'static str,
  pub description: &'static str,
  pub root: BlueprintRoot
}

impl BlueprintSubcommand {
  fn into_option_builder(self) -> CreateCommandOption {
    let (kind, options_builders) = self.root.into_options_builders();
    let mut builder = CreateCommandOption::new(kind, self.name, self.description);
    for option_builder in options_builders {
      builder = builder.add_sub_option(option_builder);
    };

    builder
  }
}

#[derive(Debug, Clone, Copy)]
pub enum BlueprintRoot {
  Command {
    function: crate::utils::NoDebug<CommandFn>,
    options: &'static [BlueprintOption]
  },
  CommandContainer {
    subcommands: &'static [BlueprintSubcommand]
  }
}

impl BlueprintRoot {
  fn into_options_builders(self) -> (CommandOptionType, Vec<CreateCommandOption>) {
    match self {
      BlueprintRoot::Command { options, .. } => {
        let builders = options.iter().copied()
          .map(BlueprintOption::into_option_builder)
          .collect();
        (CommandOptionType::SubCommand, builders)
      },
      BlueprintRoot::CommandContainer { subcommands, .. } => {
        let builders = subcommands.iter().copied()
          .map(BlueprintSubcommand::into_option_builder)
          .collect();
        (CommandOptionType::SubCommandGroup, builders)
      }
    }
  }

  fn get(self, subcommands_names: &[String]) -> Option<(&'static [BlueprintOption], CommandFn)> {
    match self {
      // this command root terminates at a command with regular options,
      // return those regular options if `subcommands_names` agrees that the command terminates here
      BlueprintRoot::Command { options, function } => subcommands_names.is_empty().then(|| (options, *function)),
      // this command root contains other subcommands_names, split the first element off of `subcommands_names`
      BlueprintRoot::CommandContainer { subcommands } => {
        let (subcommand, remaining_subcommands) = subcommands_names.split_first()?;
        // if the first element of `subcommands_names` can be split off,
        // find the corresponding subcommand blueprint and recursively call the validator on it
        find_subcommand(subcommands, subcommand).and_then(|blueprint_subcommand| {
          blueprint_subcommand.root.get(remaining_subcommands)
        })
      }
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct BlueprintOption {
  pub name: &'static str,
  pub description: &'static str,
  pub required: bool,
  pub variant: BlueprintOptionVariant
}

impl BlueprintOption {
  fn into_option_builder(self) -> CreateCommandOption {
    match self.variant {
      BlueprintOptionVariant::String { min_length, max_length, choices } => {
        let mut builder = CreateCommandOption::new(CommandOptionType::String, self.name, self.description).required(self.required);
        if let Some(min_length) = min_length { builder = builder.min_length(min_length) };
        if let Some(max_length) = max_length { builder = builder.max_length(max_length) };
        for &(name, value) in choices {
          builder = builder.add_string_choice(name, value);
        };

        builder
      },
      BlueprintOptionVariant::Integer { min_value, max_value, choices } => {
        // TODO: update this to use `min_int_value` and `max_int_value` when they are fixed
        let mut builder = CreateCommandOption::new(CommandOptionType::Integer, self.name, self.description).required(self.required);
        if let Some(min_value) = min_value { builder = builder.min_number_value(min_value as f64) };
        if let Some(max_value) = max_value { builder = builder.max_number_value(max_value as f64) };
        for &(name, value) in choices {
          builder = builder.add_number_choice(name, value as f64);
        };

        builder
      },
      BlueprintOptionVariant::Number { min_value, max_value, choices } => {
        let mut builder = CreateCommandOption::new(CommandOptionType::Number, self.name, self.description).required(self.required);
        if let Some(min_value) = min_value { builder = builder.min_number_value(min_value) };
        if let Some(max_value) = max_value { builder = builder.max_number_value(max_value) };
        for &(name, value) in choices {
          builder = builder.add_number_choice(name, value);
        };

        builder
      },
      BlueprintOptionVariant::Boolean => {
        CreateCommandOption::new(CommandOptionType::Boolean, self.name, self.description).required(self.required)
      },
      BlueprintOptionVariant::User => {
        CreateCommandOption::new(CommandOptionType::User, self.name, self.description).required(self.required)
      },
      BlueprintOptionVariant::Role => {
        CreateCommandOption::new(CommandOptionType::Role, self.name, self.description).required(self.required)
      },
      BlueprintOptionVariant::Mentionable => {
        CreateCommandOption::new(CommandOptionType::Mentionable, self.name, self.description).required(self.required)
      },
      BlueprintOptionVariant::Channel { channel_types } => {
        let builder = CreateCommandOption::new(CommandOptionType::Mentionable, self.name, self.description).required(self.required);
        if channel_types.is_empty() { builder } else { builder.channel_types(channel_types.to_owned()) }
      },
      BlueprintOptionVariant::Attachment => {
        CreateCommandOption::new(CommandOptionType::Attachment, self.name, self.description).required(self.required)
      }
    }
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum BlueprintOptionVariant {
  String {
    min_length: Option<u16>,
    max_length: Option<u16>,
    choices: BlueprintChoices<&'static str>
  },
  Integer {
    min_value: Option<i64>,
    max_value: Option<i64>,
    choices: BlueprintChoices<i64>
  },
  Number {
    min_value: Option<f64>,
    max_value: Option<f64>,
    choices: BlueprintChoices<f64>
  },
  Boolean,
  User,
  Role,
  Mentionable,
  Channel {
    channel_types: &'static [ChannelType]
  },
  Attachment
}

impl BlueprintOptionVariant {
  fn is_same_type(self, value: &BlueprintOptionValue) -> bool {
    match (self, value) {
      (BlueprintOptionVariant::String { .. }, BlueprintOptionValue::String(..)) => true,
      (BlueprintOptionVariant::Integer { .. }, BlueprintOptionValue::Integer(..)) => true,
      (BlueprintOptionVariant::Number { .. }, BlueprintOptionValue::Number(..)) => true,
      (BlueprintOptionVariant::Boolean, BlueprintOptionValue::Boolean(..)) => true,
      (BlueprintOptionVariant::User, BlueprintOptionValue::User(..)) => true,
      (BlueprintOptionVariant::Role, BlueprintOptionValue::Role(..)) => true,
      (BlueprintOptionVariant::Mentionable, BlueprintOptionValue::User(..) | BlueprintOptionValue::Role(..)) => true,
      (BlueprintOptionVariant::Channel { .. }, BlueprintOptionValue::Channel(..)) => true,
      (BlueprintOptionVariant::Attachment, BlueprintOptionValue::Attachment(..)) => true,
      _ => false
    }
  }
}

#[derive(Debug, Clone)]
pub struct BlueprintCommandResponse {
  pub tts: bool,
  pub ephemeral: bool,
  pub content: Option<String>,
  pub embeds: Vec<CreateEmbed>
}

impl BlueprintCommandResponse {
  pub fn new(content: impl Into<String>) -> Self {
    BlueprintCommandResponse {
      content: Some(content.into()),
      ..Default::default()
    }
  }

  #[allow(dead_code)]
  pub fn new_ephemeral(content: impl Into<String>) -> Self {
    BlueprintCommandResponse {
      ephemeral: true,
      content: Some(content.into()),
      ..Default::default()
    }
  }

  #[allow(dead_code)]
  pub fn with_embeds(embeds: Vec<CreateEmbed>) -> Self {
    BlueprintCommandResponse {
      embeds,
      ..Default::default()
    }
  }

  #[allow(dead_code)]
  pub fn with_ephemeral_embeds(embeds: Vec<CreateEmbed>) -> Self {
    BlueprintCommandResponse {
      ephemeral: true,
      embeds,
      ..Default::default()
    }
  }

  fn into_response_builder(self) -> CreateInteractionResponse {
    let mut response = CreateInteractionResponseMessage::new()
      .tts(self.tts).ephemeral(self.ephemeral);
    if let Some(content) = self.content { response = response.content(content) };
    if !self.embeds.is_empty() { response = response.embeds(self.embeds) };
    CreateInteractionResponse::Message(response)
  }

  fn into_response_builder_edit(self) -> EditInteractionResponse {
    let mut response = EditInteractionResponse::new();
    if let Some(content) = self.content { response = response.content(content) };
    if !self.embeds.is_empty() { response = response.embeds(self.embeds) };
    response
  }

  pub async fn send(self, cache_http: impl CacheHttp, args: &BlueprintCommandArgs) -> MelodyResult {
    if args.deferred.get() {
      if self.ephemeral { warn!("tried to send deferred ephemeral message") };
      args.interaction.edit_response(cache_http, self.into_response_builder_edit())
        .await.context("failed to edit interaction response")?;
    } else {
      args.interaction.create_response(cache_http, self.into_response_builder())
        .await.context("failed to send interaction response")?;
    };

    Ok(())
  }
}

impl Default for BlueprintCommandResponse {
  fn default() -> Self {
    BlueprintCommandResponse {
      tts: false,
      ephemeral: false,
      content: None,
      embeds: Vec::new()
    }
  }
}

pub type BlueprintChoices<T> = &'static [(&'static str, T)];

#[derive(Debug, Clone)]
pub struct BlueprintCommandArgs {
  pub command: String,
  pub subcommands: Vec<String>,
  pub option_values: Vec<BlueprintOptionValue>,
  pub interaction: CommandInteraction,
  pub deferred: Flag
}

impl BlueprintCommandArgs {
  pub fn resolve_values<'a, R: ResolveArgumentsValues<'a>>(&'a self) -> MelodyResult<R> {
    R::resolve_values(&self.option_values, &self.interaction.data.resolved)
      .ok_or(MelodyError::COMMAND_INVALID_ARGUMENTS_STRUCTURE)
  }

  pub async fn defer(&self, cache_http: impl CacheHttp) -> MelodyResult {
    self.interaction.defer(cache_http).await
      .context("failed to defer message")?;
    self.deferred.set(true);
    Ok(())
  }
}

pub type CommandFn<T = ()> = fn(Core, BlueprintCommandArgs) -> BoxFuture<'static, MelodyResult<T>>;

pub async fn dispatch(
  core: Core,
  interaction: CommandInteraction,
  blueprint_commands: &'static [BlueprintCommand]
) -> MelodyResult {
  let decomposed = parse_command(&interaction.data, blueprint_commands)?;
  let (command, subcommands, option_values, function) = decomposed;
  function(core, BlueprintCommandArgs {
    command, subcommands,
    interaction, option_values,
    deferred: Flag::new(false)
  }).await
}

fn parse_command(
  data: &CommandData, blueprint_commands: &'static [BlueprintCommand]
) -> Result<(String, Vec<String>, Vec<BlueprintOptionValue>, CommandFn), MelodyParseCommandError> {
  fn extract_arguments_recursive(
    resolved: &CommandDataResolved,
    data_options: &[CommandDataOption],
    extracted_subcommands: &mut Vec<String>,
    extracted_options: &mut Vec<BlueprintOptionValue>
  ) -> Result<(), MelodyParseCommandError> {
    if let &[CommandDataOption {
      ref name, value:
        CommandDataOptionValue::SubCommandGroup(ref data_options) |
        CommandDataOptionValue::SubCommand(ref data_options),
      ..
    }] = data_options {
      extracted_subcommands.push(name.to_owned());
      extract_arguments_recursive(resolved, &data_options, extracted_subcommands, extracted_options)?;
    } else {
      for data_option in data_options {
        extracted_options.push(match data_option.value {
          CommandDataOptionValue::String(ref value) => BlueprintOptionValue::String(value.clone()),
          CommandDataOptionValue::Integer(value) => BlueprintOptionValue::Integer(value),
          CommandDataOptionValue::Number(value) => BlueprintOptionValue::Number(value),
          CommandDataOptionValue::Boolean(value) => BlueprintOptionValue::Boolean(value),
          CommandDataOptionValue::User(value) => BlueprintOptionValue::User(value),
          CommandDataOptionValue::Role(value) => BlueprintOptionValue::Role(value),
          CommandDataOptionValue::Channel(value) => BlueprintOptionValue::Channel(value),
          CommandDataOptionValue::Attachment(value) => BlueprintOptionValue::Attachment(value),
          CommandDataOptionValue::Mentionable(id) => {
            let user_id = UserId::new(id.get());
            let role_id = RoleId::new(id.get());

            if resolved.users.contains_key(&user_id) {
              BlueprintOptionValue::User(user_id)
            } else if resolved.roles.contains_key(&role_id) {
              BlueprintOptionValue::Role(role_id)
            } else {
              return Err(MelodyParseCommandError::UnresolvedGenericId(id))
            }
          },
          _ => return Err(MelodyParseCommandError::InvalidStructure)
        });
      };
    };

    Ok(())
  }

  let mut extracted_subcommands = Vec::new();
  let mut extracted_options = Vec::new();
  extract_arguments_recursive(&data.resolved, &data.options, &mut extracted_subcommands, &mut extracted_options)?;
  let blueprint_command = find_command(blueprint_commands, &data.name)
    .ok_or(MelodyParseCommandError::NoCommandFound)?;
  let (blueprint_options, function) = blueprint_command.root.get(&extracted_subcommands)
    .ok_or(MelodyParseCommandError::NoCommandFound)?;
  blueprint_options.into_iter().zip(extracted_options.iter())
    .all(|(blueprint_option, value)| blueprint_option.variant.is_same_type(value))
    .then(|| (data.name.clone(), extracted_subcommands, extracted_options, function))
    .ok_or(MelodyParseCommandError::InvalidStructure)
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlueprintOptionValue {
  String(String),
  Integer(i64),
  Number(f64),
  Boolean(bool),
  User(UserId),
  Role(RoleId),
  Channel(ChannelId),
  Attachment(AttachmentId)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleOrUser {
  User(UserId),
  Role(RoleId)
}

pub trait ResolveArgumentValue<'a> {
  fn resolve_value(
    option_value: Option<&'a BlueprintOptionValue>,
    resolved: &'a CommandDataResolved
  ) -> Option<Self> where Self: Sized;
}

macro_rules! impl_resolve_argument_value {
  ($lt:lifetime, $resolved:ident, $Type:ty, $pat:pat => $expr:expr) => {
    impl<'a> ResolveArgumentValue<'a> for $Type {
      fn resolve_value(option_value: Option<&'a BlueprintOptionValue>, $resolved: &'a CommandDataResolved) -> Option<Self> {
        if let Some(&$pat) = option_value { $expr } else { None }
      }
    }
  };
}

impl_resolve_argument_value!('a, _r, &'a str, BlueprintOptionValue::String(ref value) => Some(value.as_str()));
impl_resolve_argument_value!('a, _r, &'a String, BlueprintOptionValue::String(ref value) => Some(value));
impl_resolve_argument_value!('a, _r, String, BlueprintOptionValue::String(ref value) => Some(value.to_owned()));
impl_resolve_argument_value!('a, _r, i64, BlueprintOptionValue::Integer(value) => Some(value));
impl_resolve_argument_value!('a, _r, u64, BlueprintOptionValue::Integer(value) => u64::try_from(value).ok());
impl_resolve_argument_value!('a, _r, i32, BlueprintOptionValue::Integer(value) => i32::try_from(value).ok());
impl_resolve_argument_value!('a, _r, u32, BlueprintOptionValue::Integer(value) => u32::try_from(value).ok());
impl_resolve_argument_value!('a, _r, isize, BlueprintOptionValue::Integer(value) => isize::try_from(value).ok());
impl_resolve_argument_value!('a, _r, usize, BlueprintOptionValue::Integer(value) => usize::try_from(value).ok());
impl_resolve_argument_value!('a, _r, NonZeroU64, BlueprintOptionValue::Integer(value) => u64::try_from(value).ok().and_then(NonZeroU64::new));
impl_resolve_argument_value!('a, _r, NonZeroU32, BlueprintOptionValue::Integer(value) => u32::try_from(value).ok().and_then(NonZeroU32::new));
impl_resolve_argument_value!('a, _r, NonZeroUsize, BlueprintOptionValue::Integer(value) => usize::try_from(value).ok().and_then(NonZeroUsize::new));
impl_resolve_argument_value!('a, _r, f32, BlueprintOptionValue::Number(value) => Some(value as f32));
impl_resolve_argument_value!('a, _r, f64, BlueprintOptionValue::Number(value) => Some(value));
impl_resolve_argument_value!('a, _r, bool, BlueprintOptionValue::Boolean(value) => Some(value));
impl_resolve_argument_value!('a, _r, UserId, BlueprintOptionValue::User(value) => Some(value));
impl_resolve_argument_value!('a, _r, RoleId, BlueprintOptionValue::Role(value) => Some(value));
impl_resolve_argument_value!('a, _r, ChannelId, BlueprintOptionValue::Channel(value) => Some(value));
impl_resolve_argument_value!('a, _r, AttachmentId, BlueprintOptionValue::Attachment(value) => Some(value));
impl_resolve_argument_value!('a, resolved, &'a User, BlueprintOptionValue::User(id) => resolved.users.get(&id));
impl_resolve_argument_value!('a, resolved, &'a Role, BlueprintOptionValue::Role(id) => resolved.roles.get(&id));
impl_resolve_argument_value!('a, resolved, &'a PartialMember, BlueprintOptionValue::User(id) => resolved.members.get(&id));
impl_resolve_argument_value!('a, resolved, &'a PartialChannel, BlueprintOptionValue::Channel(id) => resolved.channels.get(&id));
impl_resolve_argument_value!('a, resolved, &'a Attachment, BlueprintOptionValue::Attachment(id) => resolved.attachments.get(&id));

impl<'a> ResolveArgumentValue<'a> for RoleOrUser {
  fn resolve_value(option_value: Option<&'a BlueprintOptionValue>, _: &'a CommandDataResolved) -> Option<Self> {
    option_value.and_then(|option_value| match option_value {
      &BlueprintOptionValue::User(user) => Some(RoleOrUser::User(user)),
      &BlueprintOptionValue::Role(role) => Some(RoleOrUser::Role(role)),
      _ => None
    })
  }
}

impl<'a, T: ResolveArgumentValue<'a>> ResolveArgumentValue<'a> for Option<T> {
  fn resolve_value(option_value: Option<&'a BlueprintOptionValue>, resolved: &'a CommandDataResolved) -> Option<Option<T>> {
    Some(option_value.and_then(|option_value| T::resolve_value(Some(option_value), resolved)))
  }
}

impl<'a> ResolveArgumentValue<'a> for () {
  fn resolve_value(_: Option<&'a BlueprintOptionValue>, _: &'a CommandDataResolved) -> Option<Self> where Self: Sized {
    Some(())
  }
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parsed<T>(pub T);

impl<'a, T> ResolveArgumentValue<'a> for Parsed<T> where T: FromStr {
  fn resolve_value(option_value: Option<&'a BlueprintOptionValue>, resolved: &'a CommandDataResolved) -> Option<Self> where Self: Sized {
    <&'a str>::resolve_value(option_value, resolved).and_then(|s| s.parse::<T>().ok()).map(Parsed)
  }
}

macro_rules! impl_resolve_arguments_values {
  ($($G:ident),*) => {
    impl<'a, $($G: ResolveArgumentValue<'a>),*> ResolveArgumentsValues<'a> for ($($G,)*) {
      fn resolve_values(option_values: &'a [BlueprintOptionValue], resolved: &'a CommandDataResolved) -> Option<Self> {
        let mut option_values = option_values.into_iter();
        Some(($($G::resolve_value(option_values.next(), resolved)?,)*))
      }
    }
  };
}

pub trait ResolveArgumentsValues<'a> {
  fn resolve_values(
    option_values: &'a [BlueprintOptionValue],
    resolved: &'a CommandDataResolved
  ) -> Option<Self> where Self: Sized;
}

impl_resolve_arguments_values!(A, B);
impl_resolve_arguments_values!(A, B, C);
impl_resolve_arguments_values!(A, B, C, D);
impl_resolve_arguments_values!(A, B, C, D, E);
impl_resolve_arguments_values!(A, B, C, D, E, F);
impl_resolve_arguments_values!(A, B, C, D, E, F, G);
impl_resolve_arguments_values!(A, B, C, D, E, F, G, H);

impl<'a, T: ResolveArgumentValue<'a>> ResolveArgumentsValues<'a> for T {
  fn resolve_values(option_values: &'a [BlueprintOptionValue], resolved: &'a CommandDataResolved) -> Option<Self> {
    T::resolve_value(option_values.first(), resolved)
  }
}

// Avert your eyes

pub const DEFAULT_COMMAND_TYPE: CommandType = CommandType::ChatInput;
pub const DEFAULT_COMMAND_CONTEXT: BlueprintCommandContext = BlueprintCommandContext::OnlyInGuild;

#[macro_export]
macro_rules! make_concrete_asyncfn {
  ($function:expr => ($($arg:ident: $Ty:ty),* $(,)?)) => {
    (|$($arg: $Ty),*| Box::pin($function($($arg),*)) as $crate::blueprint::BoxFuture<'static, _>) as fn($($Ty),*) -> _
  };
}

#[macro_export]
macro_rules! default_expr {
  ($default:expr, $expr:expr) => ($expr);
  ($default:expr $(,)?) => ($default);
}

#[macro_export]
macro_rules! blueprint_command {
  {
    name: $name:expr,
    description: $description:expr,
    $(info: [$($info:expr),+ $(,)?],)?
    $(usage: [$($usage:expr),+ $(,)?],)?
    $(examples: [$($examples:expr),+ $(,)?],)?
    $(plugin: $plugin:expr,)?
    $(command_type: $command_type:expr,)?
    $(context: $context:expr,)?
    $(default_permissions: $default_permissions:expr,)?
    subcommands: [$($subcommand:expr),+ $(,)?]
  } => ($crate::blueprint::BlueprintCommand {
    name: $name,
    description: $description,
    info: $crate::default_expr!(None, $(Some(&[$($info),*]))?),
    usage: $crate::default_expr!(None, $(Some(&[$($usage),*]))?),
    examples: $crate::default_expr!(None, $(Some(&[$($examples),*]))?),
    plugin: $crate::default_expr!(None, $(Some($plugin))?),
    command_type: $crate::default_expr!($crate::blueprint::DEFAULT_COMMAND_TYPE, $($command_type)?),
    context: $crate::default_expr!($crate::blueprint::DEFAULT_COMMAND_CONTEXT, $($context)?),
    default_permissions: $crate::default_expr!(None, $(Some($default_permissions))?),
    root: $crate::blueprint::BlueprintRoot::CommandContainer {
      subcommands: &[$($subcommand,)+]
    }
  });
  {
    name: $name:expr,
    description: $description:expr,
    $(info: [$($info:expr),+ $(,)?],)?
    $(usage: [$($usage:expr),+ $(,)?],)?
    $(examples: [$($examples:expr),+ $(,)?],)?
    $(plugin: $plugin:expr,)?
    $(command_type: $command_type:expr,)?
    $(context: $context:expr,)?
    $(default_permissions: $default_permissions:expr,)?
    arguments: [$($option:expr),* $(,)?],
    function: $function:expr
  } => ($crate::blueprint::BlueprintCommand {
    name: $name,
    description: $description,
    info: $crate::default_expr!(None, $(Some(&[$($info),*]))?),
    usage: $crate::default_expr!(None, $(Some(&[$($usage),*]))?),
    examples: $crate::default_expr!(None, $(Some(&[$($examples),*]))?),
    plugin: $crate::default_expr!(None, $(Some($plugin))?),
    command_type: $crate::default_expr!($crate::blueprint::DEFAULT_COMMAND_TYPE, $($command_type)?),
    context: $crate::default_expr!($crate::blueprint::DEFAULT_COMMAND_CONTEXT, $($context)?),
    default_permissions: $crate::default_expr!(None, $(Some($default_permissions))?),
    root: $crate::blueprint::BlueprintRoot::Command {
      function: crate::utils::NoDebug(
        $crate::make_concrete_asyncfn!($function => (c: _, a: _))
      ),
      options: &[$($option,)*]
    }
  });
}

#[macro_export]
macro_rules! blueprint_subcommand {
  {
    name: $name:expr,
    description: $description:expr,
    subcommands: [$($subcommand:expr),+ $(,)?]
  } => ($crate::blueprint::BlueprintSubcommand {
    name: $name,
    description: $description,
    root: $crate::blueprint::BlueprintRoot::CommandContainer {
      subcommands: &[$($subcommand,)+]
    }
  });
  {
    name: $name:expr,
    description: $description:expr,
    arguments: [$($option:expr),* $(,)?],
    function: $function:expr
  } => ($crate::blueprint::BlueprintSubcommand {
    name: $name,
    description: $description,
    root: $crate::blueprint::BlueprintRoot::Command {
      function: crate::utils::NoDebug(
        $crate::make_concrete_asyncfn!($function => (c: _, a: _))
      ),
      options: &[$($option,)*]
    }
  });
}

#[macro_export]
macro_rules! blueprint_argument {
  (String {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
    $(, min_length: $min_length:expr)?
    $(, max_length: $max_length:expr)?
    $(, choices: [$($choice:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::String {
      min_length: $crate::default_expr!(None, $(Some($min_length))?),
      max_length: $crate::default_expr!(None, $(Some($max_length))?),
      choices: $crate::default_expr!(&[], $(&[$($choice,)+])?)
    }
  });
  (Integer {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
    $(, min_value: $min_value:expr)?
    $(, max_value: $max_value:expr)?
    $(, choices: [$($choice:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::Integer {
      min_value: $crate::default_expr!(None, $(Some($min_value))?),
      max_value: $crate::default_expr!(None, $(Some($max_value))?),
      choices: $crate::default_expr!(&[], $(&[$($choice,)+])?)
    }
  });
  (Number {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
    $(, min_value: $min_value:expr)?
    $(, max_value: $max_value:expr)?
    $(, choices: [$($choice:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::Number {
      min_value: $crate::default_expr!(None, $(Some($min_value))?),
      max_value: $crate::default_expr!(None, $(Some($max_value))?),
      choices: $crate::default_expr!(&[], $(&[$($choice,)+])?)
    }
  });
  (Boolean {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::Boolean
  });
  (User {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::User
  });
  (Role {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::Role
  });
  (Mentionable {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::Mentionable
  });
  (Channel {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
    $(, channel_types: [$($channel_type:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::Channel {
      channel_types: $crate::default_expr!(&[], $(&[$($channel_type,)+])?)
    }
  });
  (Attachment {
    name: $name:expr,
    description: $description:expr,
    required: $required:expr
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $required,
    variant: $crate::blueprint::BlueprintOptionVariant::Attachment
  });
}

#[derive(Clone)]
pub struct DisplayCommands<'a>(pub &'a [BlueprintCommand]);

impl<'a> fmt::Display for DisplayCommands<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_list().entries(self.0.iter().map(|blueprint| blueprint.name)).finish()
  }
}
