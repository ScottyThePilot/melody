use crate::MelodyResult;
use crate::data::Core;
use crate::utils::{Loggable, List};

use serenity::model::id::GuildId;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedReceiver as MpscReceiver;

use std::collections::HashSet;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;



pub async fn input_task(input: Arc<Mutex<MpscReceiver<String>>>, core: Core) {
  let agent = InputAgent::new(core);
  let mut input = input.lock().await;
  while let Some(line) = input.recv().await {
    agent.line(line).await.log();
  };
}

#[derive(Debug)]
enum InputCommand<'a> {
  Stop,
  Restart,
  PluginList(GuildId),
  PluginEnable(&'a str, GuildId),
  PluginDisable(&'a str, GuildId)
}

#[derive(Debug, Clone)]
pub(super) struct InputAgent {
  pub core: Core
}

impl InputAgent {
  pub(super) fn new(core: impl Into<Core>) -> Self {
    InputAgent { core: core.into() }
  }

  fn get_command<'a>(&self, mut args: impl Iterator<Item = &'a str>) -> Result<InputCommand<'a>, InputError<'a>> {
    match next(&mut args)? {
      "stop" => Ok(InputCommand::Stop),
      "restart" => Ok(InputCommand::Restart),
      "plugin" | "plugins" => match next(&mut args)? {
        "list" => {
          let guild_id = parse(next(&mut args)?)?;
          Ok(InputCommand::PluginList(GuildId(guild_id)))
        },
        "enable" => {
          let plugin = next(&mut args)?;
          let guild_id = parse(next(&mut args)?)?;
          Ok(InputCommand::PluginEnable(plugin, GuildId(guild_id)))
        },
        "disable" => {
          let plugin = next(&mut args)?;
          let guild_id = parse(next(&mut args)?)?;
          Ok(InputCommand::PluginDisable(plugin, GuildId(guild_id)))
        },
        unknown => Err(InputError::UnknownCommand(unknown))
      },
      unknown => Err(InputError::UnknownCommand(unknown))
    }
  }

  pub(super) async fn line(&self, line: String) -> MelodyResult {
    let line = line.to_lowercase();
    match self.get_command(line.split_whitespace()) {
      Err(err) => error!("{err}"),
      Ok(input_command) => match input_command {
        InputCommand::Stop => {
          info!("Shutdown triggered");
          self.core.trigger_shutdown().await;
        },
        InputCommand::Restart => {
          info!("Restart triggered");
          self.core.trigger_shutdown_restart().await;
        },
        InputCommand::PluginList(guild_id) => {
          let plugins = self.plugin_list(guild_id).await;
          info!("Plugins for guild ({guild_id}): {}", List(&plugins));
        },
        InputCommand::PluginEnable(plugin, guild_id) => {
          self.plugin_enable(plugin, guild_id).await?;
          info!("Enabled plugin {plugin} for guild ({guild_id})");
        },
        InputCommand::PluginDisable(plugin, guild_id) => {
          self.plugin_disable(plugin, guild_id).await?;
          info!("Disabled plugin {plugin} for guild ({guild_id})");
        }
      }
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

fn parse<'a, T>(value: &'a str) -> Result<T, InputError<'a>>
where T: FromStr, T::Err: Error {
  value.parse::<T>().map_err(|err| {
    InputError::FailedParsing(value, err.to_string())
  })
}

fn next<'a>(args: &mut impl Iterator<Item = &'a str>) -> Result<&'a str, InputError<'a>> {
  args.next().ok_or_else(|| InputError::UnexpectedEndOfInput)
}

#[derive(Debug, Error)]
enum InputError<'a> {
  #[error("Unknown command {0}")]
  UnknownCommand(&'a str),
  #[error("Failed to parse {0:?}: {1}")]
  FailedParsing(&'a str, String),
  #[error("Unexpected end of input")]
  UnexpectedEndOfInput
}
