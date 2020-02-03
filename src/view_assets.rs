use crate::model::Block;
use crate::model::BlockShape;
use apiw;
use apiw::graphics_subsystem::device_context::{LocalDeviceContext, ScopedDeviceContext};
use apiw::graphics_subsystem::object::Bitmap;
use apiw::graphics_subsystem::RGBColor;
use apiw::graphics_subsystem::TenaryROP;
use apiw::graphics_subsystem::{Point, Size};
use apiw::shared::ManagedStrategy;
use smallvec::{smallvec, SmallVec};
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::marker::PhantomData;

#[path = "view_assets_catalog.rs"]
pub(crate) mod resources;

use self::resources::*;
use crate::model::BlockDisplayKind;
use crate::model::Board;
use crate::model::GameButtonDisplayKind;

pub struct SpriteSheetInstance<
    'a,
    'd,
    S: BorrowMut<LocalDeviceContext>,
    D: BorrowMut<ScopedDeviceContext<'d>>,
> {
    sprite_sheet: &'a dyn SpriteSheet,
    memory_dc: S,
    target_dc: D,
    sprite_bitmap: Option<Bitmap>,
    phantom: PhantomData<&'d ()>,
}

impl<'a, 'd, S, D> SpriteSheetInstance<'a, 'd, S, D>
where
    S: BorrowMut<LocalDeviceContext>,
    D: BorrowMut<ScopedDeviceContext<'d>>,
{
    #[allow(unreachable_patterns)]
    pub fn draw_sprite(&mut self, dest_pos: Point, idx: usize) -> apiw::Result<()> {
        let src_dc = self.memory_dc.borrow();
        let dest_dc = self.target_dc.borrow_mut();
        let (src_pos, size) = self.sprite_sheet.sprite_coord(idx);
        match self.sprite_sheet.sprite_draw_param() {
            SpriteDrawParam::TransparentBlt(key) => {
                dest_dc.transparentblt(src_dc, src_pos, size, dest_pos, size, key)?
            }
            SpriteDrawParam::BitBlt | _ => {
                dest_dc.bitblt(src_dc, src_pos, dest_pos, size, TenaryROP::SRCCOPY)?
            }
        };
        Ok(())
    }
}

impl<'a, 'd, S, D> Drop for SpriteSheetInstance<'a, 'd, S, D>
where
    S: BorrowMut<LocalDeviceContext>,
    D: BorrowMut<ScopedDeviceContext<'d>>,
{
    fn drop(&mut self) {
        self.memory_dc.borrow_mut().reset_to_initial_state();

        if let Some(sprite_bitmap) = self.sprite_bitmap.take() {
            self.sprite_sheet.reuse_bitmap_instance(sprite_bitmap);
        }
    }
}

pub enum SpriteDrawParam {
    BitBlt,
    TransparentBlt(RGBColor),
}

pub trait SpriteSheet {
    fn sprite_coord(&self, sprite_idx: usize) -> (Point, Size);

    fn sprite_draw_param(&self) -> SpriteDrawParam {
        SpriteDrawParam::BitBlt
    }

    fn bitmap_instance(&self) -> Bitmap;

    fn reuse_bitmap_instance(&self, clean_instance: Bitmap);

    fn instantiate<'a, 'b, 'c>(
        &'a self,
        target_dc: &'b mut ScopedDeviceContext<'c>,
    ) -> apiw::Result<
        SpriteSheetInstance<'a, 'c, LocalDeviceContext, &'b mut ScopedDeviceContext<'c>>,
    >
    where
        Self: Sized,
    {
        let mut memory_dc = LocalDeviceContext::new_compatible_memory_dc(target_dc)?;
        let bitmap = self.bitmap_instance();
        memory_dc.select_bitmap(bitmap.clone())?;
        let sprite_sheet = self as _;
        Ok(SpriteSheetInstance {
            sprite_sheet,
            memory_dc,
            target_dc,
            sprite_bitmap: Some(bitmap),
            phantom: PhantomData,
        })
    }
}

pub struct DigitSpriteSheet {
    instances: RefCell<SmallVec<[Bitmap; 1]>>,
}

impl DigitSpriteSheet {
    pub const DIGIT_WIDTH: usize = 13;
    pub const DIGIT_HEIGHT: usize = 23;

    pub fn new() -> Self {
        DigitSpriteSheet {
            instances: RefCell::new(smallvec![]),
        }
    }

    pub fn sprite_index_neg() -> usize {
        0
    }

    pub fn sprite_index_digit(v: usize) -> usize {
        assert!(v <= 9);
        11 - v
    }
}

impl SpriteSheet for DigitSpriteSheet {
    fn sprite_coord(&self, sprite_idx: usize) -> (Point, Size) {
        (
            Point::new(0, (sprite_idx * Self::DIGIT_HEIGHT) as isize),
            Size::new(Self::DIGIT_WIDTH, Self::DIGIT_HEIGHT),
        )
    }

    fn bitmap_instance(&self) -> Bitmap {
        if let Some(v) = self.instances.borrow_mut().pop() {
            v
        } else {
            Bitmap::load_from_resource_id(IDB_DIGIT as _).expect("Failed to load resource bitmap.")
        }
    }

    fn reuse_bitmap_instance(&self, v: Bitmap) {
        self.instances.borrow_mut().push(v)
    }
}

pub struct GameButtonSpriteSheet {
    instances: RefCell<SmallVec<[Bitmap; 1]>>,
}

impl GameButtonSpriteSheet {
    pub const BUTTON_WIDTH: usize = 24;
    pub const BUTTON_HEIGHT: usize = 24;

    pub fn new() -> Self {
        GameButtonSpriteSheet {
            instances: RefCell::new(smallvec![]),
        }
    }

    pub fn sprite_index(state: GameButtonDisplayKind) -> usize {
        match state {
            GameButtonDisplayKind::Normal => 4,
            GameButtonDisplayKind::Pushed => 0,
            GameButtonDisplayKind::Danger => 3,
            GameButtonDisplayKind::Finished => 1,
            GameButtonDisplayKind::Died => 2,
        }
    }
}

impl SpriteSheet for GameButtonSpriteSheet {
    fn sprite_coord(&self, sprite_idx: usize) -> (Point, Size) {
        (
            Point::new(0, (sprite_idx * Self::BUTTON_HEIGHT) as isize),
            Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT),
        )
    }

    fn bitmap_instance(&self) -> Bitmap {
        if let Some(v) = self.instances.borrow_mut().pop() {
            v
        } else {
            Bitmap::load_from_resource_id(IDB_BUTTON as _).expect("Failed to load resource bitmap.")
        }
    }

    fn reuse_bitmap_instance(&self, v: Bitmap) {
        self.instances.borrow_mut().push(v)
    }
}

pub struct BlockSpriteSheet {
    instances: RefCell<SmallVec<[Bitmap; 1]>>,
}

impl BlockSpriteSheet {
    pub const BLOCKSIZE_X: usize = 24;
    pub const BLOCKSIZE_Y: usize = 20;

    pub const BLOCKDELTA_X: usize = 13;
    pub const BLOCKDELTA_Y: usize = 20;

    pub const TRANSPARENT_COLOR: RGBColor = RGBColor::FUCHSIA;

    pub fn new() -> Self {
        BlockSpriteSheet {
            instances: RefCell::new(smallvec![]),
        }
    }

    pub fn calc_block_area_size(board_size: (usize, usize)) -> Size {
        Size::new(
            Self::BLOCKSIZE_X + (board_size.1 - 1) * Self::BLOCKDELTA_X,
            Self::BLOCKSIZE_Y + (board_size.0 - 1) * Self::BLOCKDELTA_Y,
        )
    }

    pub fn hit_test_shape(shape: BlockShape, pos: (usize, usize)) -> apiw::Result<bool> {
        thread_local! {
            static SHAPE_DC: RefCell<LocalDeviceContext> = {
                let mut memdc = LocalDeviceContext::new_compatible_memory_dc_for_current_screen()
                    .expect("Failed to create memory dc.");
                let bitmap = Bitmap::load_from_resource_id(IDB_BLOCKS as _)
                    .expect("Failed to load resource bitmap.");
                memdc.set_background_color(BlockSpriteSheet::TRANSPARENT_COLOR)
                    .expect("Failed to select background color.");
                memdc.select_bitmap(bitmap)
                    .expect("Failed to select bitmap.");
                RefCell::new(memdc)
            };
        }

        SHAPE_DC.with(|shape_dc| {
            let mut shape_dc = shape_dc.borrow_mut();

            let shape_offset = match shape {
                BlockShape::DeltaLike => (0isize, 0isize),
                BlockShape::RevDeltaLike => (0isize, Self::BLOCKSIZE_X as isize),
            };

            let point = Point::new(
                pos.1 as isize + shape_offset.1,
                pos.0 as isize + shape_offset.0,
            );
            if let Some(color) = shape_dc.get_pixel(point)? {
                if color != Self::TRANSPARENT_COLOR {
                    return Ok(true);
                }
            }

            Ok(false)
        })
    }

    pub fn calc_block_pos(minefield_pos: Point, board_item: (usize, usize)) -> Point {
        minefield_pos.offset(
            (board_item.1 * Self::BLOCKDELTA_X) as isize,
            (board_item.0 * Self::BLOCKDELTA_Y) as isize,
        )
    }

    pub fn sprite_index(block_shape: BlockShape, block_display_kind: BlockDisplayKind) -> usize {
        let block_kind = match block_display_kind {
            BlockDisplayKind::Normal => 0,
            BlockDisplayKind::MarkedMine => 1,
            BlockDisplayKind::MarkedQuestionable => 2,
            BlockDisplayKind::ExplodedMine => 3,
            BlockDisplayKind::WrongMarkedMine => 4,
            BlockDisplayKind::NotMarkedMine => 5,
            BlockDisplayKind::PushMarkedQuestionable => 6,
            BlockDisplayKind::OpenWithNumber(n) => 19 - n as usize,
            BlockDisplayKind::PushNormal => 19,
        };

        let block_shape_offset = match block_shape {
            BlockShape::DeltaLike => 0,
            BlockShape::RevDeltaLike => 1,
        };

        block_kind * 2 + block_shape_offset
    }
}

impl SpriteSheet for BlockSpriteSheet {
    fn sprite_coord(&self, sprite_idx: usize) -> (Point, Size) {
        (
            Point::new(
                ((sprite_idx % 2) * Self::BLOCKSIZE_X) as isize,
                ((sprite_idx / 2) * Self::BLOCKSIZE_Y) as isize,
            ),
            Size::new(Self::BLOCKSIZE_X, Self::BLOCKSIZE_Y),
        )
    }

    fn sprite_draw_param(&self) -> SpriteDrawParam {
        SpriteDrawParam::TransparentBlt(Self::TRANSPARENT_COLOR)
    }

    fn bitmap_instance(&self) -> Bitmap {
        if let Some(v) = self.instances.borrow_mut().pop() {
            v
        } else {
            Bitmap::load_from_resource_id(IDB_BLOCKS as _).expect("Failed to load resource bitmap.")
        }
    }

    fn reuse_bitmap_instance(&self, v: Bitmap) {
        self.instances.borrow_mut().push(v)
    }
}

pub struct Assets {
    pub digits_sheet: DigitSpriteSheet,
    pub gamebutton_sheet: GameButtonSpriteSheet,
    pub block_sheet: BlockSpriteSheet,
}

impl Assets {
    pub fn new() -> Self {
        Assets {
            digits_sheet: DigitSpriteSheet::new(),
            gamebutton_sheet: GameButtonSpriteSheet::new(),
            block_sheet: BlockSpriteSheet::new(),
        }
    }
}
