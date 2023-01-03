use crate::MelodyResult;
use crate::blueprint::*;
use crate::data::Core;

use rand::Rng;
use serenity::model::user::User;
use serenity::model::permissions::Permissions;



pub(super) const PING: BlueprintCommand = blueprint_command! {
  name: "ping",
  description: "Gets a basic response from the bot",
  usage: ["/ping"],
  examples: ["/ping"],
  allow_in_dms: true,
  arguments: [],
  function: ping
};

#[command_attr::hook]
async fn ping(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let response = if rand::thread_rng().gen_bool(0.01) { "Pog" } else { "Pong" };
  BlueprintCommandResponse::new(response)
    .send(&core, &args.interaction).await
}

pub(super) const HELP: BlueprintCommand = blueprint_command! {
  name: "help",
  description: "Gets command help",
  usage: ["/help [command]"],
  examples: ["/help", "/help connect-four"],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(String {
      name: "command",
      description: "A specific command to get help for, otherwise returns the command list",
      required: false,
      choices: [
        ("ping", "ping"),
        ("help", "help"),
        ("avatar", "avatar"),
        ("banner", "banner"),
        ("connect-four", "connect-four"),
        ("roll", "roll")
      ]
    })
  ],
  function: help
};

#[command_attr::hook]
async fn help(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let color = super::bot_color(&core).await;
  let response = match resolve_arguments::<Option<String>>(args.option_values)? {
    // User provided a command, return help for that command
    Some(command_name) => match find_command(super::APPLICATION_COMMANDS, &command_name) {
      Some(command) => BlueprintCommandResponse::with_ephemeral_embeds(vec![command_embed(command, color)]),
      None => BlueprintCommandResponse::new_ephemeral("That command does not exist")
    },
    // User provided no command, return command list
    None => {
      let permissions = args.interaction.member.as_ref()
        .and_then(|member| member.permissions)
        .unwrap_or(Permissions::all());
      BlueprintCommandResponse::with_ephemeral_embeds(vec![
        command_list_embed(super::APPLICATION_COMMANDS, permissions, color)
      ])
    }
  };

  response.send(&core, &args.interaction).await
}

pub const AVATAR: BlueprintCommand = blueprint_command! {
  name: "avatar",
  description: "Gets another user's avatar",
  usage: ["/avatar [user]"],
  examples: ["/avatar", "/avatar @Nanachi"],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(User {
      name: "user",
      description: "The user whose avatar should be retrieved, defaults to the caller if not set",
      required: false
    })
  ],
  function: avatar
};

#[command_attr::hook]
async fn avatar(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let user = resolve_arguments::<Option<User>>(args.option_values)?
    .unwrap_or_else(|| args.interaction.user.clone());
  let response = match user.avatar_url() {
    Some(avatar_url) => BlueprintCommandResponse::new_ephemeral(avatar_url),
    None => BlueprintCommandResponse::new_ephemeral("Failed to get that user's avatar")
  };

  response.send(&core, &args.interaction).await
}

pub const BANNER: BlueprintCommand = blueprint_command! {
  name: "banner",
  description: "Gets another user's banner",
  usage: ["/banner [user]"],
  examples: ["/banner", "/banner @Nanachi"],
  allow_in_dms: true,
  arguments: [
    blueprint_argument!(User {
      name: "user",
      description: "The user whose banner should be retrieved, defaults to the caller if not set",
      required: false
    })
  ],
  function: banner
};

#[command_attr::hook]
async fn banner(core: Core, args: BlueprintCommandArgs) -> MelodyResult {
  let user = resolve_arguments::<Option<User>>(args.option_values)?
    .unwrap_or_else(|| args.interaction.user.clone());
  let response = match user.banner_url() {
    Some(banner_url) => BlueprintCommandResponse::new_ephemeral(banner_url),
    None => BlueprintCommandResponse::new_ephemeral("Failed to get that user's banner")
  };

  response.send(&core, &args.interaction).await
}
