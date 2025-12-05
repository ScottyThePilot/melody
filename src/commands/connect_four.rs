use crate::prelude::*;
use crate::data::*;
use crate::utils::{Timestamp, TimestampFormat};
use super::{MelodyContext, CommandMetaData};

use melody_connect_four::*;
use serenity::model::id::UserId;



#[poise::command(
  slash_command,
  subcommands(
    "connect_four_challenge",
    "connect_four_accept",
    "connect_four_decline",
    "connect_four_play",
    "connect_four_board",
    "connect_four_resign",
    "connect_four_claim_win",
    "connect_four_stats"
  ),
  guild_only,
  rename = "connect-four",
  name_localized("en-US", "connect-four"),
  description_localized("en-US", "Play connect-four"),
  custom_data = CommandMetaData::new()
    .info_localized_concat("en-US", [
      "To begin a game, one of the players will need to challenge another via the `/connect-four challenge` subcommand.",
      "That player will then need to accept the challenge via the `/connect-four accept` subcommand.",
      "From there, play will begin, and moves may be played with the `/connect-four play` subcommand.",
      "At any time, either player may use the `/connect-four resign` subcommand to resign from the game, or use the",
      "`/connect-four board` subcommand to see the board of their current game again.",
      "If your opponent has taken more than 3 hours on a move, you may elect to claim a win and end the game with the",
      "`/connect-four claim-win` command."
    ])
    .usage_localized("en-US", [
      "/connect-four challenge <user>",
      "/connect-four accept <user>",
      "/connect-four decline <user>",
      "/connect-four play <column>",
      "/connect-four board",
      "/connect-four resign",
      "/connect-four claim-win [confirm]",
      "/connect-four stats"
    ])
    .examples_localized("en-US", [
      "/connect-four challenge @Nanachi",
      "/connect-four accept @Reg",
      "/connect-four decline @Riko",
      "/connect-four play 4"
    ])
)]
pub async fn connect_four(_ctx: MelodyContext<'_>) -> MelodyResult {
  Err(MelodyError::COMMAND_PRECONDITION_VIOLATION_ROOT_COMMAND)
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "challenge",
  name_localized("en-US", "challenge"),
  description_localized("en-US", "Challenge another user to a game of connect-four"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four challenge <user>"])
    .examples_localized("en-US", ["/connect-four challenge @Nanachi"])
)]
async fn connect_four_challenge(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "user")]
  #[description_localized("en-US", "The user to challenge")]
  user: UserId
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let opponent = user;
  let challenger = ctx.author().id;

  let response = core.operate_persist_guild_commit(guild_id, async |persist_guild| {
    Ok(match persist_guild.connect_four.create_challenge(challenger, opponent) {
      true => {
        format!(
          "{}, {} has challenged you to a game of connect-four\nUse `/connect-four accept` to accept this challenge",
          opponent.mention(), challenger.mention()
        )
      },
      false => "You cannot challenge that user at this time\n(Are you already playing a game?)".to_owned()
    })
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "accept",
  name_localized("en-US", "accept"),
  description_localized("en-US", "Accept another user's game challenge"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four accept <user>"])
    .examples_localized("en-US", ["/connect-four accept @Reg"])
)]
async fn connect_four_accept(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "user")]
  #[description_localized("en-US", "The user to accept a challenge from")]
  user: UserId
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let challenger = user;
  let player = ctx.author().id;

  let response = core.operate_persist_guild_commit(guild_id, async |persist_guild| {
    Ok(match persist_guild.connect_four.accept_challenge(challenger, player) {
      Some(game) => {
        let &player = game.players().other(&player).unwrap();
        let board = game.print(print_piece);
        let player_key = "You are :blue_circle:, your opponent is :red_circle:";

        format!("You have accepted {}'s challenge\nIt is your turn to play\n{player_key}\n\n{board}", player.mention())
      },
      None => if persist_guild.connect_four.is_playing_user(player) {
        "You must finish your current game before starting a new one!".to_owned()
      } else {
        "You do not have a pending challenge from this user".to_owned()
      }
    })
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "decline",
  name_localized("en-US", "decline"),
  description_localized("en-US", "Decline another user's game challenge"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four decline <user>"])
    .examples_localized("en-US", ["/connect-four decline @Riko"])
)]
async fn connect_four_decline(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "user")]
  #[description_localized("en-US", "The user to decline a challenge from")]
  user: UserId
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;

  let challenger = user;
  let player = ctx.author().id;

  let response = core.operate_persist_guild_commit(guild_id, async |persist_guild| {
    Ok(match persist_guild.connect_four.remove_challenge(challenger, player) {
      true => format!("You have declined a challenge from {}", challenger.mention()),
      false => "You do not have a pending challenge from this user".to_owned()
    })
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "play",
  name_localized("en-US", "play"),
  description_localized("en-US", "Place a piece on the board"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four play <column>"])
    .examples_localized("en-US", [
      "/connect-four play 1",
      "/connect-four play 4",
      "/connect-four play 7"
    ])
)]
async fn connect_four_play(
  ctx: MelodyContext<'_>,
  #[name_localized("en-US", "column")]
  #[description_localized("en-US", "Which column to place a piece")]
  #[min = 1]
  #[max = 7]
  column: i64
) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let player = ctx.author().id;

  let response = core.operate_persist_guild_commit(guild_id, async |persist_guild| {
    Ok(match persist_guild.connect_four.find_user_game_mut(player) {
      Some((game, player_color)) => {
        let &opponent = game.players().other(&player).unwrap();
        let column = melody_connect_four::validate_column(column)
          .ok_or(MelodyError::COMMAND_PRECONDITION_VIOLATION_ARGUMENTS)?;

        match game.play_move(player_color, column) {
          UserGameResult::Victory(board) => {
            let board = board.print(print_piece);
            persist_guild.connect_four.end_user_game(player, opponent);
            format!("{} has played the winning move against {}!\n\n{board}", player.mention(), opponent.mention())
          },
          UserGameResult::Continuing(board) => {
            let board = board.print(print_piece);
            format!("It is {}'s turn to play\n\n{board}", opponent.mention())
          },
          UserGameResult::Draw(board) => {
            let board = board.print(print_piece);
            persist_guild.connect_four.end_user_game_draw((player, opponent));
            format!("The game between {} and {} has ended in a draw\n\n{board}", player.mention(), opponent.mention())
          },
          UserGameResult::NotYourTurn => "It is not your turn!".to_owned(),
          UserGameResult::IllegalMove => "That move is illegal".to_owned()
        }
      },
      None => "You are not currently playing a game!".to_owned()
    })
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "board",
  name_localized("en-US", "board"),
  description_localized("en-US", "Display the board of your current game"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four board"])
    .examples_localized("en-US", ["/connect-four board"])
)]
async fn connect_four_board(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let player = ctx.author().id;

  let response = core.operate_persist_guild(guild_id, async |persist_guild| {
    Ok(match persist_guild.connect_four.find_user_game(player) {
      Some((game, _player_color)) => {
        let board = game.print(print_piece);
        let current_turn_user = game.current_turn_user();
        format!("This is your current game's board\nIt is {}'s turn to play\n\n{board}", current_turn_user.mention())
      },
      None => "You are not currently playing a game!".to_owned()
    })
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "resign",
  name_localized("en-US", "resign"),
  description_localized("en-US", "Resign your current game"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four resign"])
    .examples_localized("en-US", ["/connect-four resign"])
)]
async fn connect_four_resign(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let player = ctx.author().id;

  let response = core.operate_persist_guild_commit(guild_id, async |persist_guild| {
    Ok(match persist_guild.connect_four.resign_user_game(player) {
      Some(game) => {
        let &opponent = game.players().other(&player).unwrap();
        format!("You have resigned your connect-four game with {}", opponent.mention())
      },
      None => "You are not currently playing a game!".to_owned()
    })
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "claim-win",
  name_localized("en-US", "claim-win"),
  description_localized("en-US", "Claim a win from your opponent if they have taken more than 3 hours on their turn"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four claim-win"])
    .examples_localized("en-US", ["/connect-four claim-win"])
)]
async fn connect_four_claim_win(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let player = ctx.author().id;

  let response = core.operate_persist_guild_commit(guild_id, async |persist_guild| {
    Ok(match persist_guild.connect_four.find_user_game_mut(player) {
      Some((game, player_color)) => if game.current_turn() == player_color {
        "You cannot claim a win when it is your turn!".to_owned()
      } else {
        let timestamp = Timestamp::new(game.last_played(), TimestampFormat::Relative);
        let &opponent = game.players().other(&player).unwrap();
        if game.can_claim_win() {
          let board = game.print(print_piece);
          persist_guild.connect_four.end_user_game(player, opponent);
          format!("{} has claimed a win against {}\n\n{board}", player.mention(), opponent.mention())
        } else {
          format!("You cannot claim a win yet\nYour opponent's turn started {}", timestamp)
        }
      },
      None => "You are not currently playing a game!".to_owned()
    })
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

#[poise::command(
  slash_command,
  guild_only,
  rename = "stats",
  name_localized("en-US", "stats"),
  description_localized("en-US", "See your wins and losses for this server"),
  custom_data = CommandMetaData::new()
    .usage_localized("en-US", ["/connect-four stats"])
    .examples_localized("en-US", ["/connect-four stats"])
)]
async fn connect_four_stats(ctx: MelodyContext<'_>) -> MelodyResult {
  let core = Core::from(ctx);
  let guild_id = ctx.guild_id().ok_or(MelodyError::COMMAND_NOT_IN_GUILD)?;
  let player = ctx.author().id;

  let response = core.operate_persist_guild(guild_id, async |persist_guild| {
    let stats = persist_guild.connect_four.get_stats(player);
    Ok(format!("You have won {} games\nYou have lost {} games", stats.wins, stats.losses))
  }).await?;

  ctx.reply(response).await.context("failed to send reply")?;
  Ok(())
}

fn print_piece(piece: Option<Color>) -> &'static str {
  match piece {
    Some(Color::Player1) => ":red_circle:",
    Some(Color::Player2) => ":blue_circle:",
    None => ":black_circle:"
  }
}
