use chrono::{DateTime, Duration, Utc};
use float_ord::FloatOrd;
use rand::Rng;
use rand::seq::SliceRandom;
use serenity::model::id::UserId;
use uord::UOrd;

use std::collections::{HashMap, HashSet};
use std::collections::hash_map::Entry;
use std::fmt;



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manager {
  challenges: HashMap<u64, HashSet<u64>>,
  stats: HashMap<u64, Stats>,
  games: HashMap<UOrd<u64>, Game>
}

impl Manager {
  pub fn get_stats(&self, player: UserId) -> Stats {
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
  pub fn accept_challenge(&mut self, challenger: UserId, opponent: UserId) -> Option<&mut Game> {
    // Cannot accept against self, cannot accept against a playing user, cannot accept while playing
    let valid = challenger != opponent && !self.is_playing(challenger) && !self.is_playing(opponent);
    // Challenge must also exist
    if valid && self.remove_challenge(challenger, opponent) {
      match self.games.entry(UOrd::new(opponent.0, challenger.0)) {
        Entry::Vacant(entry) => Some(entry.insert(Game::new(challenger, opponent))),
        // Previous clauses should have eliminated the possibility of this branch's existence
        Entry::Occupied(..) => unreachable!("tried to create a game that already exists")
      }
    } else {
      None
    }
  }

  pub fn get_game(&self, players: impl Into<UOrd<UserId>>) -> Option<&Game> {
    self.games.get(&players.into().map(|v| v.0))
  }

  pub fn get_game_mut(&mut self, players: impl Into<UOrd<UserId>>) -> Option<&mut Game> {
    self.games.get_mut(&players.into().map(|v| v.0))
  }

  pub fn find_game(&self, player: UserId) -> Option<(&Game, Color)> {
    self.games.values().find_map(|game| {
      game.player_color(player).map(|color| (game, color))
    })
  }

  pub fn find_game_mut(&mut self, player: UserId) -> Option<(&mut Game, Color)> {
    self.games.values_mut().find_map(|game| {
      game.player_color(player).map(|color| (game, color))
    })
  }

  /// Resigns this player's current game, if any.
  /// Counts as a loss for the resigning player and a win for their opponent.
  pub fn resign(&mut self, player: UserId) -> Option<Game> {
    self.games.keys()
      .find_map(|players| players.other(&player.0).copied())
      .map(|opponent| self.end_game(UserId(opponent), player).unwrap())
  }

  /// Concludes a game with a winner and a loser, applying win and loss stats.
  pub fn end_game(&mut self, winner: UserId, loser: UserId) -> Option<Game> {
    if let Some(game) = self.end_game_draw(UOrd::new(winner, loser)) {
      self.stats.entry(winner.0).or_default().wins += 1;
      self.stats.entry(loser.0).or_default().losses += 1;
      Some(game)
    } else {
      None
    }
  }

  /// Ends the game without a winner or a loser.
  pub fn end_game_draw(&mut self, players: impl Into<UOrd<UserId>>) -> Option<Game> {
    self.games.remove(&players.into().map(|v| v.0))
  }
}

impl Default for Manager {
  fn default() -> Self {
    Manager {
      challenges: HashMap::new(),
      stats: HashMap::new(),
      games: HashMap::new()
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
  board: Board,
  last_played: DateTime<Utc>,
  player1: u64,
  player2: u64
}

impl Game {
  pub fn new(player1: UserId, player2: UserId) -> Self {
    Game {
      board: Board::new(),
      last_played: Utc::now(),
      player1: player1.into(),
      player2: player2.into()
    }
  }

  pub fn play_move(&mut self, player: Color, column: usize) -> GameResult {
    if self.board.turn == player {
      self.board = match self.board.apply_move(column) {
        Some(board) => board,
        None => return GameResult::IllegalMove
      };

      if self.is_winning_position(player) {
        GameResult::Victory(self.board)
      } else if self.is_draw() {
        GameResult::Draw(self.board)
      } else {
        GameResult::Continuing(self.board)
      }
    } else {
      GameResult::NotYourTurn
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
      Color::Player1 => UserId(self.player1),
      Color::Player2 => UserId(self.player2)
    }
  }

  /// The unordered pair of players participating in this game
  pub fn players(&self) -> UOrd<UserId> {
    UOrd::new(self.player1, self.player2).map(UserId)
  }

  pub fn player_color(&self, player: UserId) -> Option<Color> {
    match () {
      () if self.player1 == player.0 => Some(Color::Player1),
      () if self.player2 == player.0 => Some(Color::Player2),
      () => None
    }
  }

  pub fn print<F>(&self, print_piece: F) -> PrintBoard<F>
  where F: Fn(Option<Color>) -> &'static str {
    PrintBoard::new(self.board, print_piece)
  }

  pub fn current_turn(&self) -> Color {
    self.board.current_turn()
  }

  pub fn is_draw(&self) -> bool {
    self.board.is_draw()
  }

  pub fn is_winning_position(&self, player: Color) -> bool {
    self.board.is_winning_position(player)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Board {
  // 6 tall x 7 wide
  matrix: [[Option<Color>; 7]; 6],
  turn: Color
}

impl Board {
  pub fn new() -> Self {
    Board {
      matrix: [[None; 7]; 6],
      // Player 2 (player who was challenged) goes first
      turn: Color::Player2
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
    board.turn = board.turn.other();
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

  fn iter_legal_moves(&self) -> impl Iterator<Item = usize> + '_ {
    (0..7).filter(|&column| self.is_move_legal(column))
  }

  fn iter_potential_positions(&self) -> impl Iterator<Item = (usize, Self)> + '_ {
    (0..7).filter_map(|column| Some((column, self.apply_move(column)?)))
  }

  /// Picks a move out of all possible moves based on one of four difficulty presets, and a random number generator.
  pub fn evaluate_move_difficulty(&self, rng: &mut impl Rng, difficulty: Difficulty) -> Option<(usize, f32)> {
    let depth = difficulty.evaluation_depth();
    let (min, max, losing_move_discount) = match difficulty.parameters() {
      None => return self.evaluate_best_move(depth),
      Some(parameters) => parameters
    };

    let mut moveset = self.evaluate_moves(depth);
    if moveset.is_empty() { return None };
    if moveset.len() == 1 { return Some(moveset[0]) };
    // Count the number of moves that will allow the other player to win on the next turn
    let losing_moves = moveset.iter().filter(|&&(_, eval)| eval == -1.0).count().min(6);
    let remaining_moves = moveset.len().checked_sub(losing_move_discount.min(losing_moves));
    moveset.truncate(match remaining_moves {
      // If the proposed discount by would leave 1 or 0 moves, just take the best move
      Some(0 | 1) | None => return Some(moveset[0]),
      // The returned value should never be 0
      Some(remaining_moves) => remaining_moves
    });

    let min = f32::ceil(min * moveset.len() as f32) as usize;
    let max = f32::floor(max * moveset.len() as f32) as usize;
    let m = moveset[min..max].choose(rng).cloned()
      .unwrap_or_else(|| moveset[0]);
    Some(m)
  }

  /// Evaluate the best move at the given position.
  fn evaluate_best_move(&self, depth: usize) -> Option<(usize, f32)> {
    let color = self.current_turn();
    self.iter_potential_positions()
      .map(|(column, game)| (column, game.evaluate_position(depth, color)))
      .max_by_key(|&(_, value)| FloatOrd(value))
  }

  /// Evaluate all possible moves, returning them in order with their evaluation scores.
  fn evaluate_moves(&self, depth: usize) -> Vec<(usize, f32)> {
    let color = self.current_turn();
    let mut evaluated_moves = self.iter_potential_positions()
      .map(|(column, game)| (column, game.evaluate_position(depth, color)))
      .collect::<Vec<(usize, f32)>>();
    evaluated_moves.sort_unstable_by_key(|&(_, value)| FloatOrd(-value));
    evaluated_moves
  }

  /// Recursively evaluates a board in reference to the given player
  /// based on the moves available to the current player
  fn evaluate_position(&self, depth: usize, color: Color) -> f32 {
    let turn = self.current_turn();
    let turn_value = if turn == color { 1.0 } else { -1.0 };

    if depth == 0 {
      return match self.is_winning_position(turn) {
        true => turn_value,
        false => 0.0
      };
    };

    let mut sum = 0.0;
    let mut count: usize = 0;
    for column in self.iter_legal_moves() {
      let game = self.apply_move(column).unwrap();
      if game.is_winning_position(turn) {
        return turn_value;
      } else {
        sum += game.evaluate_position(depth - 1, color);
        count += 1;
      };
    };

    sum / count as f32
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Difficulty {
  // Always picks the best possible move
  // Evaluation depth 7
  Maxmimum,
  // Only plays moves above 50%
  // Never plays losing moves
  // Evaluation depth 5
  Hard,
  // Only plays moves above 25% and belov 75%
  // Eliminates 4 losing moves from moveset
  // Evaluation depth 4
  Medium,
  // Only plays moves below 50%
  // Eliminates 2 losing moves from moveset
  // Evaluation depth 4
  Easy
}

impl Difficulty {
  pub fn evaluation_depth(self) -> usize {
    match self {
      Difficulty::Maxmimum => 7,
      Difficulty::Hard => 5,
      Difficulty::Medium => 4,
      Difficulty::Easy => 4
    }
  }

  pub fn parameters(self) -> Option<(f32, f32, usize)> {
    match self {
      Difficulty::Maxmimum => None,
      Difficulty::Hard => Some((0.00, 0.50, 6)),
      Difficulty::Medium => Some((0.25, 0.75, 4)),
      Difficulty::Easy => Some((0.50, 0.00, 2))
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
  Continuing(Board),
  Victory(Board),
  Draw(Board),
  NotYourTurn,
  IllegalMove
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
