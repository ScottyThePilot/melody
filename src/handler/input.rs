use crate::prelude::*;
use crate::data::Core;

use futures::future::BoxFuture;
use log::Level;
use melody_commander::{Command, Commands, Parsed, resolve_args};
use serenity::model::id::GuildId;

macro_rules! command_function {
  ($function:ident(..)) => (
    |agent, remaining_args| Box::pin(async move {
      agent.$function(remaining_args).await
    })
  );
  ($function:ident($($arg:ident: $Arg:ty),* $(,)?)) => (
    |agent, remaining_args| Box::pin(async move {
      let ($($arg,)*) = resolve_args::<($($Arg,)*)>(&remaining_args)?;
      agent.$function($($arg,)*).await
    })
  );
}

macro_rules! command {
  (
    name: $name:expr,
    description: $description:expr,
    usage: $usage:expr,
    target: $function:ident($($tt:tt)*)
  ) => (
    Command::new_target(
      $name,
      CommandHelp { description: $description, usage: $usage },
      command_function!($function($($tt)*))
    )
  );
  (
    name: $name:expr,
    description: $description:expr,
    usage: $usage:expr,
    group: [$($group_item:expr),* $(,)?]
  ) => (
    Command::new_group(
      $name,
      CommandHelp { description: $description, usage: $usage },
      &[$($group_item),*]
    )
  );
}

type CommandFunction = for<'a> fn(&'a mut InputAgent, Box<[String]>) -> BoxFuture<'a, MelodyResult>;

const COMMANDS: Commands<CommandFunction, CommandHelp> = &[
  command!{
    name: "help",
    description: "Retrieves help for the given command or subcommand",
    usage: "help <command-path>...",
    target: command_help(..)
  },
  command!{
    name: "stop",
    description: "Stops the bot",
    usage: "stop",
    target: command_stop()
  },
  // a few months ago, i was using tmux, and typing `stop` did not immediately kill the tmux session,
  // now it does, and i can't find any resources on changing the magic words it uses, so i get to
  // pull out the thesaurus to work around this...
  command!{
    name: "halt",
    description: "Stops the bot",
    usage: "halt",
    target: command_stop()
  },
  command!{
    name: "plugin",
    description: "Command group for plugin utilities",
    usage: "plugin <list|enable|disable>",
    group: [
      command!{
        name: "list",
        description: "Lists the plugins that are enabled for the given guild",
        usage: "plugin list <guild_id: u64>",
        target: command_plugin_list(guild_id: Parsed<GuildId>)
      },
      command!{
        name: "enable",
        description: "Enables a plugin in the given guild",
        usage: "plugin enable <plugin: string> <guild_id: u64>",
        target: command_plugin_enable(plugin: String, guild_id: Parsed<GuildId>)
      },
      command!{
        name: "disable",
        description: "Disables a plugin in the given guild",
        usage: "plugin disable <plugin: string> <guild_id: u64>",
        target: command_plugin_disable(plugin: String, guild_id: Parsed<GuildId>)
      }
    ]
  },
  command!{
    name: "inspect",
    description: "Command group for data store inspection utilities",
    usage: "inspect <activities|config|persist|persist-guild>",
    group: [
      command!{
        name: "activities",
        description: "Inspects the current state of the global 'activities' data store",
        usage: "inspect activities",
        target: command_inspect_activities()
      },
      command!{
        name: "config",
        description: "Inspects the current state of the global 'config' data store",
        usage: "inspect config",
        target: command_inspect_config()
      },
      command!{
        name: "persist",
        description: "Inspects the current state of the global 'persist' data store",
        usage: "inspect persist",
        target: command_inspect_persist()
      },
      command!{
        name: "persist-guild",
        description: "Inspects the current state of a guild's 'persist-guild' data store, given its respective guild-id",
        usage: "inspect persist-guild <guild_id: u64>",
        target: command_inspect_persist_guild(guild_id: Parsed<GuildId>)
      }
    ]
  },
  command!{
    name: "reload",
    description: "Command group for data store reloading utilities",
    usage: "reload <activities>",
    group: [
      command!{
        name: "activities",
        description: "Reloads the global 'activities' data store from disk",
        usage: "reload activities",
        target: command_reload_activities()
      }
    ]
  },
  command!{
    name: "update-yt-dlp",
    description: "Updates the version of yt-dlp used by the bot",
    usage: "update-yt-dlp",
    target: command_update_yt_dlp(update_to: Option<String>)
  }
];

#[derive(Debug, Clone, Copy)]
pub struct CommandHelp {
  pub description: &'static str,
  pub usage: &'static str
}

#[derive(Debug, Clone)]
pub struct InputAgent {
  core: Core,
  allowed_plugins: Option<HashSet<String>>,
  output: InputAgentOutput
}

impl InputAgent {
  pub fn new(core: impl Into<Core>) -> Self {
    InputAgent {
      core: core.into(),
      allowed_plugins: None,
      output: InputAgentOutput::new()
    }
  }

  async fn get_allowed_plugins(&mut self) -> &HashSet<String> {
    get_or_insert_with_async(&mut self.allowed_plugins, async || {
      use crate::data::MelodyFrameworkKey;
      let commands = self.core.get::<MelodyFrameworkKey>().await.read_commands_owned().await;
      let allowed_plugins = commands.iter()
        .filter_map(|command| command.category.as_deref())
        .chain(crate::commands::HARDCODED_PLUGINS.into_iter().copied())
        .map(str::to_owned)
        .collect::<HashSet<String>>();
      allowed_plugins
    }).await
  }

  async fn command_stop(&mut self) -> MelodyResult {
    self.output.info("Shutdown triggered");
    self.core.trigger_shutdown().await;
    Ok(())
  }

  async fn command_plugin_list(&mut self, guild_id: GuildId) -> MelodyResult {
    let plugins = self.plugin_list(guild_id).await;
    self.output.info(format!("Plugins for guild ({guild_id}): {}", plugins.iter().join(", ")));
    Ok(())
  }

  async fn command_plugin_enable(&mut self, plugin: String, guild_id: GuildId) -> MelodyResult {
    if self.get_allowed_plugins().await.contains(&plugin) {
      self.plugin_enable(&plugin, guild_id).await?;
      self.output.info(format!("Enabled plugin {plugin} for guild ({guild_id})"));
    } else {
      self.output.info(format!("No such plugin {plugin}"));
    };

    Ok(())
  }

  async fn command_plugin_disable(&mut self, plugin: String, guild_id: GuildId) -> MelodyResult {
    if self.get_allowed_plugins().await.contains(&plugin) {
      self.plugin_disable(&plugin, guild_id).await?;
      self.output.info(format!("Disabled plugin {plugin} for guild ({guild_id})"));
    } else {
      self.output.info(format!("No such plugin {plugin}"));
    };

    Ok(())
  }

  async fn command_inspect_activities(&mut self) -> MelodyResult {
    self.core.operate_activities(async |activities| self.output.info(format!("{activities:#?}"))).await;
    Ok(())
  }

  async fn command_inspect_config(&mut self) -> MelodyResult {
    self.core.operate_config(async |config| self.output.info(format!("{config:#?}"))).await;
    Ok(())
  }

  async fn command_inspect_persist(&mut self) -> MelodyResult {
    self.core.operate_persist(async |persist| self.output.info(format!("{persist:#?}"))).await;
    Ok(())
  }

  async fn command_inspect_persist_guild(&mut self, guild_id: GuildId) -> MelodyResult {
    self.core.operate_persist_guild(guild_id, async |persist_guild| {
      self.output.info(format!("{persist_guild:#?}"));
      Ok(())
    }).await?;

    Ok(())
  }

  async fn command_update_yt_dlp(&mut self, update_to: Option<String>) -> MelodyResult {
    if let Some(yt_dlp) = self.core.state.yt_dlp.clone() {
      let update_to = update_to.as_deref().unwrap_or("latest");
      self.output.info(format!("Updating yt-dlp..."));
      let yt_dlp_output = yt_dlp.update(update_to).await?;
      for yt_dlp_output_line in yt_dlp_output.lines() {
        self.output.trace(format!("(yt-dlp): {yt_dlp_output_line:?}"));
      };
    } else {
      self.output.error(format!("Cannot update yt-dlp, no yt-dlp path in config"));
    };

    Ok(())
  }

  async fn command_reload_activities(&mut self) -> MelodyResult {
    self.core.reload_activities().await?;
    self.output.info("Reloaded data/activities.json");

    Ok(())
  }

  async fn command_help(&mut self, args: Box<[String]>) -> MelodyResult {
    if args.is_empty() {
      let list = COMMANDS.iter().map(|command| command.name).collect::<Vec<&str>>();
      self.output.info(format!("Available commands: {list:?}"));
    } else {
      let command = melody_commander::find_command(&args, COMMANDS)?;
      let mut message = format!("Command {}", command.name);
      message.push_str(&format!("\n\t- Description: {}", command.help.description));
      if let Some(subcommands) = command.subcommands() {
        let list = subcommands.iter().map(|command| command.name).collect::<Vec<&str>>();
        message.push_str(&format!("\n\t- Subcommands: {list:?}"));
      } else {
        message.push_str(&format!("\n\t- Usage: {}", command.help.usage));
      };
      self.output.info(message);
    };

    Ok(())
  }

  pub async fn line(&mut self, line: String) -> MelodyResult {
    let output = melody_commander::apply(&line, COMMANDS)?;
    (output.target)(self, output.remaining_args).await?;
    Ok(())
  }

  #[inline]
  pub fn output(&self) -> &InputAgentOutput {
    &self.output
  }

  #[inline]
  pub fn output_mut(&mut self) -> &mut InputAgentOutput {
    &mut self.output
  }

  #[inline]
  pub fn into_output(self) -> InputAgentOutput {
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
}

#[derive(Debug, Clone, Default)]
pub struct InputAgentOutput {
  messages: Vec<(Level, String)>
}

impl InputAgentOutput {
  pub fn new() -> Self {
    Self { messages: Vec::new() }
  }

  #[inline]
  pub fn messages(&self) -> &[(Level, String)] {
    &self.messages
  }

  #[inline]
  pub fn into_messages(self) -> Vec<(Level, String)> {
    self.messages
  }

  #[allow(unused)]
  pub fn log(&mut self, level: Level, message: impl Into<String>) {
    let message = message.into();
    log!(level, "{message}");
    self.messages.push((level, message));
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

async fn get_or_insert_with_async<T>(option: &mut Option<T>, f: impl AsyncFnOnce() -> T) -> &mut T {
  // stupid hack because borrow checker is stupid
  if option.is_some() {
    unsafe { option.as_mut().unwrap_unchecked() }
  } else {
    option.insert(f().await)
  }
}
