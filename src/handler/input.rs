use crate::prelude::*;
use crate::data::Core;

use futures::future::BoxFuture;
use log::Level;
use melody_commander::{Command, Commands, Parsed, resolve_args};
use serenity::model::id::GuildId;

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

const COMMANDS: Commands<CommandFunction> = &[
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
  command!("inspect": [
    command!("activities": command_inspect_activities()),
    command!("config": command_inspect_config()),
    command!("persist": command_inspect_persist()),
    command!("persist-guild": command_inspect_persist_guild(guild_id: Parsed<GuildId>))
  ]),
  command!("update-yt-dlp": command_update_yt_dlp(update_to: Option<String>)),
  command!("reload": [
    command!("activities": command_reload_activities())
  ])
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

  async fn command_inspect_activities(&mut self) -> MelodyResult {
    self.core.operate_activities(async |activities| info!("{activities:#?}")).await;
    Ok(())
  }

  async fn command_inspect_config(&mut self) -> MelodyResult {
    self.core.operate_config(async |config| info!("{config:#?}")).await;
    Ok(())
  }

  async fn command_inspect_persist(&mut self) -> MelodyResult {
    self.core.operate_persist(async |persist| println!("{persist:#?}")).await;
    Ok(())
  }

  async fn command_inspect_persist_guild(&mut self, guild_id: GuildId) -> MelodyResult {
    self.core.operate_persist_guild(guild_id, async |persist_guild| {
      println!("{persist_guild:#?}");
      Ok(())
    }).await?;

    Ok(())
  }

  async fn command_update_yt_dlp(&mut self, update_to: Option<String>) -> MelodyResult {
    if let Some(yt_dlp) = self.core.state.yt_dlp.clone() {
      let update_to = update_to.as_deref().unwrap_or("latest");
      self.info(format!("Updating yt-dlp..."));
      let yt_dlp_output = yt_dlp.update(update_to).await?;
      for yt_dlp_output_line in yt_dlp_output.lines() {
        self.trace(format!("(yt-dlp): {yt_dlp_output_line:?}"));
      };
    } else {
      self.error(format!("Cannot update yt-dlp, no yt-dlp path in config"));
    };

    Ok(())
  }

  async fn command_reload_activities(&mut self) -> MelodyResult {
    self.core.reload_activities().await?;
    self.info("Reloaded data/activities.json");

    Ok(())
  }

  pub async fn line(&mut self, line: String) -> MelodyResult {
    let output = melody_commander::apply(&line, COMMANDS)?;
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
    self.core.operate_persist(async |persist| persist.get_guild_plugins(guild_id)).await
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
      self.core.operate_persist_commit(async |persist| {
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
