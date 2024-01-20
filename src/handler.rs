//! Items and functions associated with launching the bot and handling discord events
mod input;

use crate::{MelodyResult, MelodyError};
use crate::commands::COMMANDS;
use crate::data::*;
use crate::feature::cleverbot::{CleverBotLoggerWrapper, CleverBotWrapper};
use crate::feature::feed::{Feed, FeedManager, FeedEventHandler};
use crate::feature::message_chains::MessageChains;
use crate::feature::music_player::MusicPlayer;
use crate::terminal::Flag;
use crate::utils::{Contextualize, Loggable};

use rand::seq::SliceRandom;
use rss_feed::{TwitterPost, YouTubeVideo};
use rss_feed::url::Url;
use reqwest::Client as HttpClient;
use serenity::gateway::{ActivityData, ShardManager};
use serenity::model::application::Interaction;
use serenity::model::channel::Message;
use serenity::client::{Context, Client, EventHandler};
use serenity::model::channel::{Reaction, ReactionType};
use serenity::model::gateway::{ActivityType, Ready};
use serenity::model::guild::Member;
use serenity::model::id::{ChannelId, GuildId, UserId, RoleId};
use serenity::utils::{content_safe, ContentSafeOptions};
use songbird::{SerenityInit, Config as SongbirdConfig};
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver as MpscReceiver;
use tokio::sync::oneshot::Receiver as OneshotReceiver;
use tokio::time::MissedTickBehavior;

use std::collections::HashSet;
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

  let (token, intents, ytdlp_path) = config.operate(|config| {
    info!("YouTube RSS feeds are {}", if config.rss.youtube.is_some() { "enabled" } else { "disabled" });
    info!("Twitter RSS feeds are {}", if config.rss.twitter.is_some() { "enabled" } else { "disabled" });
    let ytdlp_path = config.music_player.as_ref().map(|mp| mp.ytdlp_path.clone());
    (config.token.clone(), config.intents, ytdlp_path)
  }).await;

  let mut client = Client::builder(&token, intents)
    .event_handler(Handler)
    .register_songbird_from_config(SongbirdConfig::default())
    .await.context("failed to init client")?;
  let core = Core::from(&client);

  // Insert data into the shared TypeMap
  let previous_build_id = persist.operate_mut_commit(|persist| Ok(persist.swap_build_id()))
    .await.context("failed to commit persist-guild state for build id")?;

  let cleverbot_delay = config.operate(|config| config.cleverbot_ratelimit).await;
  let cleverbot_logger = CleverBotLoggerWrapper::create()
    .await.context("failed to create cleverbot logger")?;

  let http_client = HttpClient::new();

  core.init(|data| {
    data.insert::<ConfigKey>(config);
    data.insert::<PersistKey>(persist);
    data.insert::<PersistGuildsKey>(persist_guilds.into());
    data.insert::<CleverBotKey>(CleverBotWrapper::new(Duration::from_secs_f64(cleverbot_delay)));
    data.insert::<CleverBotLoggerKey>(cleverbot_logger);
    data.insert::<FeedKey>(Arc::new(Mutex::new(FeedManager::new(http_client.clone(), FeedHandler))));
    data.insert::<MessageChainsKey>(MessageChains::new().into());
    data.insert::<MusicPlayerKey>(ytdlp_path.map(|ytdlp_path| {
      Arc::new(MusicPlayer::new(ytdlp_path, http_client.clone()))
    }));
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

    // Attempt to register all subscribed RSS feeds
    let feed_wrapper = core.get::<FeedKey>().await;
    let feeds = core.operate_persist(|persist| {
      persist.feeds.keys().cloned().collect::<Vec<Feed>>()
    }).await;
    for feed in feeds {
      feed_wrapper.lock().await.register(&core, feed).await;
    };
  }

  async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
    let core = Core::from(ctx);

    operate_lock(core.get_default::<TasksKey>().await, |tasks| {
      // Spawn the task for cycling activity status unless it's already been spawned
      tasks.cycle_activities.get_or_insert_with(|| {
        tokio::spawn(cycle_activity_task(core.clone()))
      });

      // Spawn the task for respawning the feed manager's tasks
      tasks.respawn_feed_tasks.get_or_insert_with(|| {
        tokio::spawn(respawn_feed_tasks_task(core.clone()))
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
    if let Interaction::Command(interaction) = interaction {
      crate::blueprint::dispatch(core, interaction, COMMANDS).await.log();
    };
  }

  async fn message(&self, ctx: Context, message: Message) {
    let core = Core::from(ctx);

    let me = core.current_user_id();
    if message.author.bot || message.author.id == me || message.content.is_empty() { return };

    if let Some(guild_id) = message.guild_id {
      let emojis = crate::utils::parse_emojis(&message.content);
      core.operate_persist_guild_commit(guild_id, |persist_guild| {
        // don't be greedy
        let mut rng = rand::thread_rng();
        if let Some(&emoji) = emojis.choose(&mut rng) {
          persist_guild.emoji_stats.increment_emoji_uses(emoji, message.author.id);
        };
        Ok(())
      }).await.log();
    };

    if should_contribute_message_chain(&core, &message).await {
      message.channel_id.say(&core, &message.content)
        .await.context_log("failed to send message");
    };

    if let Some(content) = get_message_replying_to(&message, me) {
      let options = ContentSafeOptions::new();
      let content = content_safe(&core, content, &options, &[]);

      info!("Sending message to cleverbot: {content:?}");
      match core.get::<CleverBotKey>().await.send(message.channel_id, &content).await {
        Ok(reply) => {
          info!("Recieved reply from cleverbot: {reply:?}");
          let reply = content_safe(&core, reply, &options, &[]);
          crate::feature::cleverbot::send_reply(&core, &message, &reply).await.log();
          core.get::<CleverBotLoggerKey>().await.log(message.channel_id, content, reply).await.log();
        },
        Err(error) => {
          message.reply(&core, "There was an error getting a reply from cleverbot").await
            .context_log("failed to send cleverbot failure message");
          error!("Unable to get reply from cleverbot: {error}");
        }
      };
    };
  }

  async fn guild_member_addition(&self, ctx: Context, mut member: Member) {
    let core = Core::from(ctx);
    add_join_roles(&core, &mut member).await.log();
  }

  async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
    let core = Core::from(ctx);

    if let (Some(guild_id), Some(user_id), ReactionType::Custom { id: emoji_id, .. }) = (reaction.guild_id, reaction.user_id, reaction.emoji) {
      core.operate_persist_guild_commit(guild_id, |persist_guild| {
        persist_guild.emoji_stats.increment_emoji_uses(emoji_id, user_id);
        Ok(())
      }).await.log();
    };
  }

  async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
    let core = Core::from(ctx);

    if let (Some(guild_id), Some(user_id), ReactionType::Custom { id: emoji_id, .. }) = (reaction.guild_id, reaction.user_id, reaction.emoji) {
      core.operate_persist_guild_commit(guild_id, |persist_guild| {
        persist_guild.emoji_stats.decrement_emoji_uses(emoji_id, user_id);
        Ok(())
      }).await.log();
    };
  }
}

const FEED_POST_DELAY: Duration = Duration::from_secs(3);

#[derive(Debug)]
struct FeedHandler;

#[serenity::async_trait]
impl FeedEventHandler for FeedHandler {
  async fn feed_youtube_video(&self, core: Core, channel: &str, video: YouTubeVideo) {
    let feed = Feed::YouTube { channel: channel.to_owned() };
    let delivery_channels = core.operate_persist(|persist| {
      persist.feeds.get(&feed).map_or_else(Vec::new, |feed_state| {
        feed_state.guilds.values().copied().collect::<Vec<ChannelId>>()
      })
    }).await;

    let link = core.operate_config(|config| match config.rss.youtube.as_deref() {
      Some(config) => with_domain(&video.link, &config.display_domain),
      None => video.link.clone()
    }).await;

    for channel in delivery_channels {
      channel.say(&core, link.as_str())
        .await.context_log("failed to send youtube video message");
      tokio::time::sleep(FEED_POST_DELAY).await;
    };
  }

  async fn feed_twitter_post(&self, core: Core, handle: &str, post: TwitterPost) {
    let feed = Feed::Twitter { handle: handle.to_owned() };
    let delivery_channels = core.operate_persist(|persist| {
      persist.feeds.get(&feed).map_or_else(Vec::new, |feed_state| {
        feed_state.guilds.values().copied().collect::<Vec<ChannelId>>()
      })
    }).await;

    let link = core.operate_config(|config| match config.rss.twitter.as_deref() {
      Some(config) => with_domain(&post.link, &config.display_domain),
      None => post.link.clone()
    }).await;

    for channel in delivery_channels {
      channel.say(&core, link.as_str())
        .await.context_log("failed to send twitter post message");
      tokio::time::sleep(FEED_POST_DELAY).await;
    };
  }
}

/// Gets the remaining content of a message if it either replies to the
/// given user, or begins with a mention of the given user.
fn get_message_replying_to(message: &Message, who: UserId) -> Option<&str> {
  match &message.referenced_message {
    Some(referenced_message) if referenced_message.author.id == who => Some(&message.content),
    Some(..) | None => crate::utils::strip_user_mention(&message.content, who)
  }
}

async fn add_join_roles(core: &Core, member: &mut Member) -> MelodyResult {
  let (roles, missing_roles) = core.operate_persist_guild(member.guild_id, |persist_guild| {
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
    core.operate_persist_guild_commit(member.guild_id, |persist_guild| {
      for role_id in missing_roles {
        persist_guild.join_roles.remove(&role_id);
        warn!("Removed non-existent role ({}) for guild ({})", role_id, member.guild_id);
      };

      Ok(())
    }).await?;
  };

  Ok(())
}

async fn should_contribute_message_chain(core: &Core, message: &Message) -> bool {
  operate!(core, operate_lock::<MessageChainsKey>, |message_chains| message_chains.should_contribute(&message))
}

fn with_domain(url: &Url, domain: &str) -> Url {
  let mut url = url.clone();
  let _ = url.set_host(Some(domain));
  url
}

/// Gets a list of games people are playing in a given guild
fn games(core: &Core, guild_id: GuildId) -> HashSet<String> {
  core.cache.guild(guild_id).map_or_else(HashSet::new, |guild| {
    guild.presences.values()
      .filter(|&presence| presence.user.bot != Some(true))
      .flat_map(|presence| presence.activities.iter())
      .filter(|&activity| activity.kind == ActivityType::Playing)
      .map(|activity| activity.name.clone())
      .collect::<HashSet<String>>()
  })
}

fn random_game(core: &Core) -> Option<String> {
  let mut rng = crate::utils::create_rng();
  let games = core.cache.guilds().into_iter()
    .flat_map(|guild_id| games(core, guild_id))
    .collect::<Vec<String>>();
  games.choose(&mut rng).cloned()
}

async fn termination_task(
  terminate: Arc<Mutex<OneshotReceiver<Flag>>>,
  shard_manager: Arc<ShardManager>
) {
  let mut terminate = terminate.lock().await;
  let kill_flag = (&mut *terminate).await.unwrap();

  kill_flag.set();
  shard_manager.shutdown_all().await;
}

// Yes, a tasks-task, what a mouthfull
async fn respawn_feed_tasks_task(core: Core) {
  const RESPAWN_INTERVAL: Duration = Duration::from_secs(21600); // 6 hours

  let mut interval = tokio::time::interval(RESPAWN_INTERVAL);
  interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
  interval.tick().await;

  loop {
    interval.tick().await;
    info!("Respawning feed tasks");
    let feed_wrapper = core.get::<FeedKey>().await;
    feed_wrapper.lock().await.respawn_all(&core).await;
  };
}

async fn cycle_activity_task(core: Core) {
  const NUMBERS: &[char] = &['1', '2', '3'];
  const FRACTIONS: &[char] = &[
    '\u{00bd}', '\u{00bc}', '\u{00be}',
    '\u{215b}', '\u{215c}', '\u{215d}', '\u{215e}'
  ];

  const ACTIVITY_CYCLE_TIME: Duration = Duration::from_secs(120);
  const ACTIVITIES: &[fn(&Core) -> ActivityData] = &[
    |_| ActivityData::playing("Minecraft 2"),
    |_| ActivityData::playing("Portal 3"),
    |_| ActivityData::playing("Pokemon\u{2122} Gun"),
    |_| ActivityData::playing("ULTRAKILL"),
    |_| ActivityData::playing("Mordhau"),
    |_| ActivityData::playing("Genshin Impact"),
    |_| ActivityData::playing("Honkai Star Rail"),
    |_| ActivityData::playing("Tower of Fantasy"),
    |_| ActivityData::playing("Arknights"),
    |_| ActivityData::playing("Azur Lane"),
    |_| ActivityData::playing("Girls' Frontline"),
    |_| ActivityData::playing("Blue Archive"),
    |_| ActivityData::playing("The Battle Cats"),
    |_| ActivityData::playing("Artifact"),
    |_| ActivityData::playing("Group Fortification: The Sequel"),
    |_| ActivityData::playing("Band Bastion: The Second Coming"),
    |_| ActivityData::playing("Fortress II (With Team)"),
    |_| ActivityData::playing("Gang Garrison: Second Edition"),
    |_| ActivityData::playing("Club Citadel II"),
    |_| ActivityData::playing("Squad Castle Defense 2"),
    |_| ActivityData::playing("Blazing Badge: Four Houses"),
    |_| ActivityData::playing("Arma 4"),
    |_| ActivityData::playing("Farming Simulator 25"),
    |_| ActivityData::playing("League of Legends"),
    |_| ActivityData::playing("DOTA 2"),
    |_| ActivityData::playing("Katawa Shoujou"),
    |_| {
      let mut rng = rand::thread_rng();
      let number = *NUMBERS.choose(&mut rng).unwrap();
      let fraction = *FRACTIONS.choose(&mut rng).unwrap();
      ActivityData::playing(format!("Overwatch {number}{fraction}"))
    },
    |_| ActivityData::watching("you"),
    |core| match random_game(core) {
      Some(game) => ActivityData::watching(format!("you play {game}")),
      None => ActivityData::watching("nobody :(")
    },
    |core| ActivityData::watching(format!("{} guilds", core.cache.guild_count())),
    |core| ActivityData::watching(format!("{} users", core.cache.user_count())),
    |_| ActivityData::watching("Bocchi the Rock!"),
    |_| ActivityData::watching("Lucky\u{2606}Star"),
    |_| ActivityData::watching("Chainsaw Man"),
    |_| ActivityData::watching("Made In Abyss"),
    |_| ActivityData::watching("the fog approach"),
    |_| ActivityData::listening("the screams"),
    |_| ActivityData::listening("the intrusive thoughts"),
    |_| ActivityData::listening("soft loli breathing 10 hours"),
    |_| ActivityData::competing("big balls competition"),
    |_| ActivityData::competing("taco eating competition"),
    |_| ActivityData::competing("shadow wizard money gang")
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
