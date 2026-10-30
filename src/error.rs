use abyss::error::ContextualError;



pub type MelodyResult<T = ()> = Result<T, MelodyError>;

/// An error that can occur during operation of the bot.
#[derive(Debug, Error)]
pub enum MelodyError {
  #[error("File Error: {0}")]
  FileError(ContextualError<MelodyFileError>),
  #[error("Serenity Error: {0}")]
  SerenityError(#[from] ContextualError<serenity::prelude::SerenityError>),
  #[error("Command Error: {0}")]
  CommandError(#[from] MelodyCommandError),
  #[error("Input Error: {0}")]
  InputError(#[from] melody_commander::CommandError),
  #[error("YT-DLP Error: {0}")]
  YtDlpError(#[from] crate::utils::youtube::YtDlpError),
  #[error("Feed Model Error: {0}")]
  FeedModelError(#[from] feed_machine::model::ModelError<feed_machine::model::SchemaError>)
}

impl MelodyError {
  pub const COMMAND_NOT_IN_GUILD: Self = MelodyError::CommandError(MelodyCommandError::NOT_IN_GUILD);
  pub const COMMAND_PRECONDITION_VIOLATION_ARGUMENTS: Self = MelodyError::CommandError(MelodyCommandError::PRECONDITION_VIOLATION_ARGUMENTS);
  pub const COMMAND_PRECONDITION_VIOLATION_ROOT_COMMAND: Self = MelodyError::CommandError(MelodyCommandError::PRECONDITION_VIOLATION_ROOT_COMMAND);

  pub const fn command_cache_failure(message: &'static str) -> Self {
    MelodyError::CommandError(MelodyCommandError::cache_failure(message))
  }

  pub const fn command_precondition_violation(message: &'static str) -> Self {
    MelodyError::CommandError(MelodyCommandError::precondition_violation(message))
  }
}

impl<E> From<ContextualError<E>> for MelodyError
where E: Into<MelodyFileError> {
  fn from(error: ContextualError<E>) -> Self {
    MelodyError::FileError(error.map_error(E::into))
  }
}

impl<E> From<ContextualError<singlefile::OrUserError<E, MelodyError>>> for MelodyError
where E: Into<MelodyFileError> {
  fn from(error: ContextualError<singlefile::OrUserError<E, MelodyError>>) -> Self {
    let ContextualError { error, context } = error;

    match error {
      singlefile::OrUserError::Base(error) => {
        MelodyError::FileError(ContextualError::new(error.into(), context))
      },
      singlefile::OrUserError::User(error) => error
    }
  }
}

/// An error that can be caused by trying to read or parse a file.
#[derive(Debug, Error)]
pub enum MelodyFileError {
  #[error(transparent)]
  Io(#[from] std::io::Error),
  #[error(transparent)]
  Toml(#[from] singlefile::Error<singlefile_formats::data::toml_serde::TomlError>),
  #[error(transparent)]
  Json(#[from] singlefile::Error<singlefile_formats::data::json_serde::JsonError>),
  #[error(transparent)]
  Cbor(#[from] singlefile::Error<singlefile_formats::data::cbor_serde::CborError>),
  #[error(transparent)]
  CleverBotLog(#[from] cleverbot_logs::Error)
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MelodyCommandError {
  #[error("not in a guild")]
  NotInGuild,
  #[error("data not cached: {0}")]
  CacheFailure(&'static str),
  #[error("precondition violated: {0}")]
  PreconditionViolation(&'static str)
}

impl MelodyCommandError {
  pub const NOT_IN_GUILD: Self = MelodyCommandError::NotInGuild;
  pub const PRECONDITION_VIOLATION_ARGUMENTS: Self = MelodyCommandError::precondition_violation("arguments");
  pub const PRECONDITION_VIOLATION_ROOT_COMMAND: Self = MelodyCommandError::precondition_violation("root command");

  pub const fn cache_failure(message: &'static str) -> Self {
    Self::CacheFailure(message)
  }

  pub const fn precondition_violation(message: &'static str) -> Self {
    Self::PreconditionViolation(message)
  }
}
