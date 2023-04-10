extern crate chess_engine;
extern crate chessboard_render;

use chess_engine::*;

fn main() {
  let board = Board::default();
  let board = continuing(board.play_move(Move::Piece(F2, F3)));
  let board = continuing(board.play_move(Move::Piece(E7, E6)));
  let board = continuing(board.play_move(Move::Piece(E2, E3)));
  let board = continuing(board.play_move(Move::Piece(D8, H4)));
  let img = chessboard_render::render_board(&board, !board.get_turn_color(), &[D8, H4]);
  chessboard_render::save_image(&img, "board.png").unwrap();
}

fn continuing(result: GameResult) -> Board {
  if let GameResult::Continuing(board) = result { board } else { panic!() }
}
