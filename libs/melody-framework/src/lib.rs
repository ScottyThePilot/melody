#![warn(
  absolute_paths_not_starting_with_crate,
  redundant_imports,
  redundant_lifetimes,
  future_incompatible,
  deprecated_in_future,
  missing_copy_implementations,
  missing_debug_implementations,
  unnameable_types,
  unreachable_pub
)]

pub mod commands;
pub mod handler;

extern crate itertools;
extern crate poise;
extern crate serenity;
extern crate thiserror;
extern crate tokio;
#[macro_use]
extern crate tracing;

use crate::handler::{MelodyHandler, MelodyHandlerFull};

pub use poise::BoxFuture;

use poise::reply::CreateReply;
use poise::framework::Framework as PoiseFramework;
use poise::structs::{
  Context as PoiseContext,
  Command as PoiseCommand,
  FrameworkError as PoiseFrameworkError,
  FrameworkOptions as PoiseFrameworkOptions
};
use serenity::Error as SerenityError;
use serenity::builder::CreateAllowedMentions;
use serenity::model::Permissions;
use serenity::model::id::{GuildId, UserId};
use serenity::gateway::ShardManager;
use serenity::cache::Cache;
use serenity::http::{CacheHttp, Http};
use serenity::client::{Client, Context, FullEvent};
use serenity::framework::Framework;
use thiserror::Error;
use tokio::sync::RwLock;

use std::fmt;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;



macro_rules! builder_field {
  ($vis:vis $field:ident $function:ident(): $Type:ty) => (
    $vis fn $function(mut self, $field: $Type) -> Self {
      self.$field = $field;
      self
    }
  );
}

pub struct MelodyFrameworkOptions<S, E> {
  pub state: Arc<S>,
  /// User IDs which are allowed to use `owners_only`` commands.
  pub owners: HashSet<UserId>,
  /// List of commands in the framework.
  pub commands: Vec<MelodyCommand<S, E>>,
  /// Called on every Discord event. Can be used to react to non-command events, like message deletions or guild updates.
  pub handler: Arc<dyn MelodyHandlerFull<S, E>>,
  /// Default set of allowed mentions to use for all responses.
  ///
  /// By default, user pings are allowed and role pings and everyone pings are filtered.
  pub allowed_mentions: Option<CreateAllowedMentions>,
  /// If `true`, disables automatic cooldown handling before every command invocation.
  ///
  /// Useful for implementing custom cooldown behavior. See [`poise::Command::cooldowns`] and
  /// the methods on [`poise::Cooldowns`] for how to do that.
  pub manual_cooldowns: bool,
  /// If `true`, changes behavior of guild_only command check to abort execution if the guild is not in cache.
  pub require_cache_for_guild_check: bool,
  /// If true, [`Self::owners`] is automatically initialized with the results of [`serenity::Http::get_current_application_info()`].
  ///
  /// True by default.
  pub initialize_owners: bool
}

impl<S, E> MelodyFrameworkOptions<S, E>
where
  S: Send + Sync + 'static,
  E: Send + Sync + fmt::Debug + fmt::Display + 'static,
{
  pub fn new(state: Arc<S>, handler: Arc<dyn MelodyHandlerFull<S, E>>) -> Self {
    let allowed_mentions = CreateAllowedMentions::default()
      .all_users(true).replied_user(true);
    MelodyFrameworkOptions {
      state,
      owners: HashSet::new(),
      commands: Vec::new(),
      handler,
      allowed_mentions: Some(allowed_mentions),
      manual_cooldowns: false,
      require_cache_for_guild_check: false,
      initialize_owners: true
    }
  }

  builder_field!(pub owners with_owners(): HashSet<UserId>);
  builder_field!(pub commands with_commands(): Vec<MelodyCommand<S, E>>);
  builder_field!(pub allowed_mentions with_allowed_mentions(): Option<CreateAllowedMentions>);
  builder_field!(pub manual_cooldowns with_manual_cooldowns(): bool);
  builder_field!(pub require_cache_for_guild_check with_require_cache_for_guild_check(): bool);
  builder_field!(pub initialize_owners with_initialize_owners(): bool);

  pub fn build(self) -> MelodyFramework<S, E> {
    MelodyFramework::new(self)
  }

  fn build_inner(self) -> MelodyFrameworkInner<S, E> {
    let data = MelodyFrameworkData {
      state: self.state.clone(),
      handler: self.handler.clone()
    };

    let framework = PoiseFramework::builder()
      .options(PoiseFrameworkOptions {
        commands: self.commands,
        on_error: crate::on_error,
        pre_command: crate::pre_command,
        post_command: crate::post_command,
        command_check: Some(crate::command_check),
        allowed_mentions: self.allowed_mentions,
        reply_callback: Some(crate::reply_callback),
        manual_cooldowns: self.manual_cooldowns,
        require_cache_for_guild_check: self.require_cache_for_guild_check,
        owners: self.owners,
        initialize_owners: self.initialize_owners,
        ..PoiseFrameworkOptions::default()
      })
      .setup(move |_ctx, _ready, _framework| {
        Box::pin(std::future::ready(Ok(data)))
      })
      .initialize_owners(self.initialize_owners)
      .build();

    MelodyFrameworkInner {
      state: self.state,
      handler: self.handler,
      framework
    }
  }
}

impl<S: fmt::Debug, E> fmt::Debug for MelodyFrameworkOptions<S, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MelodyFrameworkOptions")
      .field("state", &self.state)
      .field("owners", &self.owners)
      .field("commands", &self.commands)
      .field("handler", &format_args!(".."))
      .field("allowed_mentions", &self.allowed_mentions)
      .field("manual_cooldowns", &self.manual_cooldowns)
      .field("require_cache_for_guild_check", &self.require_cache_for_guild_check)
      .field("initialize_owners", &self.initialize_owners)
      .finish()
  }
}

pub struct MelodyFrameworkData<S, E> {
  state: Arc<S>,
  handler: Arc<dyn MelodyHandlerFull<S, E>>
}

impl<S, E> MelodyFrameworkData<S, E> {
  pub fn into_state(self) -> Arc<S> {
    self.state
  }
}

impl<S, E> Clone for MelodyFrameworkData<S, E> {
  fn clone(&self) -> Self {
    MelodyFrameworkData {
      state: self.state.clone(),
      handler: self.handler.clone()
    }
  }
}

impl<S, E> Deref for MelodyFrameworkData<S, E> {
  type Target = Arc<S>;

  fn deref(&self) -> &Self::Target {
    &self.state
  }
}

impl<S: fmt::Debug, E> fmt::Debug for MelodyFrameworkData<S, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MelodyFrameworkData")
      .field("state", &self.state)
      .field("handler", &format_args!(".."))
      .finish()
  }
}

pub type MelodyCommand<S, E> = PoiseCommand<MelodyFrameworkData<S, E>, E>;
pub type MelodyContext<'a, S, E> = PoiseContext<'a, MelodyFrameworkData<S, E>, E>;

#[derive(Debug)]
pub struct MelodyHandlerContext<'a, S, E> {
  pub context: Context,
  pub state: Arc<S>,
  pub commands: &'a Vec<MelodyCommand<S, E>>,
  pub shard_manager: &'a Arc<ShardManager>
}

impl<'a, S, E> MelodyHandlerContext<'a, S, E> {
  pub async fn register_guild_commands(&self, guild_id: GuildId, categories: HashSet<String>) -> Result<(), SerenityError> {
    crate::commands::register_guild_commands(&self.context, self.commands, guild_id, categories).await
  }

  pub async fn register_commands(&self, guilds: impl IntoIterator<Item = (GuildId, HashSet<String>)>) -> Result<(), SerenityError> {
    crate::commands::register_commands(&self.context, self.commands, guilds).await
  }
}

impl<'a, S, E> CacheHttp for MelodyHandlerContext<'a, S, E>
where S: Send + Sync, E: Send + Sync {
  fn cache(&self) -> Option<&Arc<Cache>> {
    Some(&self.context.cache)
  }

  fn http(&self) -> &Http {
    &self.context.http
  }
}

impl<'a, S, E> AsRef<Cache> for MelodyHandlerContext<'a, S, E> {
  fn as_ref(&self) -> &Cache {
    &self.context.cache
  }
}

impl<'a, S, E> AsRef<Http> for MelodyHandlerContext<'a, S, E> {
  fn as_ref(&self) -> &Http {
    &self.context.http
  }
}

impl<'a, S, E> Clone for MelodyHandlerContext<'a, S, E> {
  fn clone(&self) -> Self {
    MelodyHandlerContext {
      context: self.context.clone(),
      state: self.state.clone(),
      commands: self.commands,
      shard_manager: self.shard_manager
    }
  }
}

type MelodyFrameworkInnerLock<S, E> = tokio::sync::OwnedRwLockReadGuard<MelodyFrameworkInner<S, E>>;
type MelodyFrameworkInnerLockMapped<S, E, T> = tokio::sync::OwnedRwLockReadGuard<MelodyFrameworkInner<S, E>, T>;

#[derive(Debug)]
pub struct MelodyFrameworkCommandsGuard<S, E> {
  guard: MelodyFrameworkInnerLockMapped<S, E, Vec<MelodyCommand<S, E>>>
}

impl<S, E> Deref for MelodyFrameworkCommandsGuard<S, E> {
  type Target = Vec<MelodyCommand<S, E>>;

  fn deref(&self) -> &Self::Target {
    &*self.guard
  }
}

pub struct MelodyFramework<S, E> {
  inner: Arc<RwLock<MelodyFrameworkInner<S, E>>>
}

impl<S, E> MelodyFramework<S, E>
where S: Send + Sync, E: Send + Sync {
  pub fn new(options: MelodyFrameworkOptions<S, E>) -> Self
  where S: 'static, E: fmt::Debug + fmt::Display + 'static {
    MelodyFramework { inner: Arc::new(RwLock::new(options.build_inner())) }
  }

  async fn read_inner_owned(self) -> MelodyFrameworkInnerLock<S, E> {
    self.inner.read_owned().await
  }

  pub async fn read_commands_owned(self) -> MelodyFrameworkCommandsGuard<S, E> {
    MelodyFrameworkCommandsGuard {
      guard: tokio::sync::OwnedRwLockReadGuard::map(
        self.read_inner_owned().await,
        |inner| &inner.framework.options().commands
      )
    }
  }
}

impl<S, E> Clone for MelodyFramework<S, E> {
  fn clone(&self) -> Self {
    MelodyFramework { inner: Arc::clone(&self.inner) }
  }
}

#[serenity::async_trait]
impl<S, E> Framework for MelodyFramework<S, E>
where S: Send + Sync, E: Send + Sync {
  async fn init(&mut self, client: &Client) {
    self.inner.write().await.init(client).await;
  }

  async fn dispatch(&self, ctx: Context, event: FullEvent) {
    self.inner.read().await.dispatch(ctx, event).await;
  }
}

impl<S, E> fmt::Debug for MelodyFramework<S, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MelodyFramework").finish_non_exhaustive()
  }
}

struct MelodyFrameworkInner<S, E> {
  state: Arc<S>,
  handler: Arc<dyn MelodyHandler<S, E>>,
  framework: PoiseFramework<MelodyFrameworkData<S, E>, E>
}

impl<S, E> fmt::Debug for MelodyFrameworkInner<S, E> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("MelodyFrameworkInner").finish_non_exhaustive()
  }
}

#[serenity::async_trait]
impl<S, E> Framework for MelodyFrameworkInner<S, E>
where S: Send + Sync, E: Send + Sync {
  async fn init(&mut self, client: &Client) {
    self.framework.init(client).await;
  }

  async fn dispatch(&self, ctx: Context, event: FullEvent) {
    // Since we are completely bypassing Poise's event handler system,
    // and we do not support prefix commands, the only two events of any use to
    // our captive Poise framework are the "Ready" and "InteractionCreate" events.
    if matches!(event, FullEvent::Ready { .. } | FullEvent::InteractionCreate { .. }) {
      self.framework.dispatch(ctx.clone(), event.clone()).await;
    };

    let handler_context = MelodyHandlerContext {
      context: ctx,
      state: self.state.clone(),
      commands: &self.framework.options().commands,
      shard_manager: &self.framework.shard_manager()
    };

    crate::handler::dispatch(event, &*self.handler, handler_context).await
  }
}

fn reply_callback<S, E>(ctx: MelodyContext<'_, S, E>, create_reply: CreateReply) -> CreateReply
where S: Send + Sync, E: Send + Sync {
  ctx.data().handler.outgoing_reply(ctx, create_reply)
}

fn pre_command<S, E>(ctx: MelodyContext<'_, S, E>) -> BoxFuture<'_, ()>
where S: Send + Sync, E: Send + Sync {
  ctx.data().handler.pre_command(ctx)
}

fn post_command<S, E>(ctx: MelodyContext<'_, S, E>) -> BoxFuture<'_, ()>
where S: Send + Sync, E: Send + Sync {
  ctx.data().handler.post_command(ctx)
}

fn command_check<S, E>(ctx: MelodyContext<'_, S, E>) -> BoxFuture<'_, Result<bool, E>>
where S: Send + Sync, E: Send + Sync {
  ctx.data().handler.command_predicate(ctx)
}

fn on_error<S, E>(framework_error: PoiseFrameworkError<'_, MelodyFrameworkData<S, E>, E>) -> BoxFuture<'_, ()>
where S: Send + Sync, E: Send + Sync + fmt::Debug + fmt::Display {
  Box::pin(async move {
    match on_error_full(framework_error).await {
      Ok(Some((ctx, framework_error))) => {
        ctx.data().handler.command_error(ctx, framework_error).await;
      },
      Ok(None) => (),
      Err(error) => {
        error!("{error}");
      }
    };
  })
}

async fn on_error_full<S, E>(
  framework_error: PoiseFrameworkError<'_, MelodyFrameworkData<S, E>, E>
) -> Result<Option<(MelodyContext<'_, S, E>, MelodyFrameworkError<E>)>, SerenityError>
where S: Send + Sync, E: Send + Sync + fmt::Debug + fmt::Display {
  let result = match framework_error {
    PoiseFrameworkError::Setup { .. } => unreachable!(),
    PoiseFrameworkError::EventHandler { .. } => unreachable!(),
    PoiseFrameworkError::Command { error, ctx, .. } => {
      (ctx, MelodyFrameworkError::Command(error))
    },
    PoiseFrameworkError::SubcommandRequired { .. } => unreachable!(),
    PoiseFrameworkError::CommandPanic { payload, ctx, .. } => {
      (ctx, MelodyFrameworkError::CommandPanic(payload))
    },
    PoiseFrameworkError::ArgumentParse { error, input, ctx, .. } => {
      (ctx, MelodyFrameworkError::ArgumentParse(input, error))
    },
    PoiseFrameworkError::CommandStructureMismatch { description, ctx, .. } => {
      let ctx = PoiseContext::Application(ctx);
      (ctx, MelodyFrameworkError::CommandStructureMismatch(description))
    },
    PoiseFrameworkError::CooldownHit { remaining_cooldown, ctx, .. } => {
      (ctx, MelodyFrameworkError::CooldownHit(remaining_cooldown))
    },
    PoiseFrameworkError::MissingBotPermissions { missing_permissions, ctx, .. } => {
      (ctx, MelodyFrameworkError::MissingBotPermissions(missing_permissions))
    },
    PoiseFrameworkError::MissingUserPermissions { missing_permissions, ctx, .. } => {
      (ctx, MelodyFrameworkError::MissingUserPermissions(missing_permissions))
    },
    PoiseFrameworkError::NotAnOwner { ctx, .. } => {
      (ctx, MelodyFrameworkError::NotAnOwner)
    },
    PoiseFrameworkError::GuildOnly { ctx, .. } => {
      (ctx, MelodyFrameworkError::GuildOnly)
    },
    PoiseFrameworkError::DmOnly { ctx, .. } => {
      (ctx, MelodyFrameworkError::DmOnly)
    },
    PoiseFrameworkError::NsfwOnly { ctx, .. } => {
      (ctx, MelodyFrameworkError::NsfwOnly)
    },
    PoiseFrameworkError::CommandCheckFailed { .. } => unreachable!(),
    PoiseFrameworkError::DynamicPrefix { .. } => unreachable!(),
    PoiseFrameworkError::UnknownCommand { .. } => unreachable!(),
    framework_error => {
      return poise::builtins::on_error(framework_error).await.map(|()| None);
    }
  };

  Ok(Some(result))
}

#[derive(Debug, Error)]
pub enum MelodyFrameworkError<E> {
  #[error("failed to execute command: {0}")]
  Command(E),
  #[error("panicked encountered while executing a command (payload: {0:?})")]
  CommandPanic(Option<String>),
  #[error("application command structure mismatch: {0}")]
  CommandStructureMismatch(&'static str),
  #[error("failed to parse argument {0:?}: {1}")]
  ArgumentParse(Option<String>, Box<dyn std::error::Error + Send + Sync + 'static>),
  #[error("refused to serve a command (user must wait {:.2} seconds)", .0.as_secs_f32())]
  CooldownHit(std::time::Duration),
  #[error("refused to serve a command, bot missing required permissions")]
  MissingBotPermissions(Permissions),
  #[error("refused to serve a command, user missing required permissions")]
  MissingUserPermissions(Option<Permissions>),
  #[error("refused to serve a command, user is not a bot owner")]
  NotAnOwner,
  #[error("refused to serve a command, command may only be executed in guilds")]
  GuildOnly,
  #[error("refused to serve a command, command may only be executed in dms")]
  DmOnly,
  #[error("refused to serve a command, command may only be executed in nsfw channels")]
  NsfwOnly
}
