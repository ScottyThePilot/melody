#[macro_use]
extern crate thiserror;

use std::collections::VecDeque;
use std::str::FromStr;
use std::any::type_name;

#[derive(Debug, Clone, Copy)]
pub struct Command<T: 'static, H: 'static> {
  pub name: &'static str,
  pub help: H,
  pub body: CommandBody<T, H>
}

impl<T: 'static, H: 'static> Command<T, H> {
  pub const fn new_group(name: &'static str, help: H, commands: Commands<T, H>) -> Self {
    Command { name, help, body: CommandBody::Group { commands } }
  }

  pub const fn new_target(name: &'static str, help: H, target: T) -> Self {
    Command { name, help, body: CommandBody::Target { target } }
  }

  pub const fn subcommands(&self) -> Option<&'static [Self]> {
    if let CommandBody::Group { commands } = self.body {
      Some(commands)
    } else {
      None
    }
  }
}

pub type Commands<T, H> = &'static [Command<T, H>];

#[derive(Debug, Clone, Copy)]
pub enum CommandBody<T: 'static, H: 'static> {
  Group {
    commands: Commands<T, H>
  },
  Target {
    target: T
  }
}

#[derive(Debug, Clone)]
pub struct CommandOutput<T: 'static, H: 'static> {
  pub target: &'static T,
  pub help: &'static H,
  pub remaining_args: Box<[String]>
}

#[derive(Debug, Error)]
pub enum CommandError {
  #[error("insufficient arguments, expected one of {0:?}")]
  InsufficientArgsCommand(Vec<&'static str>),
  #[error("insufficient arguments, expected {0}")]
  InsufficientArgs(&'static str),
  #[error("command {0:?} not found")]
  CommandNotFound(String),
  #[error("illegal character {0:?}")]
  IllegalCharacter(char),
  #[error("illegal escape code sequence {0:?}")]
  IllegalEscapeCodeSequence(char),
  #[error("unexpected quote")]
  UnexpectedQuote,
  #[error("unexpected end of line in escape sequence")]
  UnexpectedEndOfLineInEscapeSequence,
  #[error("unexpected end of line in quote")]
  UnexpectedEndOfLineInQuote,
  #[error("failed to parse argument: {0}")]
  ArgsFromStrError(GenericError)
}

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub fn apply<T: 'static, H: 'static>(input: &str, commands: Commands<T, H>) -> Result<CommandOutput<T, H>, CommandError> {
  split_args(input).and_then(|args| {
    let mut args = VecDeque::from(args);
    let (target, help) = find_target_args(&mut args, commands)?;
    let remaining_args = Vec::from(args).into_boxed_slice();
    Ok(CommandOutput { target, help, remaining_args })
  })
}

#[derive(Debug, Clone, Copy)]
pub enum CommandNode<T: 'static, H: 'static> {
  Command(&'static Command<T, H>),
  Commands(Commands<T, H>)
}

impl<T, H> CommandNode<T, H> {
  pub fn traverse(mut self, mut args: &[String]) -> TraverseResult<'_, T, H> {
    while let Some((arg, args_remaining)) = args.split_first() {
      args = args_remaining;

      let (commands, command) = match self {
        Self::Commands(commands) => (Some(commands), None),
        Self::Command(command) => match command.body {
          CommandBody::Target { .. } => (None, Some(command)),
          CommandBody::Group { commands } => (Some(commands), Some(command))
        }
      };

      let Some(commands) = commands else {
        return TraverseResult::NotFound { commands, command, arg, args };
      };

      if let Some(command) = find_command_in_list(arg, commands) {
        self = Self::Command(command);
      } else {
        return TraverseResult::NotFound { commands: Some(commands), command, arg, args };
      };
    };

    TraverseResult::Found { node: self }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum TraverseResult<'a, T: 'static, H: 'static> {
  Found {
    node: CommandNode<T, H>
  },
  NotFound {
    commands: Option<Commands<T, H>>,
    command: Option<&'static Command<T, H>>,
    arg: &'a String,
    args: &'a [String]
  }
}

impl<'a, T, H> TraverseResult<'a, T, H> {
  pub fn command(self) -> Option<&'static Command<T, H>> {
    match self {
      Self::Found { node: CommandNode::Command(command) } => Some(command),
      Self::Found { node: CommandNode::Commands(..) } => None,
      Self::NotFound { command, .. } => command
    }
  }
}

pub fn find_command<T: 'static, H: 'static>(
  args: &[String],
  commands: Commands<T, H>
) -> Result<&'static Command<T, H>, CommandError> {
  let (first_arg, args) = args.split_first()
    .ok_or_else(|| CommandError::InsufficientArgsCommand(command_names(commands)))?;
  let command = find_command_in_list(first_arg, commands)
    .ok_or_else(|| CommandError::CommandNotFound(first_arg.to_owned()))?;

  args.iter().try_fold(command, |command, arg| {
    match &command.body {
      CommandBody::Group { commands } => {
        find_command_in_list(arg, commands)
          .ok_or_else(|| CommandError::CommandNotFound(arg.to_owned()))
      },
      CommandBody::Target { .. } => {
        Err(CommandError::CommandNotFound(arg.to_owned()))
      }
    }
  })
}

pub fn find_command_in_list<T: 'static, H: 'static>(name: &str, commands: Commands<T, H>) -> Option<&'static Command<T, H>> {
  commands.iter().find_map(|command| command.name.eq_ignore_ascii_case(name).then_some(command))
}

fn find_target_args<T: 'static, H: 'static>(
  args: &mut VecDeque<String>,
  commands: Commands<T, H>
) -> Result<(&'static T, &'static H), CommandError> {
  let first_arg = args.pop_front()
    .ok_or_else(|| CommandError::InsufficientArgsCommand(command_names(commands)))?;
  let command = find_command_in_list(&first_arg, commands)
    .ok_or_else(|| CommandError::CommandNotFound(first_arg))?;

  match &command.body {
    CommandBody::Group { commands } => find_target_args(args, commands),
    CommandBody::Target { target } => Ok((target, &command.help))
  }
}

fn command_names<T, H>(commands: Commands<T, H>) -> Vec<&'static str> {
  commands.iter().map(|command| command.name).collect()
}

#[inline]
pub fn resolve_args<R: ResolveArgs>(args: &[String]) -> Result<R::Resolved, CommandError> {
  R::resolve_args(args)
}



#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parsed<T>(pub T);

impl<T> Parsed<T> {
  pub fn into_inner(self) -> T {
    let Parsed(inner) = self;
    inner
  }
}

pub trait ResolveArg: Sized {
  type Resolved;

  fn resolve_arg(arg: Option<&str>) -> Result<Self::Resolved, CommandError>;
}

impl<T> ResolveArg for Parsed<T>
where T: FromStr, T::Err: std::error::Error + Send + Sync + 'static {
  type Resolved = T;

  fn resolve_arg(arg: Option<&str>) -> Result<Self::Resolved, CommandError> {
    match arg {
      Some(arg) => arg.parse::<T>()
        .map_err(|err| CommandError::ArgsFromStrError(err.into())),
      None => Err(CommandError::InsufficientArgs(type_name::<T>()))
    }
  }
}

impl ResolveArg for String {
  type Resolved = String;

  fn resolve_arg(arg: Option<&str>) -> Result<Self::Resolved, CommandError> {
    arg.map(str::to_owned)
      .ok_or_else(|| CommandError::InsufficientArgs(type_name::<String>()))
  }
}

impl<T> ResolveArg for Option<T> where T: ResolveArg {
  type Resolved = Option<T::Resolved>;

  fn resolve_arg(arg: Option<&str>) -> Result<Self::Resolved, CommandError> {
    arg.map(|arg| T::resolve_arg(Some(arg))).transpose()
  }
}

pub trait ResolveArgs: Sized {
  type Resolved;

  fn resolve_args(args: &[String]) -> Result<Self::Resolved, CommandError>;
}

macro_rules! impl_resolve_args {
  ($($G:ident),* $(,)?) => {
    #[allow(unused)]
    impl<$($G: ResolveArg),*> ResolveArgs for ($($G,)*) {
      type Resolved = ($($G::Resolved,)*);

      fn resolve_args(args: &[String]) -> Result<Self::Resolved, CommandError> {
        let mut args = args.into_iter();
        Ok(($($G::resolve_arg(args.next().map(String::as_str))?,)*))
      }
    }
  };
}

impl_resolve_args!();
impl_resolve_args!(A);
impl_resolve_args!(A, B);
impl_resolve_args!(A, B, C);
impl_resolve_args!(A, B, C, D);
impl_resolve_args!(A, B, C, D, E);
impl_resolve_args!(A, B, C, D, E, F);
impl_resolve_args!(A, B, C, D, E, F, G);
impl_resolve_args!(A, B, C, D, E, F, G, H);
impl_resolve_args!(A, B, C, D, E, F, G, H, I);
impl_resolve_args!(A, B, C, D, E, F, G, H, I, J);
impl_resolve_args!(A, B, C, D, E, F, G, H, I, J, K);
impl_resolve_args!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_resolve_args!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_resolve_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_resolve_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_resolve_args!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

impl<T: ResolveArg> ResolveArgs for T {
  type Resolved = T::Resolved;

  fn resolve_args(args: &[String]) -> Result<Self::Resolved, CommandError> {
    T::resolve_arg(args.first().map(String::as_str))
  }
}



fn split_args(input: &str) -> Result<Vec<String>, CommandError> {
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  enum SplitArgsMode {
    Normal,
    QuoteDouble,
    QuoteSingle
  }

  fn accept_escape_char(
    input_iter: &mut impl Iterator<Item = char>,
    arg: &mut String
  ) -> Result<(), Option<char>> {
    match input_iter.next().ok_or(None)? {
      '\'' => arg.push('\''),
      '\"' => arg.push('\"'),
      '\\' => arg.push('\\'),
      '\n' => arg.push('\n'),
      '\r' => arg.push('\r'),
      '\t' => arg.push('\t'),
      '\0' => arg.push('\0'),
      ch => return Err(Some(ch))
    };

    Ok(())
  }

  fn is_legal_char(ch: char) -> bool {
    ('\x20'..'\x7f').contains(&ch) || ch.is_ascii_whitespace()
  }

  let mut mode = SplitArgsMode::Normal;
  let mut args = Vec::new();
  let mut arg = String::new();

  let mut input_iter = input.chars();
  while let Some(ch) = input_iter.next() {
    if !is_legal_char(ch) {
      return Err(CommandError::IllegalCharacter(ch));
    };

    match mode {
      SplitArgsMode::Normal => {
        if ch.is_ascii_whitespace() {
          let arg = std::mem::take(&mut arg);
          if !arg.is_empty() {
            args.push(arg);
          };
        } else {
          if arg.is_empty() {
            if ch == '\"' {
              mode = SplitArgsMode::QuoteDouble;
              continue;
            };

            if ch == '\'' {
              mode = SplitArgsMode::QuoteSingle;
              continue;
            };
          } else if ch == '\"' || ch == '\'' {
            return Err(CommandError::UnexpectedQuote);
          };

          arg.push(ch);
        };
      },
      SplitArgsMode::QuoteDouble => {
        if ch == '\\' {
          accept_escape_char(&mut input_iter, &mut arg).map_err(|ch| match ch {
            Some(ch) => CommandError::IllegalEscapeCodeSequence(ch),
            None => CommandError::UnexpectedEndOfLineInEscapeSequence
          })?;
        } else if ch == '\"' {
          mode = SplitArgsMode::Normal;
          args.push(std::mem::take(&mut arg));
        } else {
          arg.push(ch);
        };
      },
      SplitArgsMode::QuoteSingle => {
        if ch == '\\' {
          accept_escape_char(&mut input_iter, &mut arg).map_err(|ch| match ch {
            Some(ch) => CommandError::IllegalEscapeCodeSequence(ch),
            None => CommandError::UnexpectedEndOfLineInEscapeSequence
          })?;
        } else if ch == '\'' {
          mode = SplitArgsMode::Normal;
          args.push(std::mem::take(&mut arg));
        } else {
          arg.push(ch);
        };
      }
    };
  };

  if !arg.is_empty() {
    args.push(arg);
  };

  match mode {
    SplitArgsMode::Normal => Ok(args),
    SplitArgsMode::QuoteDouble => Err(CommandError::UnexpectedEndOfLineInQuote),
    SplitArgsMode::QuoteSingle => Err(CommandError::UnexpectedEndOfLineInQuote)
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_split_args() {
    const SAMPLES: &[(&'static str, Option<&[&str]>)] = &[
      ("", Some(&[])),
      ("arg1", Some(&["arg1"])),
      ("arg1 arg2", Some(&["arg1", "arg2"])),
      (" arg1  \t arg2 \r\n ", Some(&["arg1", "arg2"])),
      ("'arg1'", Some(&["arg1"])),
      ("'arg1' 'arg2'", Some(&["arg1", "arg2"])),
      ("'arg1' 'arg2' arg3", Some(&["arg1", "arg2", "arg3"])),
      ("arg1 'arg2'", Some(&["arg1", "arg2"])),
      ("arg1 'arg2' 'arg3'", Some(&["arg1", "arg2", "arg3"])),
      ("''", Some(&[""])),
      ("'' ''", Some(&["", ""])),
      (r"'\''", Some(&["\'"])),
      (r"'", None),
      (r"a'", None)
    ];

    for (i, &(sample_input, sample_output)) in SAMPLES.iter().enumerate() {
      let output = super::split_args(sample_input).ok();
      let sample_output = sample_output.map(|so| {
        so.into_iter().copied().map(str::to_owned).collect()
      });

      assert_eq!(sample_output, output, "{sample_input:?} ({i})");
    };
  }
}
