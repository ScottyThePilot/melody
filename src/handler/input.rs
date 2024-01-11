use crate::MelodyResult;
use crate::data::Core;
use crate::utils::Loggable;

use commander::{Command, CommandError, CommandOutput, Parsed, resolve_args};
use itertools::Itertools;
use serenity::model::id::GuildId;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver as MpscReceiver;

use std::collections::HashSet;
use std::sync::Arc;



pub async fn input_task(input: Arc<Mutex<MpscReceiver<String>>>, core: Core) {
  let agent = InputAgent::new(core);
  let mut input = input.lock().await;
  while let Some(line) = input.recv().await {
    agent.line(line).await.log();
  };
}

const COMMANDS: &[Command<Target>] = &[
  Command::new_target("stop", Target::Stop),
  Command::new_target("restart", Target::Restart),
  Command::new_group("plugin", &[
    Command::new_target("list", Target::PluginList),
    Command::new_target("enable", Target::PluginEnable),
    Command::new_target("disable", Target::PluginDisable)
  ]),
  Command::new_group("feeds", &[
    Command::new_target("respawn-all", Target::FeedsRespawnAll),
    Command::new_target("abort-all", Target::FeedsAbortAll),
    Command::new_target("list-tasks", Target::FeedsListTasks),
  ])
];

#[derive(Debug, Clone, Copy)]
enum Target {
  Stop,
  Restart,
  PluginList,
  PluginEnable,
  PluginDisable,
  FeedsRespawnAll,
  FeedsAbortAll,
  FeedsListTasks
}

#[derive(Debug, Clone)]
enum TargetArgs {
  Stop,
  Restart,
  PluginList(GuildId),
  PluginEnable(String, GuildId),
  PluginDisable(String, GuildId),
  FeedsRespawnAll,
  FeedsAbortAll,
  FeedsListTasks
}

impl TryFrom<CommandOutput<Target>> for TargetArgs {
  type Error = CommandError;

  fn try_from(output: CommandOutput<Target>) -> Result<Self, Self::Error> {
    match output.target.clone() {
      Target::Stop => Ok(TargetArgs::Stop),
      Target::Restart => Ok(TargetArgs::Restart),
      Target::PluginList => {
        let Parsed(guild_id) = resolve_args::<Parsed<GuildId>>(&output.remaining_args)?;
        Ok(TargetArgs::PluginList(guild_id))
      },
      Target::PluginEnable => {
        let (plugin, Parsed(guild_id)) = resolve_args::<(String, Parsed<GuildId>)>(&output.remaining_args)?;
        Ok(TargetArgs::PluginEnable(plugin, guild_id))
      },
      Target::PluginDisable => {
        let (plugin, Parsed(guild_id)) = resolve_args::<(String, Parsed<GuildId>)>(&output.remaining_args)?;
        Ok(TargetArgs::PluginDisable(plugin, guild_id))
      },
      Target::FeedsRespawnAll => Ok(TargetArgs::FeedsRespawnAll),
      Target::FeedsAbortAll => Ok(TargetArgs::FeedsAbortAll),
      Target::FeedsListTasks => Ok(TargetArgs::FeedsListTasks),
    }
  }
}

impl TargetArgs {
  async fn execute(self, agent: &InputAgent) -> MelodyResult {
    match self {
      TargetArgs::Stop => {
        info!("Shutdown triggered");
        agent.core.trigger_shutdown().await;
      },
      TargetArgs::Restart => {
        info!("Restart triggered");
        agent.core.trigger_shutdown_restart().await;
      },
      TargetArgs::PluginList(guild_id) => {
        let plugins = agent.plugin_list(guild_id).await;
        info!("Plugins for guild ({guild_id}): {}", plugins.iter().join(", "));
      },
      TargetArgs::PluginEnable(plugin, guild_id) => {
        agent.plugin_enable(&plugin, guild_id).await?;
        info!("Enabled plugin {plugin} for guild ({guild_id})");
      },
      TargetArgs::PluginDisable(plugin, guild_id) => {
        agent.plugin_disable(&plugin, guild_id).await?;
        info!("Disabled plugin {plugin} for guild ({guild_id})");
      },
      TargetArgs::FeedsRespawnAll => {
        let feed_wrapper = agent.core.get::<crate::data::FeedKey>().await;
        feed_wrapper.lock().await.respawn_all(&agent.core).await;
      },
      TargetArgs::FeedsAbortAll => {
        let feed_wrapper = agent.core.get::<crate::data::FeedKey>().await;
        feed_wrapper.lock().await.abort_all();
      },
      TargetArgs::FeedsListTasks => {
        let feed_wrapper = agent.core.get::<crate::data::FeedKey>().await;
        for (feed, running) in feed_wrapper.lock().await.tasks() {
          debug!("Feed task: {feed} ({})", if running { "running" } else { "stopped" });
        };
      }
    };

    Ok(())
  }
}

#[derive(Debug, Clone)]
struct InputAgent {
  core: Core
}

impl InputAgent {
  fn new(core: impl Into<Core>) -> Self {
    InputAgent { core: core.into() }
  }

  async fn line(&self, line: String) -> MelodyResult {
    match commander::apply(&line, COMMANDS).and_then(TargetArgs::try_from) {
      Ok(target_args) => target_args.execute(self).await?,
      Err(err) => error!("{err}")
    };

    Ok(())
  }

  async fn plugin_list(&self, guild_id: GuildId) -> HashSet<String> {
    self.core.operate_persist(|persist| persist.get_guild_plugins(guild_id)).await
  }

  async fn plugin_enable(&self, plugin: &str, guild_id: GuildId) -> MelodyResult {
    self.plugins_modify(guild_id, |plugins| {
      plugins.insert(plugin.to_owned());
    }).await
  }

  async fn plugin_disable(&self, plugin: &str, guild_id: GuildId) -> MelodyResult {
    self.plugins_modify(guild_id, |plugins| {
      plugins.remove(plugin);
    }).await
  }

  async fn plugins_modify(&self, guild_id: GuildId, operation: impl FnOnce(&mut HashSet<String>)) -> MelodyResult {
    crate::commands::register_guild_commands(&self.core, guild_id, {
      self.core.operate_persist_commit(|persist| {
        let guild_plugins = persist.get_guild_plugins_mut(guild_id);
        operation(guild_plugins);
        Ok(guild_plugins.clone())
      }).await?
    }).await
  }
}
