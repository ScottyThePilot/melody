extern crate chessboard_render;

use chessboard_render::shakmaty::*;

use std::fs::File;
use std::io::BufWriter;

fn main() {
  let game = Chess::new();

  let game = game.play(&Move::Normal {
    role: Role::Pawn,
    from: Square::F2,
    to: Square::F3,
    capture: None,
    promotion: None
  }).unwrap();

  let game = game.play(&Move::Normal {
    role: Role::Pawn,
    from: Square::E7,
    to: Square::E6,
    capture: None,
    promotion: None
  }).unwrap();

  let game = game.play(&Move::Normal {
    role: Role::Pawn,
    from: Square::E2,
    to: Square::E3,
    capture: None,
    promotion: None
  }).unwrap();

  let game = game.play(&Move::Normal {
    role: Role::Queen,
    from: Square::D8,
    to: Square::H4,
    capture: None,
    promotion: None
  }).unwrap();

  let color = game.turn().other();

  let img = chessboard_render::render_board(&game, color, &[Square::D8, Square::H4]);
  let writer = BufWriter::new(File::create("board.png").unwrap());
  chessboard_render::encode_image_rgb(&img, writer).unwrap();
}
