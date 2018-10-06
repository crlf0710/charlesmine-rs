use apiw;

use apiw::graphics_subsystem::device_context::ScopedDeviceContext;
use apiw::graphics_subsystem::draw::Draw;
use apiw::graphics_subsystem::{Point, Size, Rect};
use apiw::graphics_subsystem::{RGBColor};
use apiw::utils::ManagedStrategy;
use apiw::Result;
use assets::GameButtonSpriteSheet;
use assets::BlockSpriteSheet;
use model::Board;
use assets::{Assets, DigitSpriteSheet};
use model::Block;
use model::BlockDisplayKind;

pub struct Layout {
    area_size: Size,
    block_area_dims: (usize, usize),
    block_area_size: Size,
    digit_pos_1: Point,
    digit_pos_2: Point,
    button_pos: Point,

    button_pressed: Cell<bool>,
    block_pressed: Cell<Option<(usize, usize, bool)>>,
}

impl Layout {
    pub const DIGITEDGE_LEFT: usize = 15;
    pub const DIGITEDGE_RIGHT: usize = 15;
    pub const DIGITEDGE_TOP: usize = 15;

    pub const BUTTONEDGE_TOP: usize = 15;

    pub const MIDDLE_BANNER: usize = 10;

    pub const BLOCK_AREA_EDGE_X: usize = 12;
    pub const BLOCK_AREA_EDGE_Y: usize = 12;
    pub const BLOCK_AREA_EDGE_TOP: usize = 12;


    pub const BLOCK_AREA_X: usize = Self::BLOCK_AREA_EDGE_X;
    pub const BLOCK_AREA_Y: usize =
        Self::DIGITEDGE_TOP + DigitSpriteSheet::DIGIT_HEIGHT +
            Self::MIDDLE_BANNER + Self::BLOCK_AREA_EDGE_TOP;

}

impl Layout {
    pub fn new(board: &Board) -> Self {
        let block_area_dims = board.size();
        let block_area_size = BlockSpriteSheet::calc_block_area_size(block_area_dims);
        let area_size = Size::new(
            Self::BLOCK_AREA_X + block_area_size.cx() + Self::BLOCK_AREA_EDGE_X,
            Self::BLOCK_AREA_Y + block_area_size.cy() + Self::BLOCK_AREA_EDGE_Y
        );

        let digit_pos_1 = Point::new(
            Self::DIGITEDGE_LEFT as _,
            Self::DIGITEDGE_TOP as _,
        );

        let digit_pos_2 = Point::new(
            (area_size.cx() - Self::DIGITEDGE_RIGHT - DigitSpriteSheet::DIGIT_WIDTH * DigitPanel::DIGITCOUNT) as _,
            Self::DIGITEDGE_TOP as _,
        );

        let button_pos = Point::new(
            (area_size.cx() - GameButtonSpriteSheet::BUTTON_WIDTH) as isize / 2,
            Self::BUTTONEDGE_TOP as _,
        );

        Layout {
            area_size,
            block_area_dims,
            block_area_size,
            digit_pos_1,
            digit_pos_2,
            button_pos,
            button_pressed: Cell::new(false),
            block_pressed: Cell::new(None),
        }
    }
}

#[derive(PartialEq)]
enum BorderPosition {
    Inner,
    Outer,
}

struct ThreeDimBorder {
    rect: Rect,
    border_pos: BorderPosition,
    color_nw: RGBColor,
    color_se: RGBColor,
}

impl ThreeDimBorder {
    const BORDER_WIDTH: usize = 2;
}

impl Draw for ThreeDimBorder {
    fn draw(self, dc: &mut ScopedDeviceContext) -> Result<()> {
        use apiw::graphics_subsystem::object::PenBuilder;
        let pen1 = PenBuilder::new()
            .width(Self::BORDER_WIDTH)
            .color(self.color_nw)
            .create()?;

        let pen2 = PenBuilder::new()
            .width(Self::BORDER_WIDTH)
            .color(self.color_se)
            .create()?;

        let deflate = if self.border_pos == BorderPosition::Outer {
            Some(Self::BORDER_WIDTH)
        } else {
            None
        };

        let rect = if let Some(w) = deflate {
            self.rect.deflate(w)
        } else {
            self.rect
        };

        dc
            .select_pen(pen1)?
            .move_to(rect.top_right())?
            .line_to(rect.top_left())?
            .line_to(rect.bottom_left())?
            .select_pen(pen2)?
            .line_to(rect.bottom_right())?
            .line_to(rect.top_right())?;

        Ok(())
    }
}

struct DigitPanel<'a> {
    pos: Point,
    value: isize,
    assets: &'a Assets,
}

impl<'a> DigitPanel<'a> {
    const MAXVALUE: isize = 999;
    const DIGITCOUNT: usize = 3;
}


impl<'a> Draw for DigitPanel<'a> {
    fn draw(self, dc: &mut ScopedDeviceContext) -> Result<()> {
        use assets::SpriteSheet;

        let value_abs = self.value.abs() as usize;
        let neg = self.value.is_negative();
        let mut digits_sheet = self.assets.digits_sheet.instantiate(dc)?;

        //let memory_dc = MemoryDeviceContext::create_compatible(&dc);
        //memory_dc.select_bitmap(asset.bitmap_digits.clone());
        for i in 0..Self::DIGITCOUNT {
            let sprite_idx = if i == 0 && neg {
                DigitSpriteSheet::sprite_index_neg()
            } else {
                let digit = (value_abs % 10_usize.pow(Self::DIGITCOUNT as u32 - i as u32))
                    / 10_usize.pow(Self::DIGITCOUNT as u32 - i as u32 - 1);
                DigitSpriteSheet::sprite_index_digit(digit)
            };

            digits_sheet.draw_sprite(self.pos.offset((i * DigitSpriteSheet::DIGIT_WIDTH) as isize, 0),
                                     sprite_idx)?;
        }

        Ok(())
    }
}

struct GameButton<'a> {
    pos: Point,
    state: GameButtonDisplayKind,
    assets: &'a Assets,
}

impl<'a> Draw for GameButton<'a> {
    fn draw(self, dc: &mut ScopedDeviceContext) -> Result<()> {
        use assets::SpriteSheet;

        let mut game_button_sheet = self.assets.gamebutton_sheet.instantiate(dc)?;
        let sprite_idx = GameButtonSpriteSheet::sprite_index(self.state);
        game_button_sheet.draw_sprite(self.pos, sprite_idx)?;

        Ok(())
    }
}

struct MineBlock<'a> {
    minefield_pos: Point,
    block_pos: (usize, usize),
    block_shape_dir: BlockShape,
    block_display_kind: BlockDisplayKind,
    assets: &'a Assets,
}

impl<'a> Draw for MineBlock<'a> {
    fn draw(self, dc: &mut ScopedDeviceContext) -> Result<()> {
        use assets::SpriteSheet;

        let mut block_sheet = self.assets.block_sheet.instantiate(dc)?;
        let sprite_idx = BlockSpriteSheet::sprite_index(self.block_shape_dir, self.block_display_kind);
        let block_draw_pos = BlockSpriteSheet::calc_block_pos(self.minefield_pos, self.block_pos);
        block_sheet.draw_sprite(block_draw_pos, sprite_idx)?;

        Ok(())
    }
}

impl Board {
    fn display_value_mine_left(&self) -> isize {
        self.goal_mark_count() as isize - self.cur_mark_count() as isize
    }

    fn display_value_time(&self) -> isize {
        use super::model::BoardStatus;
        use std::cmp::min;
        use chrono::Local;
        let status = self.status();
        let (start, end) = match status {
            BoardStatus::Ready => (None, None),
            BoardStatus::Going(v) => (Some(v), None),
            BoardStatus::Finished(s, e) => (Some(s), Some(e)),
            BoardStatus::Died(s, e, ..) => (Some(s), Some(e)),
        };
        if let Some(s) = start {
            let e = end.unwrap_or_else(Local::now);
            let v = (e - s).num_seconds();
            min(v, DigitPanel::MAXVALUE as _) as isize
        } else {
            0
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum GameTarget {
    GameButton,
    FieldBlock{y: usize, x: usize},
    Other,
}


impl Layout {
    pub fn draw(&self, dc: &mut ScopedDeviceContext, board: &Board, assets: &Assets) -> Result<()> {
        dc
            .draw(ThreeDimBorder {
                rect: Rect::new(Point::ORIGIN, self.area_size), 
				border_pos: BorderPosition::Inner, 
				color_nw: RGBColor::WHITE, 
                color_se: RGBColor::GRAY
            })?
			.draw(ThreeDimBorder {
				rect: Rect::new(Point::new(Self::BLOCK_AREA_X as _, Self::BUTTONEDGE_TOP as _), 
					Size::new(self.block_area_size.cx(), GameButtonSpriteSheet::BUTTON_HEIGHT)),
				border_pos: BorderPosition::Outer,
				color_nw: RGBColor::GRAY,
                color_se: RGBColor::WHITE
            })?
			.draw(ThreeDimBorder {
				rect: Rect::new(Point::new(Self::BLOCK_AREA_X as _, Self::BLOCK_AREA_Y as _), 
					self.block_area_size),
				border_pos: BorderPosition::Outer,
				color_nw: RGBColor::GRAY, 
                color_se: RGBColor::WHITE
            })?
            .draw(DigitPanel {
                pos: self.digit_pos_1,
                value: board.display_value_mine_left(),
                assets: &assets,
            })?
            .draw(DigitPanel {
                pos: self.digit_pos_2,
                value: board.display_value_time(),
                assets: &assets,
            })?
            .draw(GameButton {
                pos: self.button_pos,
                state: board.game_button_display_kind(self.button_pressed.get()),
                assets: &assets,
            })?
			.draw_from_iter((0..board.size().0).flat_map(
				move |y| (0..board.size().1).map(move |x| {
					MineBlock {
                        minefield_pos: Point::new(Layout::BLOCK_AREA_X as isize,
                                                  Layout::BLOCK_AREA_Y as isize),
                        block_pos: (y, x),
                        block_shape_dir: Board::block_shape(y, x),
                        block_display_kind: board.block_display_kind((y, x), self.block_pressed.get()),
                        assets: &assets,
                    }
				})
			))?;
        Ok(())
    }

    pub fn hit_test(&self, point: Point) -> GameTarget {
        let button_size = Size::new(GameButtonSpriteSheet::BUTTON_WIDTH,
                                    GameButtonSpriteSheet::BUTTON_HEIGHT);
        let button_rect =
            Rect::new(self.button_pos, button_size);
        if button_rect.contains(point) {
            return GameTarget::GameButton;
        }

        // this is div_euc that is not stabilized yet.
        fn floor_div(lhs: isize, rhs: isize) -> isize {
            let q = lhs / rhs;
            if lhs % rhs < 0 {
                return if rhs > 0 { q - 1 } else { q + 1 }
            }
            q
        }

        let y_idx = floor_div(point.y() - Layout::BLOCK_AREA_Y as isize, BlockSpriteSheet::BLOCKSIZE_Y as isize) as isize;
        if 0 <= y_idx && y_idx < self.block_area_dims.0 as isize {
            let x_idx_min = floor_div(point.x() - Layout::BLOCK_AREA_X as isize - (BlockSpriteSheet::BLOCKSIZE_X as isize - 1),
                                      BlockSpriteSheet::BLOCKDELTA_X as _) as isize;
            let x_idx_max = floor_div(point.x() - Layout::BLOCK_AREA_X as isize,
                                      BlockSpriteSheet::BLOCKDELTA_X as _) as isize;
            for x_idx in x_idx_min..=x_idx_max {
                if 0 <= x_idx && x_idx < self.block_area_dims.1 as isize {
                    let x_offset = point.x() as usize - Layout::BLOCK_AREA_X - BlockSpriteSheet::BLOCKDELTA_X * x_idx as usize;
                    let y_offset = point.y() as usize - Layout::BLOCK_AREA_Y - BlockSpriteSheet::BLOCKDELTA_Y * y_idx as usize;
                    if x_offset >= BlockSpriteSheet::BLOCKSIZE_X {
                        continue;
                    }
                    if BlockSpriteSheet::hit_test_shape(Board::block_shape(y_idx as usize, x_idx as usize), (y_offset, x_offset))
                        .expect("Failed to hit test.") {
                        return GameTarget::FieldBlock {y: y_idx as usize, x: x_idx as usize};
                    }
                }
            }
        }
        GameTarget::Other
    }

    pub fn set_button_pressed(&mut self, state: bool) {
        self.button_pressed.set(state);
    }

    pub fn set_block_pressed(&mut self, y: usize, x: usize, blast: bool) {
        self.block_pressed.set(Some((y, x, blast)));
    }

    pub fn unset_block_pressed(&mut self, y: usize, x: usize, blast: bool) {
        if self.block_pressed.get() == Some((y, x, blast)) {
            self.block_pressed.set(None)
        }
    }
}

pub trait AdjustWithLayout {
    fn adjust_with_layout(&self, &Layout) -> apiw::Result<&Self>;
}

use apiw::windows_subsystem::window::ForeignWindow;
use model::BlockShape;
use model::GameButtonDisplayKind;
use std::cell::Cell;

impl AdjustWithLayout for ForeignWindow {
    fn adjust_with_layout(&self, layout: &Layout) -> apiw::Result<&Self> {
        let rect = Rect::new(Point::ORIGIN, layout.area_size);
        let new_rect = Self::predict_window_rect_from_client_rect_and_window(rect, &self)?;
        self.reposition_set_size(new_rect.size())?;

        Ok(self)
    }
}

