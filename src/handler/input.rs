use crate::prelude::*;
use crate::data::Core;

use itertools::Itertools;
use log::Level;
use melody_commander::{Command, CommandError, CommandOutput, Parsed, resolve_args};
use serenity::model::id::GuildId;

use std::collections::HashSet;



const COMMANDS: &[Command<Target>] = &[
  Command::new_target("stop", Target::Stop),
  // a few months ago, i was using tmux, and typing `stop` did not immediately kill the tmux session,
  // now it does, and i can't find any resources on changing the magic words it uses, so i get to
  // pull out the thesaurus to work around this...
  Command::new_target("halt", Target::Stop),
  Command::new_group("plugin", &[
    Command::new_target("list", Target::PluginList),
    Command::new_target("enable", Target::PluginEnable),
    Command::new_target("disable", Target::PluginDisable)
  ]),
  Command::new_group("feeds", &[
    Command::new_target("respawn-all", Target::FeedsRespawnAll),
    Command::new_target("abort-all", Target::FeedsAbortAll),
    Command::new_target("list-tasks", Target::FeedsListTasks),
  ]),
  Command::new_target("update-yt-dlp", Target::UpdateYtDlp)
];

#[derive(Debug, Clone, Copy)]
enum Target {
  Stop,
  PluginList,
  PluginEnable,
  PluginDisable,
  FeedsRespawnAll,
  FeedsAbortAll,
  FeedsListTasks,
  UpdateYtDlp
}

#[derive(Debug, Clone)]
enum TargetArgs {
  Stop,
  PluginList(GuildId),
  PluginEnable(String, GuildId),
  PluginDisable(String, GuildId),
  FeedsRespawnAll,
  FeedsAbortAll,
  FeedsListTasks,
  UpdateYtDlp(Option<String>)
}

impl TryFrom<CommandOutput<Target>> for TargetArgs {
  type Error = CommandError;

  fn try_from(output: CommandOutput<Target>) -> Result<Self, Self::Error> {
    match output.target.clone() {
      Target::Stop => Ok(TargetArgs::Stop),
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
      Target::UpdateYtDlp => {
        let update_to = resolve_args::<Option<String>>(&output.remaining_args)?;
        Ok(TargetArgs::UpdateYtDlp(update_to))
      }
    }
  }
}

impl TargetArgs {
  async fn execute(self, agent: &mut InputAgent) -> MelodyResult {
    match self {
      TargetArgs::Stop => {
        agent.info("Shutdown triggered");
        agent.core.trigger_shutdown().await;
      },
      TargetArgs::PluginList(guild_id) => {
        let plugins = agent.plugin_list(guild_id).await;
        agent.info(format!("Plugins for guild ({guild_id}): {}", plugins.iter().join(", ")));
      },
      TargetArgs::PluginEnable(plugin, guild_id) => {
        agent.plugin_enable(&plugin, guild_id).await?;
        agent.info(format!("Enabled plugin {plugin} for guild ({guild_id})"));
      },
      TargetArgs::PluginDisable(plugin, guild_id) => {
        agent.plugin_disable(&plugin, guild_id).await?;
        agent.info(format!("Disabled plugin {plugin} for guild ({guild_id})"));
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
          agent.debug(format!("Feed task: {feed} ({})", if running { "running" } else { "stopped" }));
        };
      },
      TargetArgs::UpdateYtDlp(update_to) => {
        let yt_dlp_path = agent.core.operate_config(|config| {
          config.music_player.as_ref().map(|music_player| music_player.ytdlp_path.clone())
        }).await;

        if let Some(yt_dlp_path) = yt_dlp_path {
          let update_to = update_to.as_deref().unwrap_or("latest");
          agent.info(format!("Updating yt-dlp..."));
          let yt_dlp_output = crate::utils::youtube::update_yt_dlp(yt_dlp_path, update_to).await?;
          for yt_dlp_output_line in yt_dlp_output.lines() {
            agent.trace(format!("(yt-dlp): {yt_dlp_output_line:?}"));
          };
        } else {
          agent.error(format!("Cannot update yt-dlp, no yt-dlp path in config"));
        };
      }
    };

    Ok(())
  }
}

#[derive(Debug, Clone)]
pub struct InputAgent {
  core: Core,
  output: Vec<(Level, String)>
}

impl InputAgent {
  pub fn new(core: impl Into<Core>) -> Self {
    InputAgent {
      core: core.into(),
      output: Vec::new()
    }
  }

  pub async fn line(&mut self, line: String) -> MelodyResult {
    melody_commander::apply(&line, COMMANDS)
      .and_then(TargetArgs::try_from)?
      .execute(self).await?;
    Ok(())
  }

  #[inline]
  pub fn output(&self) -> &[(Level, String)] {
    &self.output
  }

  #[inline]
  pub fn into_output(self) -> Vec<(Level, String)> {
    self.output
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

  #[allow(unused)]
  pub fn log(&mut self, level: Level, message: impl Into<String>) {
    let message = message.into();
    log!(level, "{message}");
    self.output.push((level, message));
  }

  #[allow(unused)]
  pub fn error(&mut self, message: impl Into<String>) {
    self.log(Level::Error, message);
  }

  #[allow(unused)]
  pub fn warn(&mut self, message: impl Into<String>) {
    self.log(Level::Warn, message);
  }

  #[allow(unused)]
  pub fn info(&mut self, message: impl Into<String>) {
    self.log(Level::Info, message);
  }

  #[allow(unused)]
  pub fn debug(&mut self, message: impl Into<String>) {
    self.log(Level::Debug, message);
  }

  #[allow(unused)]
  pub fn trace(&mut self, message: impl Into<String>) {
    self.log(Level::Trace, message);
  }
}
