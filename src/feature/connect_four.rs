use serenity::model::id::UserId;
use serenity::model::user::User;
use uord::UOrd;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectFourManager {
  challenges: HashMap<u64, HashSet<u64>>,
  stats: HashMap<u64, ConnectFourStats>,
  games: HashMap<UOrd<u64>, ConnectFourGame>
}

impl ConnectFourManager {
  pub fn get_stats(&self, player: UserId) -> ConnectFourStats {
    self.stats.get(&player.0).copied().unwrap_or_default()
  }

  /// Whether the player has a game in progress currently.
  pub fn is_playing(&self, player: UserId) -> bool {
    self.games.keys().any(|players| players.contains(&player.0))
  }

  /// Whether the two players are currently playing against each other.
  pub fn are_playing(&self, players: impl Into<UOrd<UserId>>) -> bool {
    self.games.contains_key(&players.into().map(|v| v.0))
  }

  /// Whether or not a given player is challenging a given opponent.
  pub fn is_challenging(&self, challenger: UserId, opponent: UserId) -> bool {
    self.challenges.get(&challenger.0).map_or(false, |challenges| {
      challenges.contains(&opponent.0)
    })
  }

  /// Attempts to delete the given challenge, returning whether or not the challenge existed.
  pub fn remove_challenge(&mut self, challenger: UserId, opponent: UserId) -> bool {
    self.challenges.get_mut(&challenger.0).map_or(false, |challenges| {
      challenges.remove(&opponent.0)
    })
  }

  /// Creates a challenge.
  pub fn create_challenge(&mut self, challenger: UserId, opponent: UserId) -> bool {
    // Cannot challenge self and cannot challenge while playing
    if challenger != opponent && !self.is_playing(challenger) {
      self.challenges.entry(challenger.0).or_default().insert(opponent.0)
    } else {
      false
    }
  }

  /// Accepts a challenge from the given challenger.
  pub fn accept_challenge(&mut self, challenger: UserId, opponent: UserId) -> Option<&mut ConnectFourGame> {
    // Cannot accept against self, cannot accept against a playing user, cannot accept while playing
    let valid = challenger != opponent && !self.is_playing(challenger) && !self.is_playing(opponent);
    // Challenge must also exist
    if valid && self.remove_challenge(challenger, opponent) {
      match self.games.entry(UOrd::new(opponent.0, challenger.0)) {
        Entry::Vacant(entry) => Some(entry.insert(ConnectFourGame::new(challenger, opponent))),
        // Previous clauses should have eliminated the possibility of this branch's existence
        Entry::Occupied(..) => unreachable!("tried to create a game that already exists")
      }
    } else {
      None
    }
  }

  pub fn get_game(&self, players: impl Into<UOrd<UserId>>) -> Option<&ConnectFourGame> {
    self.games.get(&players.into().map(|v| v.0))
  }

  pub fn get_game_mut(&mut self, players: impl Into<UOrd<UserId>>) -> Option<&mut ConnectFourGame> {
    self.games.get_mut(&players.into().map(|v| v.0))
  }

  pub fn find_game(&self, player: UserId) -> Option<(&ConnectFourGame, ConnectFourColor)> {
    self.games.values().find_map(|game| {
      game.get_player_color(player).map(|color| (game, color))
    })
  }

  pub fn find_game_mut(&mut self, player: UserId) -> Option<(&mut ConnectFourGame, ConnectFourColor)> {
    self.games.values_mut().find_map(|game| {
      game.get_player_color(player).map(|color| (game, color))
    })
  }

  /// Resigns this player's current game, if any.
  /// Counts as a loss for the resigning player and a win for their opponent.
  pub fn resign(&mut self, player: UserId) -> Option<ConnectFourGame> {
    self.games.keys()
      .find_map(|players| players.other(&player.0).copied())
      .map(|opponent| self.end_game(UserId(opponent), player).unwrap())
  }

  /// Concludes a game with a winner and a loser, applying win and loss stats.
  pub fn end_game(&mut self, winner: UserId, loser: UserId) -> Option<ConnectFourGame> {
    if let Some(game) = self.end_game_draw(UOrd::new(winner, loser)) {
      self.stats.entry(winner.0).or_default().wins += 1;
      self.stats.entry(loser.0).or_default().losses += 1;
      Some(game)
    } else {
      None
    }
  }

  /// Ends the game without a winner or a loser.
  pub fn end_game_draw(&mut self, players: impl Into<UOrd<UserId>>) -> Option<ConnectFourGame> {
    self.games.remove(&players.into().map(|v| v.0))
  }
}

impl Default for ConnectFourManager {
  fn default() -> Self {
    ConnectFourManager {
      challenges: HashMap::new(),
      stats: HashMap::new(),
      games: HashMap::new()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectFourGame {
  turn: ConnectFourColor,
  // 6 tall x 7 wide
  board: [[Option<ConnectFourColor>; 7]; 6],
  player1: u64,
  player2: u64
}

impl ConnectFourGame {
  pub fn new(player1: UserId, player2: UserId) -> Self {
    ConnectFourGame {
      // Player 2 (player who was challenged) goes first
      turn: ConnectFourColor::Player2,
      board: [[None; 7]; 6],
      player1: player1.into(),
      player2: player2.into()
    }
  }

  /// Returns an option describing whether or not the move succeeded,
  /// which contains a bool describing whether or not this move won the game
  pub fn make_move(&mut self, player: ConnectFourColor, column: usize) -> Option<bool> {
    if column >= 7 || self.turn != player { return None };
    let row = self.board.iter_mut()
      .map(move |array| &mut array[column])
      .rposition(|cell| cell.is_none())?;
    self.board[row][column] = Some(player);
    self.turn = player.other();
    Some(self.check_for_win(player))
  }

  pub fn is_player_turn(&self, player: UserId) -> bool {
    self.current_turn_user() == player
  }

  /// Whether or not the game has ended inconclusively (board is full)
  pub fn is_draw(&self) -> bool {
    self.iter_board().all(|cell| cell.is_some())
  }

  pub fn current_turn_user(&self) -> UserId {
    match self.turn {
      ConnectFourColor::Player1 => UserId(self.player1),
      ConnectFourColor::Player2 => UserId(self.player2)
    }
  }

  pub fn players(&self) -> UOrd<UserId> {
    UOrd::new(self.player1, self.player2).map(UserId)
  }

  /// Panics if column/x >= 7 or row/y >= 6, 0-based
  pub fn get(&self, column: usize, row: usize) -> Option<ConnectFourColor> {
    self.board[row][column]
  }

  pub fn get_player_color(&self, player: UserId) -> Option<ConnectFourColor> {
    match () {
      () if self.player1 == player.0 => Some(ConnectFourColor::Player1),
      () if self.player2 == player.0 => Some(ConnectFourColor::Player2),
      () => None
    }
  }

  pub fn board(&self) -> [[Option<ConnectFourColor>; 7]; 6] {
    self.board
  }

  pub fn count(&self) -> usize {
    self.iter_board().flatten().count()
  }

  pub fn check_for_win(&self, player: ConnectFourColor) -> bool {
    fn connect_four_iter<'a>(iter: impl Iterator<Item = &'a Option<ConnectFourColor>>, color: ConnectFourColor) -> bool {
      connect_four(iter.copied().collect::<Vec<Option<ConnectFourColor>>>(), color)
    }

    self.board.iter().any(|slice| connect_four(slice, player)) ||
    columns(&self.board).any(|iter| connect_four_iter(iter, player)) ||
    diag1(&self.board).any(|iter| connect_four_iter(iter, player)) ||
    diag2(&self.board).any(|iter| connect_four_iter(iter, player))
  }

  pub fn is_playing(&self, player: UserId) -> bool {
    self.player1 == player.0 || self.player2 == player.0
  }

  pub fn display<'a>(&'a self, player1: &'a User, player2: &'a User) -> DisplayConnectFourGame<'a> {
    DisplayConnectFourGame { game: self, player1, player2 }
  }

  pub fn iter_board(&self) -> impl Iterator<Item = Option<ConnectFourColor>> {
    self.board().into_iter().flat_map(<[Option<ConnectFourColor>; 7]>::into_iter)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConnectFourStats {
  pub wins: usize,
  pub losses: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ConnectFourColor {
  Player1,
  Player2
}

impl ConnectFourColor {
  pub fn other(self) -> Self {
    match self {
      ConnectFourColor::Player1 => ConnectFourColor::Player2,
      ConnectFourColor::Player2 => ConnectFourColor::Player1
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayConnectFourGame<'a> {
  game: &'a ConnectFourGame,
  player1: &'a User,
  player2: &'a User
}

impl<'a> fmt::Display for DisplayConnectFourGame<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{} playing against {}, it is {}'s turn for {} turns total",
      self.player1.tag(),
      self.player2.tag(),
      match self.game.turn {
        ConnectFourColor::Player1 => self.player1.tag(),
        ConnectFourColor::Player2 => self.player2.tag()
      },
      self.game.count()
    )
  }
}



fn diag1<const W: usize, const H: usize, T>(array: &[[T; W]; H])
-> impl Iterator<Item = impl Iterator<Item = &T>> {
  (0..=(W + H - 2)).map(move |k| {
    (0..=k).filter_map(move |j| {
      let i = k - j;
      (i < H && j < W).then(|| &array[i][j])
    })
  })
}

fn diag2<const W: usize, const H: usize, T>(array: &[[T; W]; H])
-> impl Iterator<Item = impl Iterator<Item = &T>> {
  (0..=(W + H - 2)).map(move |k| {
    (0..=k).filter_map(move |j| {
      let i = k - j;
      (i < H && j < W).then(|| &array[H - i - 1][j])
    })
  })
}

fn columns<const W: usize, const H: usize, T>(array: &[[T; W]; H])
-> impl Iterator<Item = impl Iterator<Item = &T>> {
  (0..W).map(move |x| (0..H).map(move |y| &array[y][x]))
}

fn connect_four(list: impl AsRef<[Option<ConnectFourColor>]>, color: ConnectFourColor) -> bool {
  list.as_ref().windows(4).any(|w| w == [Some(color); 4])
}

pub fn validate_column(value: i64) -> Option<usize> {
  if let Some(value @ 0..=6) = value.checked_sub(1) { Some(value as usize) } else { None }
}
