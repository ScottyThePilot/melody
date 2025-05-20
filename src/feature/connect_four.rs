use chrono::{DateTime, Duration, Utc};
use serenity::model::id::UserId;
use uord::UOrd;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::ops::Deref;
use std::fmt;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manager {
  challenges: HashMap<UserId, HashSet<UserId>>,
  stats: HashMap<UserId, Stats>,
  #[serde(default)]
  user_games: HashMap<UOrd<UserId>, UserGame>
}

impl Manager {
  pub fn get_stats(&self, player: UserId) -> Stats {
    self.stats.get(&player).copied().unwrap_or_default()
  }

  pub fn is_playing(&self, player: UserId) -> bool {
    self.is_playing_user(player)
  }

  /// Whether the player has a game in progress currently.
  pub fn is_playing_user(&self, player: UserId) -> bool {
    self.user_games.keys().any(|players| players.contains(&player))
  }

  /// Whether or not a given player is challenging a given opponent.
  pub fn is_challenging(&self, challenger: UserId, opponent: UserId) -> bool {
    self.challenges.get(&challenger).map_or(false, |challenges| {
      challenges.contains(&opponent)
    })
  }

  /// Attempts to delete the given challenge, returning whether or not the challenge existed.
  pub fn remove_challenge(&mut self, challenger: UserId, opponent: UserId) -> bool {
    self.challenges.get_mut(&challenger).map_or(false, |challenges| {
      challenges.remove(&opponent)
    })
  }

  /// Creates a challenge.
  pub fn create_challenge(&mut self, challenger: UserId, opponent: UserId) -> bool {
    // Cannot challenge self and cannot challenge while playing
    if challenger != opponent && !self.is_playing_user(challenger) {
      self.challenges.entry(challenger).or_default().insert(opponent)
    } else {
      false
    }
  }

  /// Accepts a challenge from the given challenger.
  pub fn accept_challenge(&mut self, challenger: UserId, opponent: UserId) -> Option<&mut UserGame> {
    // Cannot accept against self, cannot accept against a playing user, cannot accept while playing
    let valid = challenger != opponent && !self.is_playing(challenger) && !self.is_playing(opponent);
    // Challenge must also exist
    if valid && self.remove_challenge(challenger, opponent) {
      match self.user_games.entry(UOrd::new(opponent, challenger)) {
        Entry::Vacant(entry) => Some(entry.insert(UserGame::new(challenger, opponent))),
        // Previous clauses should have eliminated the possibility of this branch's existence
        Entry::Occupied(..) => unreachable!("tried to create a game that already exists")
      }
    } else {
      None
    }
  }

  pub fn get_user_game(&self, players: impl Into<UOrd<UserId>>) -> Option<&UserGame> {
    self.user_games.get(&players.into().map(|v| v))
  }

  pub fn get_user_game_mut(&mut self, players: impl Into<UOrd<UserId>>) -> Option<&mut UserGame> {
    self.user_games.get_mut(&players.into().map(|v| v))
  }

  pub fn find_user_game(&self, player: UserId) -> Option<(&UserGame, Color)> {
    self.user_games.values().find_map(|game| {
      game.player_color(player).map(|color| (game, color))
    })
  }

  pub fn find_user_game_mut(&mut self, player: UserId) -> Option<(&mut UserGame, Color)> {
    self.user_games.values_mut().find_map(|game| {
      game.player_color(player).map(|color| (game, color))
    })
  }

  /// Resigns this player's current game, if any.
  /// Counts as a loss for the resigning player and a win for their opponent.
  pub fn resign_user_game(&mut self, player: UserId) -> Option<UserGame> {
    self.user_games.keys()
      .find_map(|players| players.other(&player).copied())
      .map(|opponent| self.end_user_game(opponent, player).unwrap())
  }

  /// Concludes a game with a winner and a loser, applying win and loss stats.
  pub fn end_user_game(&mut self, winner: UserId, loser: UserId) -> Option<UserGame> {
    if let Some(game) = self.end_user_game_draw(UOrd::new(winner, loser)) {
      self.stats.entry(winner).or_default().wins += 1;
      self.stats.entry(loser).or_default().losses += 1;
      Some(game)
    } else {
      None
    }
  }

  /// Ends the game without a winner or a loser.
  pub fn end_user_game_draw(&mut self, players: impl Into<UOrd<UserId>>) -> Option<UserGame> {
    self.user_games.remove(&players.into().map(|v| v))
  }
}

impl Default for Manager {
  fn default() -> Self {
    Manager {
      challenges: HashMap::new(),
      stats: HashMap::new(),
      user_games: HashMap::new()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserGame {
  board: Board,
  #[serde(default = "Utc::now")]
  last_played: DateTime<Utc>,
  player1: UserId,
  player2: UserId
}

impl UserGame {
  pub fn new(player1: UserId, player2: UserId) -> Self {
    UserGame {
      board: Board::new(Color::Player2),
      last_played: Utc::now(),
      player1,
      player2
    }
  }

  pub fn play_move(&mut self, player: Color, column: usize) -> UserGameResult {
    if self.board.turn == player {
      self.board = match self.board.apply_move(column) {
        Some(board) => board,
        None => return UserGameResult::IllegalMove
      };

      if self.board.is_winning_position(player) {
        UserGameResult::Victory(self.board)
      } else if self.board.is_draw() {
        UserGameResult::Draw(self.board)
      } else {
        UserGameResult::Continuing(self.board)
      }
    } else {
      UserGameResult::NotYourTurn
    }
  }

  pub fn can_claim_win(&self) -> bool {
    Utc::now() - self.last_played > Duration::hours(3)
  }

  pub fn last_played(&self) -> DateTime<Utc> {
    self.last_played
  }

  pub fn current_turn_user(&self) -> UserId {
    match self.board.turn {
      Color::Player1 => self.player1,
      Color::Player2 => self.player2
    }
  }

  /// The unordered pair of players participating in this game.
  pub fn players(&self) -> UOrd<UserId> {
    UOrd::new(self.player1, self.player2)
  }

  pub fn player_color(&self, player: UserId) -> Option<Color> {
    match () {
      () if self.player1 == player => Some(Color::Player1),
      () if self.player2 == player => Some(Color::Player2),
      () => None
    }
  }
}

impl Deref for UserGame {
  type Target = Board;

  fn deref(&self) -> &Self::Target {
    &self.board
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserGameResult {
  Continuing(Board),
  Victory(Board),
  Draw(Board),
  NotYourTurn,
  IllegalMove
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
  // 6 tall x 7 wide
  matrix: [[Option<Color>; 7]; 6],
  turn: Color
}

impl Board {
  pub fn new(turn: Color) -> Self {
    Board {
      matrix: [[None; 7]; 6],
      // Player 2 (player who was challenged) goes first
      turn
    }
  }

  /// Panics if column/x >= 7
  fn apply_move(self, column: usize) -> Option<Self> {
    assert!(column < 7);
    let mut board = self;
    let row = board.matrix.iter_mut()
      .map(move |array| &mut array[column])
      .rposition(|cell| cell.is_none())?;
    board.matrix[row][column] = Some(board.turn);
    board.turn.flip();
    Some(board)
  }

  pub fn is_move_legal(&self, column: usize) -> bool {
    self.matrix.iter()
      .map(|array| &array[column])
      .any(|cell| cell.is_none())
  }

  pub fn current_turn(&self) -> Color {
    self.turn
  }

  pub fn matrix(&self) -> [[Option<Color>; 7]; 6] {
    self.matrix
  }

  /// Panics if column/x > 6 or row/y > 5, 0-based
  pub fn get(&self, column: usize, row: usize) -> Option<Color> {
    self.matrix[row][column]
  }

  /// Whether or not the game has ended inconclusively (board is full)
  pub fn is_draw(&self) -> bool {
    self.iter_board().all(|cell| cell.is_some())
  }

  /// Whether or not the game is a winning position for the given player color
  pub fn is_winning_position(&self, player: Color) -> bool {
    fn connect_four_iter<'a>(iter: impl Iterator<Item = &'a Option<Color>>, color: Color) -> bool {
      connect_four(iter.copied().collect::<Vec<Option<Color>>>(), color)
    }

    self.matrix.iter().any(|slice| connect_four(slice, player)) ||
    columns(&self.matrix).any(|iter| connect_four_iter(iter, player)) ||
    diag1(&self.matrix).any(|iter| connect_four_iter(iter, player)) ||
    diag2(&self.matrix).any(|iter| connect_four_iter(iter, player))
  }

  pub fn print<F>(self, print_piece: F) -> PrintBoard<F>
  where F: Fn(Option<Color>) -> &'static str {
    PrintBoard::new(self, print_piece)
  }

  fn iter_board(&self) -> impl Iterator<Item = Option<Color>> {
    self.matrix().into_iter().flat_map(<[Option<Color>; 7]>::into_iter)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Stats {
  pub wins: usize,
  pub losses: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Color {
  Player1,
  Player2
}

impl Color {
  pub fn other(self) -> Self {
    match self {
      Color::Player1 => Color::Player2,
      Color::Player2 => Color::Player1
    }
  }

  pub fn flip(&mut self) {
    *self = self.other();
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

fn connect_four(list: impl AsRef<[Option<Color>]>, color: Color) -> bool {
  list.as_ref().windows(4).any(|w| w == [Some(color); 4])
}

pub fn validate_column(value: i64) -> Option<usize> {
  if let Some(value @ 0..=6) = value.checked_sub(1) { Some(value as usize) } else { None }
}

#[derive(Debug, Clone, Copy)]
pub struct PrintBoard<F> {
  matrix: [[Option<Color>; 7]; 6],
  print_piece: F
}

impl<F> PrintBoard<F>
where F: Fn(Option<Color>) -> &'static str {
  pub fn new(board: Board, print_piece: F) -> Self {
    PrintBoard { matrix: board.matrix, print_piece }
  }
}

impl<F> fmt::Display for PrintBoard<F>
where F: Fn(Option<Color>) -> &'static str {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, ":one::two::three::four::five::six::seven:")?;
    for row in self.matrix {
      for piece in row {
        f.write_str((self.print_piece)(piece))?;
      };

      writeln!(f)?;
    };

    Ok(())
  }
}
