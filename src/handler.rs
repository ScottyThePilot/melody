//! Items and functions associated with launching the bot and handling discord events
mod input;

use crate::MelodyResult;
use crate::commands::APPLICATION_COMMANDS;
use crate::data::*;
use crate::feature::message_chains::MessageChains;
use crate::terminal::Flag;
use crate::utils::{Contextualize, Loggable};

use chrono::Utc;
use rand::Rng;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::client::{Context, Client, EventHandler};
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::GuildId;
use tokio::sync::Mutex;

use tokio::sync::mpsc::{unbounded_channel as mpsc_channel, UnboundedReceiver as MpscReceiver};
use tokio::sync::oneshot::Receiver as OneshotReceiver;

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
  let previous_build_id = persist.operate_mut(|persist| persist.swap_build_id()).await;
  core.init(|data| {
    data.insert::<ConfigKey>(config);
    data.insert::<PersistKey>(persist);
    data.insert::<PersistGuildsKey>(persist_guilds.into());
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
  async fn ready(&self, ctx: Context, ready_info: Ready) {
    let core = Core::from(ctx);

    info!("Bot connected: {} ({})", ready_info.user.tag(), ready_info.user.id);
  }

  async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
    let core = Core::from(ctx);

    for (guild_id, guild_name) in crate::commands::iter_guilds(&core, &guilds) {
      info!("Discovered guild: {guild_name} ({guild_id})");
    };

    if core.get::<PreviousBuildIdKey>().await != crate::build_id::get() {
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

    if message.mentions_user_id(me) && rand::thread_rng().gen_bool(0.10) {
      message.channel_id.send_message(&core, |create| create.content("What?"))
        .await.context_log("failed to send message");
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

async fn termination_task(
  terminate: Arc<Mutex<OneshotReceiver<Flag>>>,
  shard_manager: Arc<Mutex<ShardManager>>
) {
  let mut terminate = terminate.lock().await;
  let kill_flag = (&mut *terminate).await.unwrap();

  kill_flag.set();
  shard_manager.lock().await.shutdown_all().await;
}
