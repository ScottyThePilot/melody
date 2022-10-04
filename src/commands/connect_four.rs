use crate::{MelodyError, MelodyResult};
use crate::blueprint::*;
use crate::data::*;
use crate::feature::connect_four::ConnectFourColor;

use serenity::client::Context;
use serenity::model::user::User;
use serenity::model::mention::Mention;
use serenity::model::application::command::CommandType;

use std::fmt;



pub(super) const CONNECT_FOUR: BlueprintCommand = blueprint_command! {
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
struct PrintBoard([[Option<ConnectFourColor>; 7]; 6]);

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
struct PrintPiece(Option<ConnectFourColor>);

impl fmt::Display for PrintPiece {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(match self.0 {
      Some(ConnectFourColor::Player1) => ":red_circle:",
      Some(ConnectFourColor::Player2) => ":blue_circle:",
      None => ":black_circle:"
    })
  }
}
