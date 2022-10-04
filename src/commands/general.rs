use crate::MelodyResult;
use crate::blueprint::*;

use rand::Rng;
use serenity::client::Context;
use serenity::model::permissions::Permissions;
use serenity::model::application::command::CommandType;



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
async fn ping(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let response = if rand::thread_rng().gen_bool(0.01) { "Pog" } else { "Pong" };
  BlueprintCommandResponse::new(response)
    .send(ctx, &args.interaction).await
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
        super::choice("help"),
        super::choice("connect-four")
      ]
    })
  ],
  function: help
};

#[command_attr::hook]
async fn help(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let color = super::bot_color(ctx).await;
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

  response.send(ctx, &args.interaction).await
}
