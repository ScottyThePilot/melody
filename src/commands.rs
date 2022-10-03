use crate::{MelodyError, MelodyResult};
use crate::blueprint::*;
use crate::data::*;
use crate::feature::connect_four::ConnectFourColor;
use crate::utils::Contextualize;

use itertools::Itertools;
use rand::Rng;
use serenity::client::Context;
use serenity::model::id::GuildId;
use serenity::model::permissions::Permissions;
use serenity::model::user::User;
use serenity::model::mention::Mention;
use serenity::model::application::command::{Command, CommandType};
use serenity::utils::Color;

use std::fmt;



pub const APPLICATION_COMMANDS: &[BlueprintCommand] = &[
  PING, HELP, CONNECT_FOUR
];

macro_rules! choice {
  ($l:literal) => (($l, $l));
}

pub async fn bot_color(ctx: &Context) -> Color {
  data_get_config(ctx).await.access().await.accent_color
    .or_else(|| ctx.cache.current_user_field(|me| me.accent_colour))
    .unwrap_or(serenity::utils::colours::branding::BLURPLE)
}

pub async fn register_commands(ctx: &Context, guilds: &[GuildId]) -> MelodyResult {
  let (exclusive_commands, default_commands) = APPLICATION_COMMANDS.into_iter().copied()
    .partition::<Vec<_>, _>(BlueprintCommand::is_exclusive);
  info!("Found {} commands, {} exclusive commands", default_commands.len(), exclusive_commands.len());
  Command::set_global_application_commands(&ctx, commands_builder(&default_commands))
    .await.context("failed to register commands")?;
  for &guild_id in guilds {
    let guild_name = ctx.cache.guild_field(guild_id, |guild| guild.name.clone())
      .unwrap_or_else(|| "Unknown".to_owned());
    info!("Discovered guild: {guild_name} ({guild_id})");

    if exclusive_commands.is_empty() { continue };
    let plugins = Persist::get_guild_plugins(&data_get_persist(ctx).await, guild_id).await;

    let guild_commands = exclusive_commands.iter().cloned()
      .filter(|&blueprint| blueprint.is_enabled(&plugins))
      .collect::<Vec<_>>();
    if guild_commands.is_empty() {
      info!("Clearing exclusive commands for guild {guild_name} ({guild_id}");
      guild_id.set_application_commands(&ctx, |builder| builder)
        .await.context("failed to clear guild commands")?;
    } else {
      let commands_text = guild_commands.iter().map(|blueprint| blueprint.name).join(", ");
      info!("Registering exclusive commands ({commands_text}) for guild {guild_name} ({guild_id})");
      guild_id.set_application_commands(&ctx, commands_builder(&guild_commands))
        .await.context("failed to register guild-only commands")?;
    };
  };

  Ok(())
}

const PING: BlueprintCommand = blueprint_command! {
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

const HELP: BlueprintCommand = blueprint_command! {
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
        choice!("help"),
        choice!("connect-four")
      ]
    })
  ],
  function: help
};

#[command_attr::hook]
async fn help(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let color = bot_color(ctx).await;
  let response = match resolve_arguments::<Option<String>>(args.option_values)? {
    // User provided a command, return help for that command
    Some(command_name) => match find_command(APPLICATION_COMMANDS, &command_name) {
      Some(command) => BlueprintCommandResponse::with_ephemeral_embeds(vec![command_embed(command, color)]),
      None => BlueprintCommandResponse::new_ephemeral("That command does not exist")
    },
    // User provided no command, return command list
    None => {
      let permissions = args.interaction.member.as_ref()
        .and_then(|member| member.permissions)
        .unwrap_or(Permissions::all());
      BlueprintCommandResponse::with_ephemeral_embeds(vec![
        command_list_embed(APPLICATION_COMMANDS, permissions, color)
      ])
    }
  };

  response.send(ctx, &args.interaction).await
}



const CONNECT_FOUR: BlueprintCommand = blueprint_command! {
  name: "connect-four",
  description: "Play connect-four",
  usage: [
    "/connect-four challenge <user>",
    "/connect-four accept <user>",
    "/connect-four decline <user>",
    "/connect-four play <column>",
    "/connect-four resign",
    "/connect-four stats"
  ],
  examples: [
    "/connect-four challenge @Nanachi",
    "/connect-four accept @Reg",
    "/connect-four decline @Riko",
    "/connect-four play 4"
  ],
  allow_in_dms: false,
  subcommands: [
    blueprint_subcommand! {
      name: "challenge",
      description: "Challenge another user to a game of connect-four",
      arguments: [
        blueprint_argument!(User {
          name: "user",
          description: "The user to challenge",
          required: true
        })
      ],
      function: connectfour_challenge
    },
    blueprint_subcommand! {
      name: "accept",
      description: "Accept another user's game challenge",
      arguments: [
        blueprint_argument!(User {
          name: "user",
          description: "The user to accept a challenge from",
          required: true
        })
      ],
      function: connectfour_accept
    },
    blueprint_subcommand! {
      name: "decline",
      description: "Decline another user's game challenge",
      arguments: [
        blueprint_argument!(User {
          name: "user",
          description: "The user to decline a challenge from",
          required: true
        })
      ],
      function: connectfour_decline
    },
    blueprint_subcommand! {
      name: "play",
      description: "Place a piece on the board",
      arguments: [
        blueprint_argument!(Integer {
          name: "column",
          description: "Which column to place a piece",
          required: true,
          min_value: 1,
          max_value: 7
        })
      ],
      function: connectfour_play
    },
    blueprint_subcommand! {
      name: "resign",
      description: "Resign your current game",
      arguments: [],
      function: connectfour_resign
    },
    blueprint_subcommand! {
      name: "stats",
      description: "See your wins and loss stats for this server",
      arguments: [],
      function: connectfour_stats
    }
  ]
};

#[command_attr::hook]
async fn connectfour_challenge(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::InvalidCommand)?;
  let challenger = args.interaction.user.id;
  let opponent = resolve_arguments::<User>(args.option_values)?;
  let response = data_modify_persist_guild(ctx, guild_id, |mut persist_guild| {
    Ok(match persist_guild.connect_four.create_challenge(challenger, opponent.id) {
      true => {
        format!(
          "{}, {} has challenged you to a game of connect-four\n\
          Use `/connect-four accept` to accept this challenge",
          Mention::User(opponent.id),
          Mention::User(challenger)
        )
      },
      false => "You cannot challenge that user at this time".to_owned()
    })
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(ctx, &args.interaction).await
}

#[command_attr::hook]
async fn connectfour_accept(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::InvalidCommand)?;
  let player = args.interaction.user.id;
  let challenger = resolve_arguments::<User>(args.option_values)?;
  let response = data_modify_persist_guild(ctx, guild_id, |mut persist_guild| {
    Ok(match persist_guild.connect_four.accept_challenge(challenger.id, player) {
      Some(game) => {
        let &player = game.players().other(&player).unwrap();
        let board = PrintBoard(game.board());
        let player_key = "You are :blue_circle:, your opponent is :red_circle:";

        format!("You have accepted {}'s challenge\nIt is your turn to play\n{player_key}\n\n{board}", Mention::User(player))
      },
      None => if persist_guild.connect_four.is_playing(player) {
        "You must finish your current game before starting a new one!".to_owned()
      } else {
        "You do not have a pending challenge from this user".to_owned()
      }
    })
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(ctx, &args.interaction).await
}

#[command_attr::hook]
async fn connectfour_decline(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::InvalidCommand)?;
  let player = args.interaction.user.id;
  let challenger = resolve_arguments::<User>(args.option_values)?;
  let response = data_modify_persist_guild(ctx, guild_id, |mut persist_guild| {
    Ok(match persist_guild.connect_four.remove_challenge(challenger.id, player) {
      true => format!("You have declined a challenge from {}", challenger.tag()),
      false => "You do not have a pending challenge from this user".to_owned()
    })
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(ctx, &args.interaction).await
}

#[command_attr::hook]
async fn connectfour_play(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::InvalidCommand)?;
  let player = args.interaction.user.id;
  let response = data_modify_persist_guild(ctx, guild_id, |mut persist_guild| {
    Ok(match persist_guild.connect_four.find_game_mut(player) {
      Some((game, player_color)) => {
        let &opponent = game.players().other(&player).unwrap();
        let column = resolve_arguments::<i64>(args.option_values)?;
        let column = crate::feature::connect_four::validate_column(column)
          .ok_or(MelodyError::InvalidArguments)?;

        match game.make_move(player_color, column) {
          Some(true) => {
            let board = PrintBoard(game.board());
            persist_guild.connect_four.end_game(player, opponent);
            format!("{} has played the winning move against {}!\n\n{board}", Mention::User(player), Mention::User(opponent))
          },
          Some(false) => {
            let board = PrintBoard(game.board());
            if game.is_draw() {
              persist_guild.connect_four.end_game_draw((player, opponent));
              format!("The game between {} and {} has ended in a draw\n\n{board}", Mention::User(player), Mention::User(opponent))
            } else {
              format!("It is {}'s turn to play\n\n{board}", Mention::User(opponent))
            }
          },
          None => "It is not your turn!".to_owned()
        }
      },
      None => "You are not currently playing a game!".to_owned()
    })
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(ctx, &args.interaction).await
}

#[command_attr::hook]
async fn connectfour_resign(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::InvalidCommand)?;
  let player = args.interaction.user.id;
  let response = data_modify_persist_guild(ctx, guild_id, |mut persist_guild| {
    Ok(match persist_guild.connect_four.resign(player) {
      Some(game) => {
        let &opponent = game.players().other(&player).unwrap();
        format!("You have resigned your connect-four game with {}", Mention::User(opponent))
      },
      None => "You are not currently playing a game!".to_owned()
    })
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(ctx, &args.interaction).await
}

#[command_attr::hook]
async fn connectfour_stats(ctx: &Context, args: BlueprintCommandArgs) -> MelodyResult {
  let guild_id = args.interaction.guild_id.ok_or(MelodyError::InvalidCommand)?;
  let player = args.interaction.user.id;
  let response = data_access_persist_guild(ctx, guild_id, |persist_guild| {
    let stats = persist_guild.connect_four.get_stats(player);
    Ok(format!("You have won {} games\nYou have lost {} games", stats.wins, stats.losses))
  }).await?;

  BlueprintCommandResponse::new(response)
    .send(ctx, &args.interaction).await
}



#[derive(Debug, Clone, Copy)]
pub struct PrintBoard([[Option<ConnectFourColor>; 7]; 6]);

impl fmt::Display for PrintBoard {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, ":one::two::three::four::five::six::seven:")?;
    for row in self.0 {
      for piece in row {
        write!(f, "{}", PrintPiece(piece))?;
      };

      writeln!(f)?;
    };

    Ok(())
  }
}

#[derive(Debug, Clone, Copy)]
pub struct PrintPiece(Option<ConnectFourColor>);

impl fmt::Display for PrintPiece {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(match self.0 {
      Some(ConnectFourColor::Player1) => ":red_circle:",
      Some(ConnectFourColor::Player2) => ":blue_circle:",
      None => ":black_circle:"
    })
  }
}
