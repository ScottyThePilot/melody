use fontdue::{Font, FontSettings};
use fontdue::layout::{Layout, LayoutSettings, CoordinateSystem, TextStyle, HorizontalAlign, VerticalAlign};
use glam::{Vec2, UVec2};
use shakmaty::{Color, Piece, Role, Square, File, Rank, Position};
use image::{DynamicImage, GenericImageView, ImageResult, Pixel, Rgb, Rgba, RgbImage, RgbaImage};

use std::io::prelude::*;
use std::io::Cursor;



type RgbaSubImage<'a> = image::SubImage<&'a RgbaImage>;

const TILE_SIZE: u32 = 64;
const MARGIN_SIZE: UVec2 = UVec2::new(16, 32);
const BOARD_SIZE: UVec2 = UVec2::splat(TILE_SIZE * 8);
const BOARD_SIZE_FULL: UVec2 = UVec2::new(
  BOARD_SIZE.x + MARGIN_SIZE.x * 2,
  BOARD_SIZE.y + MARGIN_SIZE.y * 2
);

const NAME_POS_TOP: UVec2 = UVec2::new(
  BOARD_SIZE_FULL.x / 2,
  MARGIN_SIZE.y / 2
);

const NAME_POS_BOTTOM: UVec2 = UVec2::new(
  BOARD_SIZE_FULL.x / 2,
  BOARD_SIZE_FULL.y - MARGIN_SIZE.y / 2
);

const BLACK: Rgb<u8> = Rgb([0x00; 3]);
const DARK: Rgb<u8> = Rgb([0xb5, 0x88, 0x63]);
const LIGHT: Rgb<u8> = Rgb([0xf0, 0xd9, 0xb5]);
const DARK_HIGHLIGHT: Rgb<u8> = Rgb([0xaa, 0xa2, 0x3b]);
const LIGHT_HIGHLIGHT: Rgb<u8> = Rgb([0xce, 0xd2, 0x6b]);
const NEUTRAL: Rgb<u8> = Rgb([0xc9, 0xa3, 0x7e]);

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

/// Renders the given chessboard to a image buffer.
/// The chessboard will be oriented from the perspective specified by `side`.
/// Squares included in `highlighted` will be highlighted in green.
pub fn render_board(
  position: &impl Position,
  side: Color,
  highlighted: &[Square],
  players: [&str; 2],
  assets: &Assets
) -> RgbImage {
  let mut img = RgbImage::from_pixel(BOARD_SIZE_FULL.x, BOARD_SIZE_FULL.y, NEUTRAL);

  let (top_player, bottom_player) = match side {
    Color::White => (players[1], players[0]),
    Color::Black => (players[0], players[1])
  };

  fill_text(&mut img, BLACK, top_player, TextOptions {
    pos: NAME_POS_TOP,
    size: 24.0,
    font: &assets.font,
    horizontal_align: HorizontalAlign::Center,
    vertical_align: VerticalAlign::Middle,
    window: UVec2::splat(1024)
  });

  fill_text(&mut img, BLACK, bottom_player, TextOptions {
    pos: NAME_POS_BOTTOM,
    size: 24.0,
    font: &assets.font,
    horizontal_align: HorizontalAlign::Center,
    vertical_align: VerticalAlign::Middle,
    window: UVec2::splat(1024)
  });

  for bx in 0u32..8u32 {
    for by in 0u32..8u32 {
      // the board is flipped vertically, to put the player on the bottom
      // if the player is black, it is flipped vertically (again), and horizontally
      let pos = match side {
        Color::Black => UVec2::new(7 - bx, by),
        Color::White => UVec2::new(bx, 7 - by)
      };

      let show_rank_numbers = pos.x == 0;
      let show_file_letters = pos.y == 7;

      let pos = pos * TILE_SIZE + MARGIN_SIZE;
      let square = Square::from_coords(File::new(bx), Rank::new(by));
      // the chess board always has dark squares in the bottom left and top right
      let is_dark = bx % 2 == by % 2;
      let is_highlighted = highlighted.contains(&square);
      let color = get_color(is_dark, is_highlighted);
      let color_inverted = get_color(!is_dark, is_highlighted);

      fill_square(&mut img, color, pos, TILE_SIZE);

      // Only print file letters for the bottom-most rank of tiles
      if show_file_letters {
        let file_letter = (b'A' + bx as u8) as char;
        let mut text_temporary = [0x00];
        let text = file_letter.encode_utf8(&mut text_temporary);

        fill_text(&mut img, color_inverted, text, TextOptions {
          pos: pos + TILE_SIZE - UVec2::new(2, 0),
          size: 12.0,
          font: &assets.font,
          horizontal_align: HorizontalAlign::Right,
          vertical_align: VerticalAlign::Bottom,
          window: UVec2::splat(256)
        });
      };

      // Only print rank numbers for the right-most file of tiles
      if show_rank_numbers {
        let rank_number = (b'1' + by as u8) as char;
        let mut text_temporary = [0x00];
        let text = rank_number.encode_utf8(&mut text_temporary);

        fill_text(&mut img, color_inverted, text, TextOptions {
          pos: pos + UVec2::new(2, 0),
          size: 12.0,
          font: &assets.font,
          horizontal_align: HorizontalAlign::Left,
          vertical_align: VerticalAlign::Top,
          window: UVec2::splat(256)
        });
      };

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

fn get_color(is_dark: bool, is_highlighted: bool) -> Rgb<u8> {
  if is_dark {
    if is_highlighted { DARK_HIGHLIGHT } else { DARK }
  } else {
    if is_highlighted { LIGHT_HIGHLIGHT } else { LIGHT }
  }
}

#[derive(Debug)]
pub struct Assets {
  font: Font,
  pieces: RgbaImage,
  check: RgbaImage
}

impl Assets {
  fn get_piece(&self, piece: Piece) -> RgbaSubImage {
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

    self.pieces.view(column * TILE_SIZE, row * TILE_SIZE, TILE_SIZE, TILE_SIZE)
  }

  pub fn load() -> Self {
    let font_settings = FontSettings::default();
    let font = Font::from_bytes(include_bytes!("../assets/Roboto-Bold.ttf").as_slice(), font_settings)
      .expect("failed to construct static font");
    let pieces = decode_image(Cursor::new(include_bytes!("../assets/Pieces.png").as_slice()))
      .expect("failed to decode static image").into_rgba8();
    Assets { font, pieces, check: generate_image_check() }
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

fn fill_square(destination: &mut RgbImage, pixel: Rgb<u8>, pos: UVec2, size: u32) {
  debug_assert!(destination.width() > pos.x + size);
  debug_assert!(destination.height() > pos.y + size);
  for sx in pos.x..(pos.x + size) {
    for sy in pos.y..(pos.y + size) {
      destination.put_pixel(sx, sy, pixel);
    };
  };
}

struct TextOptions<'f> {
  pos: UVec2,
  size: f32,
  font: &'f Font,
  horizontal_align: HorizontalAlign,
  vertical_align: VerticalAlign,
  window: UVec2
}

fn fill_text(destination: &mut RgbImage, pixel: Rgb<u8>, text: &str, text_options: TextOptions) {
  let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
  layout.reset(&LayoutSettings {
    x: text_options.pos.x as f32,
    y: text_options.pos.y as f32,
    max_width: Some(text_options.window.x as f32),
    max_height: Some(text_options.window.y as f32),
    horizontal_align: text_options.horizontal_align,
    vertical_align: text_options.vertical_align,
    ..LayoutSettings::default()
  });

  let global_offset = UVec2::new(
    match text_options.horizontal_align {
      HorizontalAlign::Left => 0,
      HorizontalAlign::Center => text_options.window.x / 2,
      HorizontalAlign::Right => text_options.window.x
    },
    match text_options.vertical_align {
      VerticalAlign::Top => 0,
      VerticalAlign::Middle => text_options.window.y / 2,
      VerticalAlign::Bottom => text_options.window.y
    }
  );

  let font = text_options.font;
  layout.append(&[font], &TextStyle::new(text, text_options.size, 0));
  for glyph in layout.glyphs() {
    let glyph_offset = Vec2::new(glyph.x, glyph.y).round().as_ivec2()
      .saturating_sub_unsigned(global_offset);
    let (metrics, bitmap) = font.rasterize_config(glyph.key);
    for sx in 0..metrics.width {
      for sy in 0..metrics.height {
        let alpha = bitmap[sx + sy * metrics.width];
        if alpha == 0x00 { continue };

        let s_pos = UVec2::new(sx as u32, sy as u32);
        let destination_pos = (s_pos).checked_add_signed(glyph_offset)
          .filter(|pos| destination.width() > pos.x && destination.height() > pos.y);
        if let Some(destination_pos) = destination_pos {
          let source_pixel = Rgba([pixel[0], pixel[1], pixel[2], alpha]);
          let destination_pixel = destination.get_pixel_mut(destination_pos.x, destination_pos.y);
          *destination_pixel = blend(*destination_pixel, source_pixel);
        };
      };
    };
  };
}

fn copy<P, S>(destination: &mut RgbImage, source: &S, pos: UVec2)
where P: Pixel<Subpixel = u8>, S: GenericImageView<Pixel = P> {
  for sx in 0..source.width() {
    for sy in 0..source.height() {
      let source_pixel = source.get_pixel(sx, sy).to_rgba();
      let pos = pos + UVec2::new(sx, sy);
      // don't do anything if the source pixel is transparent
      if source_pixel[3] != 0x00 {
        let destination_pixel = destination.get_pixel_mut(pos.x, pos.y);
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
  use image::{ExtendedColorType, ImageEncoder};
  use image::codecs::png::{CompressionType, FilterType, PngEncoder};
  PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive)
    .write_image(img.as_raw(), img.width(), img.height(), ExtendedColorType::Rgb8)
}

pub fn encode_image_rgba<W: Write>(img: &RgbaImage, writer: W) -> ImageResult<()> {
  use image::{ExtendedColorType, ImageEncoder};
  use image::codecs::png::{CompressionType, FilterType, PngEncoder};
  PngEncoder::new_with_quality(writer, CompressionType::Best, FilterType::Adaptive)
    .write_image(img.as_raw(), img.width(), img.height(), ExtendedColorType::Rgba8)
}

pub fn decode_image<R: BufRead + Seek>(reader: R) -> ImageResult<DynamicImage> {
  image::codecs::png::PngDecoder::new(reader).and_then(DynamicImage::from_decoder)
}
