//! Items and functions associated with launching the bot and handling discord events
mod input;

use crate::prelude::*;
use crate::data::*;
pub use self::input::InputAgent;

use melody_flag::Flag;
use melody_framework::handler::{MelodyHandler, MelodyHandlerFull};
use rand::seq::IndexedRandom;
use reqwest::Client as HttpClient;
use serenity::cache::Cache;
use serenity::client::Client;
use serenity::gateway::ShardManager;
use serenity::model::channel::{Reaction, ReactionType, Message};
use serenity::model::gateway::Ready;
use serenity::model::guild::{Guild, UnavailableGuild};
use serenity::model::guild::Member;
use serenity::model::id::{GuildId, UserId, RoleId};
use serenity::utils::{content_safe, ContentSafeOptions};
use songbird::{SerenityInit, Config as SongbirdConfig};
use term_stratum::StratumEvent;
use tokio::sync::mpsc::UnboundedReceiver as MpscReceiver;
use tokio::time::MissedTickBehavior;

use std::sync::Arc;
use std::time::Duration;

pub type MelodyCommand = melody_framework::MelodyCommand<State, MelodyError>;
pub type MelodyContext<'a> = melody_framework::MelodyContext<'a, State, MelodyError>;
pub type MelodyFramework = melody_framework::MelodyFramework<State, MelodyError>;
pub type MelodyFrameworkError = melody_framework::MelodyFrameworkError<MelodyError>;
pub type MelodyFrameworkOptions = melody_framework::MelodyFrameworkOptions<State, MelodyError>;
pub type MelodyHandlerContext<'a> = melody_framework::MelodyHandlerContext<'a, State, MelodyError>;



/// Performs a clean launch of the bot
pub async fn launch(event_receiver: MpscReceiver<StratumEvent>) -> MelodyResult {
  let config = Config::create().await?;
  let persist = Persist::create().await?;
  let persist_guilds = PersistGuildsWrapper::from(PersistGuilds::create().await?);
  let activities = Activities::create().await?;

  let (token, intents) = config.operate(async |config| {
    (config.token.clone(), config.intents)
  }).await;

  let http_client = HttpClient::new();
  let state = Arc::new(State::new(
    config, persist, persist_guilds,
    activities, http_client
  ).await?);

  let handler = Arc::new(Handler { setup_done: Flag::new(false) });
  let framework = MelodyFrameworkOptions::new(state.clone(), handler)
    .with_commands(crate::commands::create_commands_list())
    .build();

  let mut client = Client::builder(&token, intents)
    .framework(framework.clone())
    .type_map_insert::<MelodyFrameworkKey>(framework)
    .register_songbird_from_config(SongbirdConfig::default())
    .await.context("failed to init client")?;
  client.data.write().await
    .insert::<ShardManagerKey>(client.shard_manager.clone());

  let core = Core::new(&client, state);
  let events_task = tokio::spawn(events_task(
    core.clone(), client.shard_manager.clone(), event_receiver
  ));

  client.start().await.context("failed to start client")?;

  core.abort().await;
  events_task.abort();
  client.data.write().await.clear();

  Ok(())
}

#[derive(Debug)]
struct Handler {
  setup_done: Flag
}

#[serenity::async_trait]
impl MelodyHandler<State, MelodyError> for Handler {
  async fn ready(&self, _ctx: MelodyHandlerContext<'_>, ready_info: Ready) {
    info!("Bot connected: {} ({})", ready_info.user.tag(), ready_info.user.id);
  }

  async fn cache_ready(&self, ctx: MelodyHandlerContext<'_>, guilds: Vec<GuildId>) {
    if self.setup_done.swap(true) { return };
    let core = Core::from(&ctx);

    if core.is_new_build() {
      info!("New build detected, registering commands");
      let guilds = core.operate_persist(async |persist| {
        guilds.iter()
          .map(|&guild_id| (guild_id, persist.get_guild_plugins(guild_id)))
          .collect::<Vec<(GuildId, HashSet<String>)>>()
      }).await;

      ctx.register_commands(guilds).await
        .context("failed to register commands").log_error();
    } else {
      info!("Old build detected, commands will not be re-registered");
    };

    // Attempt to register all subscribed RSS feeds
    core.feed().await.spawn_feeds_from_persist().await.log_error();

    core.operate_tasks(|tasks| {
      // Spawn the task for cycling activity status unless it's already been spawned
      tasks.cycle_activities.get_or_insert_with(|| {
        tokio::spawn(cycle_activity_task(core.clone()))
      });
    }).await;
  }

  async fn guild_create(&self, _ctx: MelodyHandlerContext<'_>, guild: Guild, is_new: Option<bool>) {
    let reason = match is_new {
      Some(true) => "added to guild",
      Some(false) => "populate",
      None => "create"
    };

    info!("Guild discovered: {} ({}) - {}", guild.name, guild.id, reason);
  }

  async fn guild_delete(&self, ctx: MelodyHandlerContext<'_>, incomplete: UnavailableGuild, guild_full_cached: Option<Guild>) {
    let guild_name = guild_full_cached.as_ref().map_or("Unknown", |guild| guild.name.as_str());
    let reason = match incomplete.unavailable {
      true => "outage",
      false => "removed from guild"
    };

    info!("Guild lost: {} ({}) - {}", guild_name, incomplete.id, reason);

    let core = Core::from(ctx);
    if !incomplete.unavailable {
      info!("Unregistering feeds for guild: {} ({})", guild_name, incomplete.id);
      core.feed().await.unregister_guild_feeds(incomplete.id).await.log_error();
    };
  }

  async fn message(&self, ctx: MelodyHandlerContext<'_>, message: Message) {
    let core = Core::from(ctx);

    let me = core.current_user_id();
    if message.author.id == me || message.content.is_empty() { return };

    if !message.author.bot {
      if let Some(guild_id) = message.guild_id {
        let emojis = crate::utils::parse_emojis(&message.content);
        core.operate_persist_guild_commit(guild_id, async |persist_guild| {
          // don't be greedy
          let mut rng = rand::rng();
          if let Some(&emoji) = emojis.choose(&mut rng) {
            persist_guild.emoji_stats.increment_emoji_uses(emoji, message.author.id);
          };
          Ok(())
        }).await.log_error();
      };

      if core.state.message_chains.operate_mut(async |message_chains| message_chains.should_contribute(&message)).await {
        message.channel_id.say(&core, &message.content).await.context("failed to send message").log_error();
      };
    };

    if is_mentioning_user(&message, me) {
      let content = clean_message_for_cleverbot(&core, &message.content, me);

      info!("Sending message to cleverbot: {content:?}");
      let typing = message.channel_id.start_typing(&core.http);
      match core.state.cleverbot.send(message.channel_id, &content).await {
        Ok(reply) => {
          info!("Recieved reply from cleverbot: {reply:?}");
          let options = ContentSafeOptions::new().show_discriminator(false);
          let reply = content_safe(&core, reply, &options, &[]);
          crate::feature::cleverbot::send_reply(&core, &message, &reply).await.log_error();
          core.state.cleverbot_logger.clone()
            .log(message.channel_id, content, reply).await.log_error();
        },
        Err(error) => {
          error!("Unable to get reply from cleverbot: {error}");
          message.reply(&core, "There was an error getting a reply from cleverbot").await
            .context("failed to send cleverbot failure message")
            .log_error();
        }
      };

      typing.stop();
    };
  }

  async fn guild_member_addition(&self, ctx: MelodyHandlerContext<'_>, mut member: Member) {
    let core = Core::from(ctx);

    add_join_roles(&core, &mut member).await.log_error();
  }

  async fn reaction_add(&self, ctx: MelodyHandlerContext<'_>, reaction: Reaction) {
    let core = Core::from(ctx);

    if let (Some(guild_id), Some(user_id), ReactionType::Custom { id: emoji_id, .. }) = (reaction.guild_id, reaction.user_id, reaction.emoji) {
      core.operate_persist_guild_commit(guild_id, async |persist_guild| {
        persist_guild.emoji_stats.increment_emoji_uses(emoji_id, user_id);
        Ok(())
      }).await.log_error();
    };
  }

  async fn reaction_remove(&self, ctx: MelodyHandlerContext<'_>, reaction: Reaction) {
    let core = Core::from(ctx);

    if let (Some(guild_id), Some(user_id), ReactionType::Custom { id: emoji_id, .. }) = (reaction.guild_id, reaction.user_id, reaction.emoji) {
      core.operate_persist_guild_commit(guild_id, async |persist_guild| {
        persist_guild.emoji_stats.decrement_emoji_uses(emoji_id, user_id);
        Ok(())
      }).await.log_error();
    };
  }
}

#[serenity::async_trait]
impl MelodyHandlerFull<State, MelodyError> for Handler {
  async fn command_error(&self, ctx: MelodyContext<'_>, error: MelodyFrameworkError) {
    error!("command error ({}): {error}", ctx.command().name);

    let response = framework_error_friendly_name(error);
    ctx.reply(response).await
      .context("failed to send error handler reply")
      .log_error();
  }
}

/// Gets the remaining content of a message if it either replies to the
/// given user, or begins with a mention of the given user.
fn is_mentioning_user(message: &Message, who: UserId) -> bool {
  message.referenced_message.as_ref()
    .is_some_and(|referenced_message| referenced_message.author.id == who)
    || message.mentions_user_id(who)
}

fn clean_message_for_cleverbot(cache: impl AsRef<Cache>, content: &str, me: UserId) -> String {
  use crate::feature::cleverbot::CLEVERBOT_CANONICAL_NAME;

  let content = crate::utils::replace_user_mentions(content.trim(), |user_id, m| {
    // only replace when mentions target the current user
    // replace with empty when it's at the start (prefix-ping)
    // replace with the 'cleverbot canonical name' otherwise
    (user_id == me).then(|| if m.start() == 0 { "" } else { CLEVERBOT_CANONICAL_NAME })
  });

  let options = ContentSafeOptions::new().show_discriminator(false);
  let content = content_safe(cache, content.trim(), &options, &[]);

  content
}

async fn add_join_roles(core: &Core, member: &mut Member) -> MelodyResult {
  let (roles, missing_roles) = core.operate_persist_guild(member.guild_id, async |persist_guild| {
    core.cache.guild(member.guild_id).map(|guild| {
      persist_guild.join_roles.iter()
        .filter_map(|(&role_id, &filter)| filter.applies(member.user.bot).then_some(role_id))
        .partition::<Vec<RoleId>, _>(|role_id| guild.roles.contains_key(&role_id))
    }).ok_or(MelodyError::command_cache_failure("guild"))
  }).await?;

  if !roles.is_empty() {
    member.add_roles(core, &roles).await
      .context("failed to grant user join roles")?;
    for role_id in roles.iter() {
      info!("Granted join role ({}) to user {} ({})", member.user.name, member.user.id, role_id);
    };
  };

  if !missing_roles.is_empty() {
    core.operate_persist_guild_commit(member.guild_id, async |persist_guild| {
      for role_id in missing_roles {
        persist_guild.join_roles.remove(&role_id);
        warn!("Removed non-existent role ({}) for guild ({})", role_id, member.guild_id);
      };

      Ok(())
    }).await?;
  };

  Ok(())
}

async fn events_task(
  core: Core,
  shard_manager: Arc<ShardManager>,
  mut event_receiver: MpscReceiver<StratumEvent>
) {
  let mut input_agent = InputAgent::new(core);
  while let Some(event) = event_receiver.recv().await {
    match event {
      StratumEvent::Input(line) => {
        let result = input_agent.line(line).await;
        if let Err(err) = result {
          input_agent.error(err.to_string());
        };
      },
      StratumEvent::Terminate => {
        shard_manager.shutdown_all().await;
      }
    };
  };
}

async fn cycle_activity_task(core: Core) {
  const ACTIVITY_CYCLE_TIME: Duration = Duration::from_secs(120);

  let mut interval = tokio::time::interval(ACTIVITY_CYCLE_TIME);
  interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

  loop {
    interval.tick().await;
    core.randomize_activities().await;
  };
}

fn framework_error_friendly_name(framework_error: MelodyFrameworkError) -> String {
  match framework_error {
    MelodyFrameworkError::Command(..) => {
      "An unexpected internal error has occurred".to_owned()
    },
    MelodyFrameworkError::CommandPanic(..) => {
      "An unexpected internal error has occurred".to_owned()
    },
    MelodyFrameworkError::CommandStructureMismatch(..) => {
      "An unexpected internal error has occurred".to_owned()
    },
    MelodyFrameworkError::ArgumentParse(..) => {
      "An unexpected internal error has occurred".to_owned()
    },
    MelodyFrameworkError::CooldownHit(remaining_cooldown) => {
      format!("Please wait {:.2} seconds before using that action again", remaining_cooldown.as_secs_f64())
    },
    MelodyFrameworkError::MissingBotPermissions(missing_permissions) => {
      format!("I am missing '{missing_permissions}' permissions")
    },
    MelodyFrameworkError::MissingUserPermissions(missing_permissions) => {
      if let Some(missing_permissions) = missing_permissions {
        format!("You do not have permission to do that (missing '{missing_permissions}' permissions)")
      } else {
        format!("You do not have permission to do that")
      }
    },
    MelodyFrameworkError::NotAnOwner => {
      "You do not have permission to do that (you are not a bot owner)".to_owned()
    },
    MelodyFrameworkError::GuildOnly => {
      "This command cannot be used in DM channels".to_owned()
    },
    MelodyFrameworkError::DmOnly => {
      "This command cannot be used in guild channels".to_owned()
    },
    MelodyFrameworkError::NsfwOnly => {
      "This command cannot be used in NSFW channels".to_owned()
    }
  }
}
