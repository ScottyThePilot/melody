#![allow(missing_debug_implementations)]
use crate::{MelodyError, MelodyResult};
use crate::utils::{Blockify, Contextualize};

use itertools::Itertools;
use serenity::builder::{
  CreateApplicationCommand,
  CreateApplicationCommands,
  CreateApplicationCommandOption,
  CreateInteractionResponse,
  CreateInteractionResponseData,
  CreateEmbed
};
use serenity::client::Context;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::application_command::{
  ApplicationCommandInteraction, CommandData, CommandDataOption, CommandDataOptionValue
};
use serenity::model::application::command::{CommandType, CommandOptionType};
use serenity::model::channel::{Attachment, ChannelType, PartialChannel};
use serenity::model::permissions::Permissions;
use serenity::model::guild::{PartialMember, Role};
use serenity::model::user::User;
use serenity::utils::Color;
pub use serenity::futures::future::BoxFuture;

use std::collections::HashSet;
use std::fmt;

macro_rules! when {
  ($ident:ident, $expr:expr) => {
    if let Some($ident) = $ident { $expr; };
  };
}

macro_rules! builder {
  ($subject:expr) => (builder!($subject, build));
  ($subject:expr, $build:ident) => (move |builder| {
    $subject.$build(builder);
    builder
  });
}

pub fn commands_builder<'a>(commands: impl IntoIterator<Item = &'a BlueprintCommand>)
-> impl FnOnce(&mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
  move |builder| commands.into_iter().fold(builder, move |builder, &blueprint| {
    builder.create_application_command(builder!(blueprint));
    builder
  })
}

pub fn find_command(commands: &'static [BlueprintCommand], name: &str) -> Option<&'static BlueprintCommand> {
  commands.into_iter().find(|command| command.name == name)
}

pub fn find_subcommand(subcommands: &'static [BlueprintSubcommand], name: &str) -> Option<&'static BlueprintSubcommand> {
  subcommands.into_iter().find(|subcommand| subcommand.name == name)
}

pub fn command_embed(command: &'static BlueprintCommand, color: Color) -> CreateEmbed {
  let mut builder = CreateEmbed::default();
  builder.title(crate::utils::capitalize_words(command.name));
  builder.description(command.description);
  builder.color(color);
  builder.field("Usage", command.stringify_usage(), false);
  builder.field("Examples", command.stringify_examples(), false);
  builder.field("Required Permissions", command.stringify_permissions(), false);
  builder.field("Allowed in DM", if command.allow_in_dms { "Yes" } else { "No" }, false);
  builder.footer(|builder| {
    builder.text(format_args!("Melody v{}", env!("CARGO_PKG_VERSION")))
  });

  builder
}

pub fn command_list_embed(commands: &'static [BlueprintCommand], permissions: Permissions, color: Color) -> CreateEmbed {
  let mut builder = CreateEmbed::default();
  let mut commands = commands.to_owned();
  commands.sort_by_key(|command| command.name);
  let body = commands.into_iter()
    .filter(|command| permissions.contains(command.default_permissions.unwrap_or(Permissions::empty())))
    .map(|command| format!("`/{}`: *{}*", command.name, command.description))
    .join("\n");

  builder.title("Command Help");
  builder.description("Below is a list of commands, each with a short description of what they do.");
  builder.color(color);
  builder.field("Command List", body, false);
  builder.footer(|builder| {
    builder.text(format_args!("Melody v{}", env!("CARGO_PKG_VERSION")))
  });

  builder
}

#[derive(Clone, Copy)]
pub struct BlueprintCommand {
  pub name: &'static str,
  pub description: &'static str,
  pub usage: Option<&'static [&'static str]>,
  pub examples: Option<&'static [&'static str]>,
  pub plugin: Option<&'static str>,
  pub command_type: CommandType,
  pub allow_in_dms: bool,
  pub default_permissions: Option<Permissions>,
  pub root: BlueprintRoot
}

impl BlueprintCommand {
  fn build(self, builder: &mut CreateApplicationCommand) {
    builder.name(self.name);
    builder.description(self.description);
    builder.kind(self.command_type);
    builder.dm_permission(self.allow_in_dms);
    if let Some(permissions) = self.default_permissions {
      builder.default_member_permissions(permissions);
    };

    self.root.build_command(builder);
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

  fn stringify_usage(self) -> String {
    self.usage.map_or_else(|| "none".to_owned(), |usage| {
      usage.into_iter().map(Blockify::new).join("\n")
    })
  }

  fn stringify_examples(self) -> String {
    self.examples.map_or_else(|| "none".to_owned(), |example| {
      example.into_iter().map(Blockify::new).join("\n")
    })
  }

  fn stringify_permissions(self) -> String {
    self.default_permissions.map_or_else(|| "`EVERYONE`".to_owned(), |p| p.to_string())
  }
}

#[derive(Clone, Copy)]
pub struct BlueprintSubcommand {
  pub name: &'static str,
  pub description: &'static str,
  pub root: BlueprintRoot
}

impl BlueprintSubcommand {
  fn build(self, builder: &mut CreateApplicationCommandOption) {
    builder.name(self.name);
    builder.description(self.description);
    self.root.build_command_option(builder);
  }
}

#[derive(Clone, Copy)]
pub enum BlueprintRoot {
  Command {
    function: CommandFn,
    options: &'static [BlueprintOption]
  },
  CommandContainer {
    subcommands: &'static [BlueprintSubcommand]
  }
}

impl BlueprintRoot {
  fn build_command(self, builder: &mut CreateApplicationCommand) {
    match self {
      BlueprintRoot::Command { options, .. } => {
        for &option in options {
          builder.create_option(|builder| {
            option.build(builder);
            builder
          });
        };
      },
      BlueprintRoot::CommandContainer { subcommands, .. } => {
        for &subcommand in subcommands {
          builder.create_option(|builder| {
            subcommand.build(builder);
            builder
          });
        };
      }
    };
  }

  fn build_command_option(self, builder: &mut CreateApplicationCommandOption) {
    match self {
      BlueprintRoot::Command { options, .. } => {
        builder.kind(CommandOptionType::SubCommand);
        for &option in options {
          builder.create_sub_option(builder!(option));
        };
      },
      BlueprintRoot::CommandContainer { subcommands, .. } => {
        builder.kind(CommandOptionType::SubCommandGroup);
        for &subcommand in subcommands {
          builder.create_sub_option(builder!(subcommand));
        };
      }
    };
  }
}

#[derive(Debug, Clone, Copy)]
pub struct BlueprintOption {
  pub name: &'static str,
  pub description: &'static str,
  pub required: bool,
  pub data: BlueprintOptionData
}

impl BlueprintOption {
  fn build(self, builder: &mut CreateApplicationCommandOption) {
    builder.name(self.name);
    builder.description(self.description);
    builder.required(self.required);
    self.data.build(builder);
  }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum BlueprintOptionData {
  String {
    min_length: Option<u16>,
    max_length: Option<u16>,
    choices: BlueprintChoices<&'static str>
  },
  Integer {
    min_value: Option<i32>,
    max_value: Option<i32>,
    choices: BlueprintChoices<i32>
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

impl BlueprintOptionData {
  fn build(self, builder: &mut CreateApplicationCommandOption) {
    match self {
      BlueprintOptionData::String { min_length, max_length, choices } => {
        builder.kind(CommandOptionType::String);
        when!(min_length, builder.min_length(min_length));
        when!(max_length, builder.max_length(max_length));
        for &(name, value) in choices {
          builder.add_string_choice(name, value);
        };
      },
      BlueprintOptionData::Integer { min_value, max_value, choices } => {
        builder.kind(CommandOptionType::Integer);
        when!(min_value, builder.min_int_value(min_value));
        when!(max_value, builder.max_int_value(max_value));
        for &(name, value) in choices {
          builder.add_int_choice(name, value);
        };
      },
      BlueprintOptionData::Number { min_value, max_value, choices } => {
        builder.kind(CommandOptionType::Number);
        when!(min_value, builder.min_number_value(min_value));
        when!(max_value, builder.max_number_value(max_value));
        for &(name, value) in choices {
          builder.add_number_choice(name, value);
        };
      },
      BlueprintOptionData::Boolean => {
        builder.kind(CommandOptionType::Boolean);
      },
      BlueprintOptionData::User => {
        builder.kind(CommandOptionType::User);
      },
      BlueprintOptionData::Role => {
        builder.kind(CommandOptionType::Role);
      },
      BlueprintOptionData::Mentionable => {
        builder.kind(CommandOptionType::Mentionable);
      },
      BlueprintOptionData::Channel { channel_types } => {
        builder.kind(CommandOptionType::Channel);
        if !channel_types.is_empty() {
          builder.channel_types(channel_types);
        };
      },
      BlueprintOptionData::Attachment => {
        builder.kind(CommandOptionType::Attachment);
      }
    };
  }

  fn is_same_type(self, option: &CommandDataOptionValue) -> bool {
    match (self, option) {
      (BlueprintOptionData::String { .. }, CommandDataOptionValue::String(..)) => true,
      (BlueprintOptionData::Integer { .. }, CommandDataOptionValue::Integer(..)) => true,
      (BlueprintOptionData::Number { .. }, CommandDataOptionValue::Number(..)) => true,
      (BlueprintOptionData::Boolean, CommandDataOptionValue::Boolean(..)) => true,
      (BlueprintOptionData::User | BlueprintOptionData::Mentionable, CommandDataOptionValue::User(..)) => true,
      (BlueprintOptionData::Role | BlueprintOptionData::Mentionable, CommandDataOptionValue::Role(..)) => true,
      (BlueprintOptionData::Channel { .. } | BlueprintOptionData::Mentionable, CommandDataOptionValue::Channel(..)) => true,
      (BlueprintOptionData::Attachment, CommandDataOptionValue::Attachment(..)) => true,
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

  pub fn new_ephemeral(content: impl Into<String>) -> Self {
    BlueprintCommandResponse {
      ephemeral: true,
      content: Some(content.into()),
      ..Default::default()
    }
  }

  pub fn with_embeds(embeds: Vec<CreateEmbed>) -> Self {
    BlueprintCommandResponse {
      embeds,
      ..Default::default()
    }
  }

  pub fn with_ephemeral_embeds(embeds: Vec<CreateEmbed>) -> Self {
    BlueprintCommandResponse {
      ephemeral: true,
      embeds,
      ..Default::default()
    }
  }

  fn build_data(self, builder: &mut CreateInteractionResponseData) {
    builder.tts(self.tts);
    builder.ephemeral(self.ephemeral);
    if let Some(content) = self.content {
      builder.content(content);
    };
    if !self.embeds.is_empty() {
      builder.set_embeds(self.embeds);
    };
  }

  fn build(self, builder: &mut CreateInteractionResponse) {
    builder.kind(InteractionResponseType::ChannelMessageWithSource);
    builder.interaction_response_data(builder!(self, build_data));
  }

  pub async fn send(self, ctx: &Context, interaction: &ApplicationCommandInteraction) -> MelodyResult {
    interaction.create_interaction_response(&ctx, builder!(self))
      .await.context("failed to send interaction response")
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

pub struct BlueprintCommandArgs {
  pub command: String,
  pub subcommands: Vec<String>,
  pub interaction: ApplicationCommandInteraction,
  pub option_values: Vec<CommandDataOptionValue>
}

pub type BlueprintChoices<T> = &'static [(&'static str, T)];

pub type CommandFn<T = ()> = for<'f> fn(&'f Context, BlueprintCommandArgs) -> BoxFuture<'f, MelodyResult<T>>;

pub async fn dispatch(
  ctx: &Context,
  interaction: ApplicationCommandInteraction,
  blueprint_commands: &'static [BlueprintCommand]
) -> MelodyResult {
  let (command, subcommands, option_values, function) = {
    decompose_command(blueprint_commands, &interaction.data).ok_or(MelodyError::InvalidCommand)
  }?;

  function(ctx, BlueprintCommandArgs { command, subcommands, interaction, option_values }).await
}

/// Returns the name of the command executed, the list of subcommand arguments, and the list of regular arguments
fn decompose_command(
  blueprint_commands: &'static [BlueprintCommand],
  data: &CommandData
) -> Option<(String, Vec<String>, Vec<CommandDataOptionValue>, CommandFn)> {
  fn extract_command_arguments_recursive(
    data_options: &[CommandDataOption],
    extracted_subcommands: &mut Vec<String>,
    extracted_options: &mut Vec<CommandDataOptionValue>
  ) {
    for data_option in data_options {
      if let CommandOptionType::SubCommandGroup | CommandOptionType::SubCommand = data_option.kind {
        extracted_subcommands.push(data_option.name.clone());
        extract_command_arguments_recursive(&data_option.options, extracted_subcommands, extracted_options);
      } else if let Some(data_value) = &data_option.resolved {
        extracted_options.push(data_value.clone());
      };
    };
  }

  fn validate_subcommands_recursive(
    blueprint_root: BlueprintRoot,
    extracted_subcommands: &[String]
  ) -> Option<(&'static [BlueprintOption], CommandFn)> {
    match blueprint_root {
      // this command root terminates at a command with regular options,
      // return those regular options if `extracted_subcommands` agrees that the command terminates here
      BlueprintRoot::Command { options, function } => extracted_subcommands.is_empty().then(|| (options, function)),
      // this command root contains other subcommands, split the first element off of `extracted_subcommands`
      BlueprintRoot::CommandContainer { subcommands } => {
        let (subcommand, remaining_subcommands) = extracted_subcommands.split_first()?;
        // if the first element of `extracted_subcommands` can be split off,
        // find the corresponding subcommand blueprint and recursively call the validator on it
        find_subcommand(subcommands, subcommand).and_then(|blueprint_subcommand| {
          validate_subcommands_recursive(blueprint_subcommand.root, remaining_subcommands)
        })
      }
    }
  }

  let mut extracted_subcommands = Vec::new();
  let mut extracted_options = Vec::new();
  extract_command_arguments_recursive(&data.options, &mut extracted_subcommands, &mut extracted_options);
  let blueprint_command = find_command(blueprint_commands, &data.name)?;
  let (blueprint_options, function) = validate_subcommands_recursive(blueprint_command.root, &extracted_subcommands)?;
  blueprint_options.into_iter().zip(extracted_options.iter())
    .all(|(blueprint_option, option)| blueprint_option.data.is_same_type(option))
    .then(|| (data.name.clone(), extracted_subcommands, extracted_options, function))
}



macro_rules! impl_resolve_argument_value {
  ($Type:ty, $pat:pat => $expr:expr) => {
    impl ResolveArgumentValue for $Type {
      fn resolve_value(option_value: Option<CommandDataOptionValue>) -> Option<Self> {
        if let Some($pat) = option_value { $expr } else { None }
      }
    }
  };
}

pub trait ResolveArgumentValue {
  fn resolve_value(option_value: Option<CommandDataOptionValue>) -> Option<Self> where Self: Sized;
}

impl_resolve_argument_value!(String, CommandDataOptionValue::String(value) => Some(value));
impl_resolve_argument_value!(i64, CommandDataOptionValue::Integer(value) => Some(value));
impl_resolve_argument_value!(f64, CommandDataOptionValue::Number(value) => Some(value));
impl_resolve_argument_value!(bool, CommandDataOptionValue::Boolean(value) => Some(value));
impl_resolve_argument_value!(User, CommandDataOptionValue::User(value, _) => Some(value));
impl_resolve_argument_value!(PartialMember, CommandDataOptionValue::User(_, value) => value);
impl_resolve_argument_value!(PartialChannel, CommandDataOptionValue::Channel(value) => Some(value));
impl_resolve_argument_value!(Role, CommandDataOptionValue::Role(value) => Some(value));
impl_resolve_argument_value!(Attachment, CommandDataOptionValue::Attachment(value) => Some(value));

impl<T: ResolveArgumentValue> ResolveArgumentValue for Option<T> {
  fn resolve_value(option_value: Option<CommandDataOptionValue>) -> Option<Option<T>> {
    match option_value {
      None => Some(None),
      Some(option_value) => T::resolve_value(Some(option_value)).map(Some)
    }
  }
}

macro_rules! impl_resolve_arguments_values {
  ($($G:ident),*) => {
    impl<$($G: ResolveArgumentValue),*> ResolveArgumentsValues for ($($G,)*) {
      fn resolve_values(option_values: Vec<CommandDataOptionValue>) -> Option<Self> {
        let mut option_values = option_values.into_iter();
        Some(($($G::resolve_value(option_values.next())?,)*))
      }
    }
  };
}

pub trait ResolveArgumentsValues {
  fn resolve_values(option_values: Vec<CommandDataOptionValue>) -> Option<Self> where Self: Sized;
}

impl<T: ResolveArgumentValue> ResolveArgumentsValues for T {
  fn resolve_values(option_values: Vec<CommandDataOptionValue>) -> Option<Self> where Self: Sized {
    let mut option_values = option_values.into_iter();
    T::resolve_value(option_values.next())
  }
}

impl_resolve_arguments_values!(A, B);
impl_resolve_arguments_values!(A, B, C);
impl_resolve_arguments_values!(A, B, C, D);
impl_resolve_arguments_values!(A, B, C, D, E);
impl_resolve_arguments_values!(A, B, C, D, E, F);
impl_resolve_arguments_values!(A, B, C, D, E, F, G);
impl_resolve_arguments_values!(A, B, C, D, E, F, G, H);

pub fn resolve_arguments<R: ResolveArgumentsValues>(option_values: Vec<CommandDataOptionValue>) -> MelodyResult<R> {
  R::resolve_values(option_values).ok_or(MelodyError::InvalidArguments)
}

// Avert your eyes

#[macro_export]
macro_rules! default_expr {
  ($default:expr, $expr:expr) => ($expr);
  ($default:expr $(,)?) => ($default);
}

#[macro_export]
macro_rules! blueprint_command {
  {
    name: $name:literal,
    description: $description:literal,
    $(usage: [$($usage:expr),+ $(,)?],)?
    $(examples: [$($examples:expr),+ $(,)?],)?
    $(plugin: $plugin:literal,)?
    $(command_type: $command_type:expr,)?
    $(allow_in_dms: $allow_in_dms:expr,)?
    $(default_permissions: $default_permissions:expr,)?
    subcommands: [$($subcommand:expr),+ $(,)?]
  } => ($crate::blueprint::BlueprintCommand {
    name: $name,
    description: $description,
    usage: $crate::default_expr!(None, $(Some(&[$($usage),*]))?),
    examples: $crate::default_expr!(None, $(Some(&[$($examples),*]))?),
    plugin: $crate::default_expr!(None, $(Some($plugin))?),
    command_type: $crate::default_expr!(CommandType::ChatInput, $($command_type)?),
    allow_in_dms: $crate::default_expr!(false, $($allow_in_dms)?),
    default_permissions: $crate::default_expr!(None, $(Some($default_permissions))?),
    root: $crate::blueprint::BlueprintRoot::CommandContainer {
      subcommands: &[$($subcommand,)+]
    }
  });
  {
    name: $name:literal,
    description: $description:literal,
    $(usage: [$($usage:expr),+ $(,)?],)?
    $(examples: [$($examples:expr),+ $(,)?],)?
    $(plugin: $plugin:literal,)?
    $(command_type: $command_type:expr,)?
    $(allow_in_dms: $allow_in_dms:expr,)?
    $(default_permissions: $default_permissions:expr,)?
    arguments: [$($option:expr),* $(,)?],
    function: $function:expr
  } => ($crate::blueprint::BlueprintCommand {
    name: $name,
    description: $description,
    usage: $crate::default_expr!(None, $(Some(&[$($usage),*]))?),
    examples: $crate::default_expr!(None, $(Some(&[$($examples),*]))?),
    plugin: $crate::default_expr!(None, $(Some($plugin))?),
    command_type: $crate::default_expr!(CommandType::ChatInput, $($command_type)?),
    allow_in_dms: $crate::default_expr!(false, $($allow_in_dms)?),
    default_permissions: $crate::default_expr!(None, $(Some($default_permissions))?),
    root: $crate::blueprint::BlueprintRoot::Command {
      function: $function,
      options: &[$($option)*]
    }
  });
}

#[macro_export]
macro_rules! blueprint_subcommand {
  {
    name: $name:literal,
    description: $description:literal,
    subcommands: [$($subcommand:expr),+ $(,)?]
  } => ($crate::blueprint::BlueprintSubcommand {
    name: $name,
    description: $description,
    root: $crate::blueprint::BlueprintRoot::CommandContainer {
      subcommands: &[$($subcommand,)+]
    }
  });
  {
    name: $name:literal,
    description: $description:literal,
    arguments: [$($option:expr),* $(,)?],
    function: $function:expr
  } => ($crate::blueprint::BlueprintSubcommand {
    name: $name,
    description: $description,
    root: $crate::blueprint::BlueprintRoot::Command {
      function: $function,
      options: &[$($option,)*]
    }
  });
}

#[macro_export]
macro_rules! blueprint_argument {
  (String {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
    $(, min_length: $min_length:expr)?
    $(, max_length: $max_length:expr)?
    $(, choices: [$($choice:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::String {
      min_length: $crate::default_expr!(None, $(Some($min_length))?),
      max_length: $crate::default_expr!(None, $(Some($max_length))?),
      choices: $crate::default_expr!(&[], $(&[$($choice,)+])?)
    }
  });
  (Integer {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
    $(, min_value: $min_value:expr)?
    $(, max_value: $max_value:expr)?
    $(, choices: [$($choice:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::Integer {
      min_value: $crate::default_expr!(None, $(Some($min_value))?),
      max_value: $crate::default_expr!(None, $(Some($max_value))?),
      choices: $crate::default_expr!(&[], $(&[$($choice,)+])?)
    }
  });
  (Number {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
    $(, min_value: $min_value:expr)?
    $(, max_value: $max_value:expr)?
    $(, choices: [$($choice:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::Number {
      min_value: $crate::default_expr!(None, $(Some($min_value))?),
      max_value: $crate::default_expr!(None, $(Some($max_value))?),
      choices: $crate::default_expr!(&[], $(&[$($choice,)+])?)
    }
  });
  (Boolean {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::Boolean
  });
  (User {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::User
  });
  (Role {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::Role
  });
  (Mentionable {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::Mentionable
  });
  (Channel {
    name: $name:literal,
    description: $description:literal,
    $(, required: $required:expr)?
    $(, channel_types: [$($channel_type:expr),+ $(,)?])?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::Channel {
      channel_types: &[$($channel_type,)+]
    }
  });
  (Attachment {
    name: $name:literal,
    description: $description:literal
    $(, required: $required:expr)?
  }) => ($crate::blueprint::BlueprintOption {
    name: $name,
    description: $description,
    required: $crate::default_expr!(false, $($required)?),
    data: $crate::blueprint::BlueprintOptionData::Attachment
  });
}

#[derive(Clone)]
pub struct DisplayCommands<'a>(pub &'a [BlueprintCommand]);

impl<'a> fmt::Display for DisplayCommands<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.debug_list().entries(self.0.iter().map(|blueprint| blueprint.name)).finish()
  }
}
