pub extern crate shakmaty;
pub extern crate image;

use glam::{Vec2, UVec2};
use shakmaty::{Color, Piece, Role, Square, File, Rank, Position};
use image::{DynamicImage, GenericImageView, GrayAlphaImage, ImageResult, Pixel, Rgb, Rgba, RgbImage, RgbaImage};
use once_cell::sync::OnceCell;

use std::io::{Read, Write};

type GrayAlphaSubImage<'a> = image::SubImage<&'a GrayAlphaImage>;
type Pos = (u32, u32);

const BOARD_FULL: u32 = BOARD + EDGE * 2;
const BOARD: u32 = 512;
const TILE: u32 = 64;
const EDGE: u32 = 32;

const DARK: Rgb<u8> = Rgb([0xb5, 0x88, 0x63]);
const LIGHT: Rgb<u8> = Rgb([0xf0, 0xd9, 0xb5]);
const DARK_HIGHLIGHT: Rgb<u8> = Rgb([0xaa, 0xa2, 0x3b]);
const LIGHT_HIGHLIGHT: Rgb<u8> = Rgb([0xce, 0xd2, 0x6b]);
const NEUTRAL: Rgb<u8> = Rgb([0xc9, 0xa3, 0x7e]);

const RANKS: Pos = (0, EDGE);
const FILES: Pos = (EDGE, BOARD + EDGE);

/// Returns true if the given position has a king that is in check
fn should_show_check(position: &impl Position, square: Square) -> bool {
  position.board().piece_at(square).map_or(false, |king| {
    king.role == Role::King &&
    position.king_attackers(
      square, king.color.other(),
      position.board().occupied()
    ).any()
  })
}

/// Pre-initializes all static resources for chessboard rendering
pub fn init() {
  Assets::instance();
}

/// Renders the given chessboard to a image buffer.
/// The chessboard will be oriented from the perspective specified by `side`.
/// Squares included in `highlighted` will be highlighted in green.
pub fn render_board(position: &impl Position, side: Color, highlighted: &[Square]) -> RgbImage {
  let assets = Assets::instance();
  let mut img = RgbImage::from_pixel(BOARD_FULL, BOARD_FULL, NEUTRAL);
  // get the correct textures for the files and ranks markings and copy them to the image buffer
  copy(&mut img, &*assets.get_files(side), FILES);
  copy(&mut img, &*assets.get_ranks(side), RANKS);
  for bx in 0u32..8u32 {
    for by in 0u32..8u32 {
      // the board is flipped vertically, to put the player on the bottom
      // if the player is black, it is flipped vertically (again), and horizontally
      let (x, y) = match side {
        Color::Black => (7 - bx, by),
        Color::White => (bx, 7 - by)
      };

      let pos = (x * TILE + EDGE, y * TILE + EDGE);
      let square = Square::from_coords(File::new(bx), Rank::new(by));
      // the chess board always has dark squares in the bottom left and top right
      let is_dark = bx % 2 == by % 2;
      let color = if highlighted.contains(&square) {
        if is_dark { DARK_HIGHLIGHT } else { LIGHT_HIGHLIGHT }
      } else {
        if is_dark { DARK } else { LIGHT }
      };

      fill_square(&mut img, color, pos, TILE);

      if should_show_check(position, square) {
        copy(&mut img, &assets.check, pos);
      };

      if let Some(piece) = position.board().piece_at(square) {
        copy(&mut img, &*assets.get_piece(piece), pos);
      };
    };
  };

  img
}

struct Assets {
  files: GrayAlphaImage,
  ranks: GrayAlphaImage,
  pieces: GrayAlphaImage,
  check: RgbaImage
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
    let column = match piece.role {
      Role::Queen => 0,
      Role::King => 1,
      Role::Rook => 2,
      Role::Bishop => 3,
      Role::Knight => 4,
      Role::Pawn => 5,
    };

    let row = match piece.color {
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
      files: decode_static_image(include_bytes!("../assets/files.png")).into_luma_alpha8(),
      ranks: decode_static_image(include_bytes!("../assets/ranks.png")).into_luma_alpha8(),
      pieces: decode_static_image(include_bytes!("../assets/pieces.png")).into_luma_alpha8(),
      check: generate_image_check()
    }
  }
}

fn generate_image_check() -> RgbaImage {
  const CENTER: Vec2 = Vec2::splat((64.0 - 1.0) / 2.0);
  RgbaImage::from_fn(64, 64, |x, y| {
    let pos = UVec2::new(x, y);
    let dist = pos.as_vec2().distance(CENTER);
    let alpha = (1.0 - (dist / 32.0).powi(2)).clamp(0.0, 1.0);
    Rgba([0xff, 0x00, 0x00, (alpha * 255.0) as u8])
  })
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

fn copy<P, S>(destination: &mut RgbImage, source: &S, (x, y): Pos)
where P: Pixel<Subpixel = u8>, S: GenericImageView<Pixel = P> {
  for sx in 0..source.width() {
    for sy in 0..source.height() {
      let source_pixel = source.get_pixel(sx, sy).to_rgba();
      // don't do anything if the source pixel is transparent
      if source_pixel[3] != 0x00 {
        let destination_pixel = destination.get_pixel_mut(sx + x, sy + y);
        *destination_pixel = blend(*destination_pixel, source_pixel);
      };
    };
  };
}

fn blend(p1: Rgb<u8>, p2: Rgba<u8>) -> Rgb<u8> {
  let mut p1 = p1.to_rgba();
  p1.blend(&p2);
  p1.to_rgb()
}

pub fn encode_image_rgb<W: Write>(img: &RgbImage, writer: W) -> ImageResult<()> {
  use image::{ColorType, ImageEncoder};
  use image::codecs::png::{CompressionType, FilterType, PngEncoder};
  PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive)
    .write_image(img.as_raw(), img.width(), img.height(), ColorType::Rgb8)
}

pub fn encode_image_rgba<W: Write>(img: &RgbaImage, writer: W) -> ImageResult<()> {
  use image::{ColorType, ImageEncoder};
  use image::codecs::png::{CompressionType, FilterType, PngEncoder};
  PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive)
    .write_image(img.as_raw(), img.width(), img.height(), ColorType::Rgba8)
}

pub fn decode_image<R: Read>(reader: R) -> ImageResult<DynamicImage> {
  image::codecs::png::PngDecoder::new(reader).and_then(DynamicImage::from_decoder)
}

fn decode_static_image(data: &'static [u8]) -> DynamicImage {
  decode_image(data).expect("failed to decode static image")
}
