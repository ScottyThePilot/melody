#[macro_use]
extern crate thiserror;

use std::collections::VecDeque;
use std::str::FromStr;



#[derive(Debug, Clone, Copy)]
pub struct Command<T: 'static> {
  pub name: &'static str,
  pub body: CommandBody<T>
}

impl<T: 'static> Command<T> {
  pub const fn new_group(name: &'static str, commands: &'static [Command<T>]) -> Self {
    Command { name, body: CommandBody::Group { commands } }
  }

  pub const fn new_target(name: &'static str, target: T) -> Self {
    Command { name, body: CommandBody::Target { target } }
  }
}

#[derive(Debug, Clone, Copy)]
pub enum CommandBody<T: 'static> {
  Group {
    commands: &'static [Command<T>]
  },
  Target {
    target: T
  }
}

#[derive(Debug, Clone)]
pub struct CommandOutput<T: 'static> {
  pub target: &'static T,
  pub remaining_args: Box<[String]>
}

#[derive(Debug, Error)]
pub enum CommandError {
  #[error("empty command group")]
  EmptyCommandGroup,
  #[error("insufficient arguments")]
  InsufficientArgs,
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

pub fn apply<T: 'static>(input: &str, commands: &'static [Command<T>]) -> Result<CommandOutput<T>, CommandError> {
  split_args(input).and_then(|args| {
    let mut args = VecDeque::from(args);
    let target = find_target_args(&mut args, commands)?;
    let remaining_args = Vec::from(args).into_boxed_slice();
    Ok(CommandOutput { target, remaining_args })
  })
}

fn find_target_args<T: 'static>(
  args: &mut VecDeque<String>,
  commands: &'static [Command<T>]
) -> Result<&'static T, CommandError> {
  let first_arg = args.pop_front().ok_or(CommandError::InsufficientArgs)?;
  let body = commands.iter()
    .find_map(|command| command.name.eq_ignore_ascii_case(&first_arg).then_some(&command.body))
    .ok_or_else(|| CommandError::CommandNotFound(first_arg))?;

  match body {
    CommandBody::Group { commands } => find_target_args(args, commands),
    CommandBody::Target { target } => Ok(target)
  }
}

#[inline]
pub fn resolve_args<R: ResolveArgs>(args: &[String]) -> Result<R, CommandError> {
  R::resolve_args(args)
}



#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Parsed<T>(pub T);

pub trait ResolveArg: Sized {
  fn resolve_arg(arg: Option<&str>) -> Result<Self, CommandError>;
}

impl<T> ResolveArg for Parsed<T>
where T: FromStr, T::Err: std::error::Error + Send + Sync + 'static {
  fn resolve_arg(arg: Option<&str>) -> Result<Self, CommandError> {
    match arg {
      Some(arg) => arg.parse::<T>().map(Parsed)
        .map_err(|err| CommandError::ArgsFromStrError(err.into())),
      None => Err(CommandError::InsufficientArgs)
    }
  }
}

impl ResolveArg for String {
  fn resolve_arg(arg: Option<&str>) -> Result<Self, CommandError> {
    arg.map(str::to_owned).ok_or(CommandError::InsufficientArgs)
  }
}

impl<T> ResolveArg for Option<T> where T: ResolveArg {
  fn resolve_arg(arg: Option<&str>) -> Result<Self, CommandError> {
    arg.map(|arg| T::resolve_arg(Some(arg))).transpose()
  }
}

pub trait ResolveArgs: Sized {
  fn resolve_args(args: &[String]) -> Result<Self, CommandError>;
}

macro_rules! impl_resolve_args {
  ($($G:ident),* $(,)?) => {
    impl<$($G: ResolveArg),*> ResolveArgs for ($($G,)*) {
      fn resolve_args(args: &[String]) -> Result<Self, CommandError> {
        let mut args = args.into_iter();
        Ok(($($G::resolve_arg(args.next().map(String::as_str))?,)*))
      }
    }
  };
}

impl_resolve_args!(A, B);
impl_resolve_args!(A, B, C);
impl_resolve_args!(A, B, C, D);
impl_resolve_args!(A, B, C, D, E);
impl_resolve_args!(A, B, C, D, E, F);
impl_resolve_args!(A, B, C, D, E, F, G);
impl_resolve_args!(A, B, C, D, E, F, G, H);

impl<T: ResolveArg> ResolveArgs for T {
  fn resolve_args(args: &[String]) -> Result<Self, CommandError> {
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
