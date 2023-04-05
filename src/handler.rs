//! Items and functions associated with launching the bot and handling discord events
mod input;

use crate::MelodyResult;
use crate::commands::APPLICATION_COMMANDS;
use crate::data::*;
use crate::feature::cleverbot::CleverBotManager;
use crate::feature::message_chains::MessageChains;
use crate::terminal::Flag;
use crate::utils::{Contextualize, Loggable};

use rand::seq::SliceRandom;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::client::{Context, Client, EventHandler};
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::gateway::{Activity, GatewayIntents, Ready};
use serenity::model::id::{GuildId, UserId};
use serenity::utils::{content_safe, ContentSafeOptions};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver as MpscReceiver;
use tokio::sync::oneshot::Receiver as OneshotReceiver;
use tokio::time::MissedTickBehavior;

use std::sync::Arc;
use std::time::Duration;



/// Performs a clean launch of the bot, returning true if the bot expects to be restarted, and false if not.
pub async fn launch(
  terminate: Arc<Mutex<OneshotReceiver<Flag>>>,
  input: Arc<Mutex<MpscReceiver<String>>>
) -> MelodyResult<bool> {
  let config = Config::create().await?;
  let persist = Persist::create().await?;
  let persist_guilds = PersistGuilds::create().await?;

  let token = config.access().await.token.clone();
  let mut client = Client::builder(&token, intents())
    .event_handler(Handler).await.context("failed to init client")?;
  let core = Core::from(&client);

  // Insert data into the shared TypeMap
  let previous_build_id = persist.operate_mut_commit(|persist| Ok(persist.swap_build_id()))
    .await.context("failed to commit persist-guild state for build id")?;

  core.init(|data| {
    data.insert::<ConfigKey>(config);
    data.insert::<PersistKey>(persist);
    data.insert::<PersistGuildsKey>(persist_guilds.into());
    data.insert::<CleverBotKey>(CleverBotManager::new().into());
    data.insert::<MessageChainsKey>(MessageChains::new().into());
    data.insert::<ShardManagerKey>(client.shard_manager.clone());
    data.insert::<PreviousBuildIdKey>(previous_build_id);
    data.insert::<RestartKey>(false);
  }).await;

  // Handles command-line input from the terminal wrapper
  let input_task = tokio::spawn(self::input::input_task(input, core.clone()));
  // Handles an interrupt signal from the terminal wrapper
  let termination_task = tokio::spawn(termination_task(terminate, client.shard_manager.clone()));

  client.start().await.context("failed to start client")?;

  core.abort().await;

  input_task.abort();
  termination_task.abort();
  if core.take::<RestartKey>().await {
    info!("Restarting in 3 seconds...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    Ok(true)
  } else {
    Ok(false)
  }
}

#[derive(Debug)]
struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
  async fn ready(&self, _ctx: Context, ready_info: Ready) {
    info!("Bot connected: {} ({})", ready_info.user.tag(), ready_info.user.id);
  }

  async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
    let core = Core::from(ctx);

    operate_lock(core.get_default::<TasksKey>().await, |tasks| {
      // Spawn the task for cycling activity status unless it's already been spawned
      tasks.cycle_activities.get_or_insert_with(|| {
        tokio::spawn(cycle_activity_task(core.clone()))
      });
    }).await;

    for (guild_id, guild_name) in crate::commands::iter_guilds(&core, &guilds) {
      info!("Discovered guild: {guild_name} ({guild_id})");
    };

    if core.is_new_build().await {
      info!("New build detected, registering commands");
      crate::commands::register_commands(&core, &guilds).await.log();
    } else {
      info!("Old build detected, commands will not be re-registered");
    };
  }

  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    let core = Core::from(ctx);
    if let Interaction::ApplicationCommand(interaction) = interaction {
      crate::blueprint::dispatch(core, interaction, APPLICATION_COMMANDS).await.log();
    };
  }

  async fn message(&self, ctx: Context, message: Message) {
    let core = Core::from(ctx);

    let me = core.cache.current_user_id();
    if message.author.bot || message.author.id == me || message.content.is_empty() { return };

    let ptr = core.get::<MessageChainsKey>().await;
    let contribute = operate_lock(ptr, |message_chains| {
      message_chains.should_contribute(&message)
    }).await;

    if contribute {
      info!("Contributing to message chain in channel ({})", message.channel_id);
      message.channel_id.send_message(&core, |create| create.content(&message.content))
        .await.context_log("failed to send message");
    };

    if let Some(content) = get_message_replying_to(&message, me) {
      let options = ContentSafeOptions::new();
      let content = content_safe(&core, content, &options, &[]);

      info!("Recieved message for cleverbot: {content:?}");
      match core.get::<CleverBotKey>().await.send(message.channel_id, &content).await {
        Ok(reply) => {
          info!("Recieved reply from cleverbot: {reply:?}");
          let reply = content_safe(&core, reply, &options, &[]);
          crate::feature::cleverbot::send_reply(&core, &message, &reply).await.log();
        },
        Err(error) => {
          message.reply(&core, "There was an error getting a reply from cleverbot").await
            .context_log("failed to send cleverbot failure message");
          error!("Unable to get reply from cleverbot: {error}");
        }
      };
    };
  }
}

#[inline]
fn intents() -> GatewayIntents {
  GatewayIntents::GUILDS |
  GatewayIntents::GUILD_MEMBERS |
  GatewayIntents::GUILD_BANS |
  GatewayIntents::GUILD_EMOJIS_AND_STICKERS |
  //GatewayIntents::GUILD_INTEGRATIONS |
  //GatewayIntents::GUILD_PRESENCES |
  GatewayIntents::GUILD_MESSAGES |
  GatewayIntents::GUILD_MESSAGE_REACTIONS |
  GatewayIntents::MESSAGE_CONTENT
}

/// Gets the remaining content of a message if it either replies to the
/// given user, or begins with a mention of the given user.
fn get_message_replying_to(message: &Message, who: UserId) -> Option<&str> {
  match &message.referenced_message {
    Some(referenced_message) if referenced_message.author.id == who => Some(&message.content),
    Some(..) | None => crate::utils::strip_user_mention(&message.content, who)
  }
}

async fn termination_task(
  terminate: Arc<Mutex<OneshotReceiver<Flag>>>,
  shard_manager: Arc<Mutex<ShardManager>>
) {
  let mut terminate = terminate.lock().await;
  let kill_flag = (&mut *terminate).await.unwrap();

  kill_flag.set();
  shard_manager.lock().await.shutdown_all().await;
}

async fn cycle_activity_task(core: Core) {
  const ACTIVITY_CYCLE_TIME: Duration = Duration::from_secs(120);
  const ACTIVITIES: &[fn(&Core) -> Activity] = &[
    |_| Activity::playing("Minecraft 2"),
    |_| Activity::playing("Portal 3"),
    |_| Activity::playing("Pokemon\u{2122} Gun"),
    |_| Activity::playing("Artifact"),
    |_| Activity::playing("Group Fortification: The Sequel"),
    |_| Activity::playing("Arma 4"),
    |_| Activity::playing("Farming Simulator 23"),
    |_| Activity::playing("League of Legends"),
    |_| {
      let mut rng = crate::utils::create_rng();
      let number = ['1', '2', '3'].choose(&mut rng).unwrap();
      let fraction = ['\u{00bc}', '\u{00bd}', '\u{00be}', '\u{215b}', '\u{215c}', '\u{215d}', '\u{215e}'].choose(&mut rng).unwrap();
      Activity::playing(format!("Overwatch {number}{fraction}"))
    },
    |_| Activity::watching("you"),
    |core| Activity::watching(format!("{} guilds", core.cache.guild_count())),
    |core| Activity::watching(format!("{} users", core.cache.user_count())),
    |_| Activity::watching("Bocchi the Rock!"),
    |_| Activity::watching("Lucky\u{2606}Star"),
    |_| Activity::watching("Chainsaw Man"),
    |_| Activity::watching("Made In Abyss"),
    |_| Activity::listening("the screams"),
    |_| Activity::listening("the intrusive thoughts"),
    |_| Activity::listening("soft loli breathing 10 hours"),
    |_| Activity::competing("big balls competition"),
    |_| Activity::competing("taco eating competition")
  ];

  let mut rng = crate::utils::create_rng();
  let mut interval = tokio::time::interval(ACTIVITY_CYCLE_TIME);
  interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

  loop {
    interval.tick().await;
    core.set_activities(|_| {
      ACTIVITIES.choose(&mut rng).map(|f| f(&core))
    }).await;
  };
}
