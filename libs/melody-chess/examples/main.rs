extern crate melody_chess;

use melody_chess::shakmaty::*;

use std::fs::File;
use std::io::BufWriter;

fn main() {
  let game = Chess::new();

  let game = game.play(Move::Normal {
    role: Role::Pawn,
    from: Square::F2,
    to: Square::F3,
    capture: None,
    promotion: None
  }).unwrap();

  let game = game.play(Move::Normal {
    role: Role::Pawn,
    from: Square::E7,
    to: Square::E6,
    capture: None,
    promotion: None
  }).unwrap();

  let game = game.play(Move::Normal {
    role: Role::Pawn,
    from: Square::E2,
    to: Square::E3,
    capture: None,
    promotion: None
  }).unwrap();

  let game = game.play(Move::Normal {
    role: Role::Queen,
    from: Square::D8,
    to: Square::H4,
    capture: None,
    promotion: None
  }).unwrap();

  let color = game.turn().other();

  let assets = melody_chess::render::Assets::load();
  let img = melody_chess::render::render_board(
    &game, color,
    &[Square::D8, Square::H4],
    ["White", "Black"],
    &assets
  );

  let writer = BufWriter::new(File::create("board.png").unwrap());
  melody_chess::render::encode_image_rgb(&img, writer).unwrap();
}
