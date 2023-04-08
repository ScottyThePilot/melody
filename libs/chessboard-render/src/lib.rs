use chess_engine::{Board, Color, Piece, Position};
use image::{GenericImageView, GrayAlphaImage, ImageResult, LumaA, Pixel, Rgb, RgbImage};
use once_cell::sync::OnceCell;

type GrayAlphaSubImage<'a> = image::SubImage<&'a GrayAlphaImage>;
type Pos = (u32, u32);

const BOARD_FULL: u32 = BOARD + 64;
const BOARD: u32 = 512;
const TILE: u32 = 64;
const EDGE: u32 = 32;

const DARK: Rgb<u8> = Rgb([0xb5, 0x88, 0x63]);
const LIGHT: Rgb<u8> = Rgb([0xf0, 0xd9, 0xb5]);
const NEUTRAL: Rgb<u8> = Rgb([0xc9, 0xa3, 0x7e]);

const RANKS: Pos = (0, EDGE);
const FILES: Pos = (EDGE, BOARD + EDGE);

pub fn init() {
  Assets::instance();
}

pub fn render_board(board: &Board, side: Color) -> RgbImage {
  let assets = Assets::instance();
  let mut img = RgbImage::from_pixel(BOARD_FULL, BOARD_FULL, NEUTRAL);
  // get the correct textures for the files and ranks markings and copy them to the image buffer
  copy(&mut img, assets.get_files(side), FILES);
  copy(&mut img, assets.get_ranks(side), RANKS);
  for bx in 0u32..8u32 {
    for by in 0u32..8u32 {
      // the board is flipped vertically, because of the way chess is
      // if the player is black, it is flipped vertically (again), and horizontally
      let (x, y) = match side {
        Color::Black => (7 - bx, by),
        Color::White => (bx, 7 - by)
      };

      let pos = (x * TILE + EDGE, y * TILE + EDGE);
      // the chess board always has dark squares in the bottom left and top right
      let color = if bx % 2 == by % 2 { DARK } else { LIGHT };
      fill_square(&mut img, color, pos, TILE);
      if let Some(piece) = board.get_piece(Position::new(by as i32, bx as i32)) {
        copy(&mut img, assets.get_piece(piece), pos);
      };
    };
  };

  img
}

pub fn encode_image<W: std::io::Write>(img: &RgbImage, writer: W) -> ImageResult<()> {
  use image::{ColorType, ImageEncoder};
  use image::codecs::png::{CompressionType, FilterType, PngEncoder};
  PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive)
    .write_image(img.as_raw(), img.width(), img.height(), ColorType::Rgb8)
}

pub fn save_image(img: &RgbImage, path: impl AsRef<std::path::Path>) -> ImageResult<()> {
  encode_image(img, std::io::BufWriter::new(std::fs::File::create(path)?))
}

struct Assets {
  files: GrayAlphaImage,
  ranks: GrayAlphaImage,
  pieces: GrayAlphaImage
}

impl Assets {
  fn get_files(&self, color: Color) -> GrayAlphaSubImage {
    self.files.view(0, match color {
      Color::White => 0,
      Color::Black => EDGE
    }, BOARD, EDGE)
  }

  fn get_ranks(&self, color: Color) -> GrayAlphaSubImage {
    self.ranks.view(match color {
      Color::White => 0,
      Color::Black => EDGE
    }, 0, EDGE, BOARD)
  }

  fn get_piece(&self, piece: Piece) -> GrayAlphaSubImage {
    let (column, color) = match piece {
      Piece::Queen(color, ..) => (0, color),
      Piece::King(color, ..) => (1, color),
      Piece::Rook(color, ..) => (2, color),
      Piece::Bishop(color, ..) => (3, color),
      Piece::Knight(color, ..) => (4, color),
      Piece::Pawn(color, ..) => (5, color),
    };

    let row = match color {
      Color::White => 0,
      Color::Black => 1
    };

    self.pieces.view(column * TILE, row * TILE, TILE, TILE)
  }

  fn instance() -> &'static Self {
    static INSTANCE: OnceCell<Assets> = OnceCell::new();
    INSTANCE.get_or_init(Self::load)
  }

  fn load() -> Self {
    Assets {
      files: decode_image(include_bytes!("../assets/files.png")),
      ranks: decode_image(include_bytes!("../assets/ranks.png")),
      pieces: decode_image(include_bytes!("../assets/pieces.png"))
    }
  }
}

fn fill_square(destination: &mut RgbImage, pixel: Rgb<u8>, (x, y): Pos, size: u32) {
  debug_assert!(destination.width() > x + size);
  debug_assert!(destination.height() > y + size);
  for sx in x..(x + size) {
    for sy in y..(y + size) {
      destination.put_pixel(sx, sy, pixel);
    };
  };
}

fn copy(destination: &mut RgbImage, source: GrayAlphaSubImage, (x, y): Pos) {
  for sx in 0..source.width() {
    for sy in 0..source.height() {
      let source_pixel = source.get_pixel(sx, sy);
      // don't do anything if the source pixel is transparent
      if source_pixel[1] != 0x00 {
        let destination_pixel = destination.get_pixel_mut(sx + x, sy + y);
        *destination_pixel = blend(*destination_pixel, source_pixel);
      };
    };
  };
}

fn blend(p1: Rgb<u8>, p2: LumaA<u8>) -> Rgb<u8> {
  let mut p1 = p1.to_rgba();
  let p2 = p2.to_rgba();
  p1.blend(&p2);
  p1.to_rgb()
}

fn decode_image(data: &'static [u8]) -> GrayAlphaImage {
  use image::DynamicImage;
  use image::codecs::png::PngDecoder;
  let decoder = PngDecoder::new(data).expect("failed to decode image");
  let img = DynamicImage::from_decoder(decoder).expect("failed to decode image");
  img.into_luma_alpha8()
}
