use crate::prelude::*;
use crate::data::Core;

use futures::future::BoxFuture;
use log::Level;
use melody_commander::{Command, Commands, Parsed, resolve_args};
use serenity::model::id::GuildId;

use std::collections::HashSet;

macro_rules! command {
  ($name:literal: $function:ident($($arg:ident: $Arg:ty),*)) => (
    <Command<CommandFunction>>::new_target($name, |agent, remaining_args| Box::pin(async move {
      let ($($arg,)*) = resolve_args::<($($Arg,)*)>(&remaining_args)?;
      agent.$function($($arg,)*).await
    }))
  );
  ($name:literal: [$($command:expr),* $(,)?]) => (
    <Command<CommandFunction>>::new_group($name, &[$($command),*])
  );
}

type CommandFunction = for<'a> fn(&'a mut InputAgent, Box<[String]>) -> BoxFuture<'a, MelodyResult>;

const COMMANDS2: Commands<CommandFunction> = &[
  command!("stop": command_stop()),
  // a few months ago, i was using tmux, and typing `stop` did not immediately kill the tmux session,
  // now it does, and i can't find any resources on changing the magic words it uses, so i get to
  // pull out the thesaurus to work around this...
  command!("halt": command_stop()),
  command!("plugin": [
    command!("list": command_plugin_list(guild_id: Parsed<GuildId>)),
    command!("enable": command_plugin_enable(plugin: String, guild_id: Parsed<GuildId>)),
    command!("disable": command_plugin_disable(plugin: String, guild_id: Parsed<GuildId>))
  ]),
  command!("feeds": [
    command!("respawn-all": command_feeds_respawn_all()),
    command!("abort-all": command_feeds_abort_all()),
    command!("list-tasks": command_feeds_list_tasks())
  ]),
  command!("update-yt-dlp": command_update_yt_dlp(update_to: Option<String>))
];

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

  async fn command_stop(&mut self) -> MelodyResult {
    self.info("Shutdown triggered");
    self.core.trigger_shutdown().await;
    Ok(())
  }

  async fn command_plugin_list(&mut self, guild_id: GuildId) -> MelodyResult {
    let plugins = self.plugin_list(guild_id).await;
    self.info(format!("Plugins for guild ({guild_id}): {}", plugins.iter().join(", ")));
    Ok(())
  }

  async fn command_plugin_enable(&mut self, plugin: String, guild_id: GuildId) -> MelodyResult {
    self.plugin_enable(&plugin, guild_id).await?;
    self.info(format!("Enabled plugin {plugin} for guild ({guild_id})"));
    Ok(())
  }

  async fn command_plugin_disable(&mut self, plugin: String, guild_id: GuildId) -> MelodyResult {
    self.plugin_disable(&plugin, guild_id).await?;
    self.info(format!("Disabled plugin {plugin} for guild ({guild_id})"));
    Ok(())
  }

  async fn command_feeds_respawn_all(&mut self) -> MelodyResult {
    let feed_wrapper = self.core.state.feed.clone();
    feed_wrapper.lock().await.respawn_all(&self.core).await;
    Ok(())
  }

  async fn command_feeds_abort_all(&mut self) -> MelodyResult {
    let feed_wrapper = self.core.state.feed.clone();
    feed_wrapper.lock().await.abort_all();
    Ok(())
  }

  async fn command_feeds_list_tasks(&mut self) -> MelodyResult {
    let feed_wrapper = self.core.state.feed.clone();
    for (feed, running) in feed_wrapper.lock().await.tasks() {
      self.debug(format!("Feed task: {feed} ({})", if running { "running" } else { "stopped" }));
    };

    Ok(())
  }

  async fn command_update_yt_dlp(&mut self, update_to: Option<String>) -> MelodyResult {
    let yt_dlp_path = self.core.operate_config(|config| {
      config.music_player.as_ref().map(|music_player| music_player.ytdlp_path.clone())
    }).await;

    if let Some(yt_dlp_path) = yt_dlp_path {
      let update_to = update_to.as_deref().unwrap_or("latest");
      self.info(format!("Updating yt-dlp..."));
      let yt_dlp_output = crate::utils::youtube::update_yt_dlp(yt_dlp_path, update_to).await?;
      for yt_dlp_output_line in yt_dlp_output.lines() {
        self.trace(format!("(yt-dlp): {yt_dlp_output_line:?}"));
      };
    } else {
      self.error(format!("Cannot update yt-dlp, no yt-dlp path in config"));
    };

    Ok(())
  }

  pub async fn line(&mut self, line: String) -> MelodyResult {
    let output = melody_commander::apply(&line, COMMANDS2)?;
    (output.target)(self, output.remaining_args).await?;
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
    use crate::data::MelodyFrameworkKey;
    let commands = self.core.get::<MelodyFrameworkKey>().await.read_commands_owned().await;
    melody_framework::commands::register_guild_commands(&self.core, &commands, guild_id, {
      self.core.operate_persist_commit(|persist| {
        let guild_plugins = persist.get_guild_plugins_mut(guild_id);
        operation(guild_plugins);
        Ok(guild_plugins.clone())
      }).await?
    }).await.context("failed to register commands")
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
