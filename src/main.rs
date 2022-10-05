#![warn(missing_debug_implementations)]
extern crate ahash;
extern crate chrono;
extern crate command_attr;
extern crate fern;
#[macro_use]
extern crate log;
extern crate once_cell;
extern crate serenity;
extern crate singlefile;
#[macro_use]
extern crate serde;
extern crate serde_cbor;
#[macro_use]
extern crate thiserror;
extern crate tokio;
extern crate toml;

#[macro_use]
pub(crate) mod utils;
#[macro_use]
pub(crate) mod blueprint;
pub(crate) mod build_id;
pub(crate) mod commands;
pub(crate) mod data;
pub(crate) mod feature;

use crate::commands::APPLICATION_COMMANDS;
use crate::data::*;
use crate::utils::{Contextualize, Loggable};

use fern::Dispatch;
use rand::Rng;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::client::{Context, Client, EventHandler};
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::GuildId;

use std::time::Duration;



#[tokio::main]
async fn main() {
  setup_logger().unwrap();

  loop {
    match start().await {
      Ok(true) => continue,
      Ok(false) => break,
      Err(error) => return error!("{error}")
    };
  };
}

async fn start() -> MelodyResult<bool> {
  let config = Config::create().await.context("failed to load config.toml")?;
  let persist = Persist::create().await.context("failed to load data/persist.bin")?;

  let previous_build_id = Persist::swap_build_id(&persist).await;
  let token = config.access().await.token.clone();
  let mut client = Client::builder(&token, intents())
    .event_handler(Handler).await.context("failed to init client")?;

  // Insert data into the shared TypeMap
  data_insert::<BrainKey>(&client, Brain::create()).await;
  data_insert::<ConfigKey>(&client, config).await;
  data_insert::<PersistKey>(&client, persist).await;
  data_insert::<PersistGuildsKey>(&client, PersistGuilds::create()).await;
  data_insert::<ShardManagerKey>(&client, client.shard_manager.clone()).await;
  data_insert::<PreviousBuildIdKey>(&client, previous_build_id).await;
  data_insert::<RestartKey>(&client, false).await;

  let shard_manager = client.shard_manager.clone();
  let ctrl_c = tokio::spawn(async move {
    tokio::signal::ctrl_c().await.unwrap();
    shard_manager.lock().await.shutdown_all().await;
    info!("Manual shutdown...");
  });

  client.start().await.context("failed to start client")?;
  ctrl_c.abort();

  if data_take::<RestartKey>(&client).await {
    info!("Restarting in 10 seconds...");
    tokio::time::sleep(Duration::from_secs(10)).await;

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
    info!("Bot connected: {} ({})", ready_info.user.tag(), ready_info.user.id);
  }

  async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
    if data_get::<PreviousBuildIdKey>(&ctx).await != crate::build_id::get() {
      info!("New build detected, registering commands");
      crate::commands::register_commands(&ctx, &guilds).await.log();
    } else {
      info!("Commands will not be re-registered");
    };
  }

  async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(interaction) = interaction {
      crate::blueprint::dispatch(&ctx, interaction, APPLICATION_COMMANDS).await.log();
    };
  }

  async fn message(&self, ctx: Context, message: Message) {
    let me = ctx.cache.current_user_id();
    if message.author.bot || message.author.id == me || message.content.is_empty() { return };

    let should_contribute = data_access_brain_mut(&ctx, |mut brain| {
      let should_contribute = brain.observe_message(&message) >= 3 && rand::thread_rng().gen_bool(0.50);
      if should_contribute { brain.reset_message_chain(message.channel_id) };
      should_contribute
    }).await;

    if should_contribute {
      info!("Contributing to message chain in channel ({})", message.channel_id);
      message.channel_id.send_message(&ctx, |create| create.content(&message.content))
        .await.context("failed to send message").log();
    };

    if message.mentions_user_id(me) && rand::thread_rng().gen_bool(0.10) {
      message.channel_id.send_message(&ctx, |create| create.content("What?"))
        .await.context("failed to send message").log();
    };
  }
}

pub type MelodyResult<T = ()> = Result<T, MelodyError>;

#[derive(Debug, Error)]
pub enum MelodyError {
  #[error("File Error: {1} ({0})")]
  FileError(singlefile::Error, &'static str),
  #[error("Serenity Error: {1} ({0})")]
  SerenityError(serenity::Error, &'static str),
  #[error("Invalid command")]
  InvalidCommand,
  #[error("Invalid arguments")]
  InvalidArguments
}

fn setup_logger() -> Result<(), fern::InitError> {
  Dispatch::new()
    .format(move |out, message, record| {
      out.finish(format_args!(
        "{}[{}]({}) {}",
        chrono::Local::now().format("[%H:%M:%S]"),
        record.level(),
        record.target(),
        message
      ))
    })
    .level(log::LevelFilter::Warn)
    .level_for("melody", log::LevelFilter::Trace)
    .chain(std::io::stdout())
    .chain({
      std::fs::create_dir_all("./data/")?;
      fern::log_file("./data/latest.log")?
    })
    .apply()?;
  Ok(())
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
