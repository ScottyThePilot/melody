use crate::MelodyResult;
use crate::commands::APPLICATION_COMMANDS;
use crate::data::*;
use crate::feature::message_chains::MessageChains;
use crate::utils::{Contextualize, Loggable};

use rand::Rng;
use serenity::model::application::interaction::Interaction;
use serenity::model::channel::Message;
use serenity::client::{Context, Client, EventHandler};
use serenity::model::gateway::{GatewayIntents, Ready};
use serenity::model::id::GuildId;

use std::time::Duration;



/// Performs a clean launch of the bot, returning true if the bot expects to be restarted, and false if not.
pub async fn launch() -> MelodyResult<bool> {
  let config = Config::create().await?;
  let persist = Persist::create().await?;
  let persist_guilds = PersistGuilds::create().await?;

  let previous_build_id = Persist::swap_build_id(&persist).await;
  let token = config.access().await.token.clone();
  let mut client = Client::builder(&token, intents())
    .event_handler(Handler).await.context("failed to init client")?;

  // Insert data into the shared TypeMap
  data_insert::<ConfigKey>(&client, config).await;
  data_insert::<PersistKey>(&client, persist).await;
  data_insert::<PersistGuildsKey>(&client, persist_guilds.into()).await;
  data_insert::<MessageChainsKey>(&client, MessageChains::new().into()).await;
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
  async fn ready(&self, _ctx: Context, ready_info: Ready) {
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

    let contribute = operate_lock(
      data_get::<MessageChainsKey>(&ctx).await,
      |message_chains| message_chains.should_contribute(&message)
    ).await;

    if contribute {
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
