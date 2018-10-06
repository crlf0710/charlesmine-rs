use std::borrow::{BorrowMut};
use std::cell::RefCell;
use smallvec::SmallVec;
use apiw;
use apiw::graphics_subsystem::{Point, Size};
use apiw::graphics_subsystem::object::Bitmap;
use apiw::utils::ManagedStrategy;
use apiw::graphics_subsystem::device_context::{ScopedDeviceContext, LocalDeviceContext};
use std::marker::PhantomData;
use apiw::graphics_subsystem::TenaryROP;
use model::Block;
use model::BlockShape;
use apiw::graphics_subsystem::RGBColor;

#[allow(dead_code)]
mod resources {
    pub const IDC_MYICON : isize = 2;
    pub const IDD_CHARLESMINE_DIALOG : isize = 102;
    pub const IDS_APP_TITLE : isize = 103;
    pub const IDI_CHARLESMINE : isize = 107;
    pub const IDC_CHARLESMINE : isize = 109;
    pub const IDC_TEXT1 : isize = 112;
    pub const IDC_TEXT2 : isize = 113;
    pub const IDC_TEXT3 : isize = 114;
    pub const IDC_TEXT4 : isize = 115;
    pub const IDC_EDIT1 : isize = 116;
    pub const IDC_EDIT2 : isize = 117;
    pub const IDC_EDIT3 : isize = 118;
    pub const IDC_EDIT4 : isize = 119;
    pub const IDC_BUTTON1 : isize = 120;
    pub const IDC_BUTTON2 : isize = 121;
    pub const IDC_EXTRA1 : isize = 122;
    pub const IDC_EXTRA2 : isize = 123;
    pub const IDC_EXTRA3 : isize = 124;
    pub const IDR_MAINFRAME : isize = 128;
    pub const IDB_BLOCKS : isize = 129;
    pub const IDB_BUTTON : isize = 130;
    pub const IDB_DIGIT : isize = 131;
    pub const IDM_FILE_NEW : isize = 151;
    pub const IDM_FILE_GAME_EASY : isize = 152;
    pub const IDM_FILE_GAME_MEDIUM : isize = 153;
    pub const IDM_FILE_GAME_HARD : isize = 154;
    pub const IDM_FILE_GAME_CUSTOM : isize = 155;
    pub const IDM_FILE_MARK : isize = 156;
    pub const IDM_FILE_HERO_LIST : isize = 157;
    pub const IDM_FILE_EXIT : isize = 158;
    pub const IDM_ADVANCED_LOADMAP : isize = 161;
    pub const IDM_ADVANCED_SAVEMAP : isize = 162;
    pub const IDM_ADVANCED_RESTART : isize = 163;
    pub const IDM_ADVANCED_RECORD_RECORD : isize = 164;
    pub const IDM_ADVANCED_RECORD_PLAY : isize = 166;
    pub const IDM_ADVANCED_RECORD_STOP : isize = 167;
    pub const IDM_HELP_ABOUT : isize = 171;
    pub const IDD_ABOUTBOX : isize = 201;
    pub const IDD_CUSTOM_GAME : isize = 202;
    pub const IDD_HERO_NAME : isize = 203;
    pub const IDD_HERO_LIST : isize = 204;
    pub const IDS_ABOUTTEXT : isize = 241;
    pub const IDS_ABOUTTEXT1 : isize = 242;
    pub const IDS_ABOUTTEXT2 : isize = 243;
    pub const IDS_ABOUTTEXT3 : isize = 244;
    pub const IDS_CUSTOMGAME : isize = 251;
    pub const IDS_CUSTOMGAME_HEIGHT : isize = 252;
    pub const IDS_CUSTOMGAME_WIDTH : isize = 253;
    pub const IDS_CUSTOMGAME_MINE : isize = 254;
    pub const IDS_HERO_NAME : isize = 261;
    pub const IDS_HERO_NAME_TEXT1 : isize = 262;
    pub const IDS_HERO_NAME_TEXT2 : isize = 263;
    pub const IDS_HERO_NAME_TEXT3 : isize = 264;
    pub const IDS_HERO_LIST : isize = 271;
    pub const IDS_HERO_LIST_TEXT1 : isize = 272;
    pub const IDS_HERO_LIST_TEXT2 : isize = 273;
    pub const IDS_HERO_LIST_TEXT3 : isize = 274;
    pub const IDS_HERO_LIST_BUTTON : isize = 275;
    pub const IDS_FILE_FILTER : isize = 281;
    pub const IDS_REPLAY_FILTER : isize = 282;
    pub const IDS_FILE_SAVE_ERROR : isize = 283;
    pub const IDS_FILE_LOAD_ERROR : isize = 284;
    pub const IDS_FILE_RECORD_FINISH : isize = 285;
    pub const IDS_FILE_PLAYBACK_FINISH : isize = 286;
    pub const IDS_APP_TITLE_RECORD : isize = 287;
    pub const IDS_FILE_RECORD_START : isize = 288;
    pub const IDS_FILE_RECORD_RESTART : isize = 289;
    pub const IDC_STATIC : isize = -1;
}

use self::resources::*;
use model::Board;
use model::BlockDisplayKind;
use model::GameButtonDisplayKind;

pub struct SpriteSheetInstance<'a, 'd, S: BorrowMut<LocalDeviceContext>, D: BorrowMut<ScopedDeviceContext<'d>>> {
    sprite_sheet: &'a dyn SpriteSheet,
    memory_dc: S,
    target_dc: D,
    sprite_bitmap: Option<Bitmap>,
    phantom: PhantomData<&'d ()>,
}

impl<'a, 'd, S, D> SpriteSheetInstance<'a, 'd, S, D>
    where S: BorrowMut<LocalDeviceContext>, D: BorrowMut<ScopedDeviceContext<'d>>
{
    #[allow(unreachable_patterns)]
    pub fn draw_sprite(&mut self, dest_pos: Point, idx: usize) -> apiw::Result<()> {
        let src_dc = self.memory_dc.borrow();
        let dest_dc = self.target_dc.borrow_mut();
        let (src_pos, size) = self.sprite_sheet.sprite_coord(idx);
        match self.sprite_sheet.sprite_draw_param() {
            SpriteDrawParam::TransparentBlt(key) => {
                dest_dc.transparentblt(src_dc, src_pos, size, dest_pos, size, key)?
            },
            SpriteDrawParam::BitBlt | _ => {
                dest_dc.bitblt(src_dc, src_pos, dest_pos, size, TenaryROP::SRCCOPY)?
            },
        };
        Ok(())
    }
}

impl<'a, 'd, S, D> Drop for SpriteSheetInstance<'a, 'd, S, D>
    where S: BorrowMut<LocalDeviceContext>, D: BorrowMut<ScopedDeviceContext<'d>>
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

    fn instantiate<'a, 'b, 'c>(&'a self, target_dc: &'b mut ScopedDeviceContext<'c>)
        -> apiw::Result<SpriteSheetInstance<'a, 'c, LocalDeviceContext, &'b mut ScopedDeviceContext<'c>>>
        where Self: Sized {
        let mut memory_dc = LocalDeviceContext::new_compatible_memory_dc(target_dc)?;
        let bitmap = self.bitmap_instance();
        memory_dc.select_bitmap(bitmap.clone())?;
        let sprite_sheet = self as _;
        Ok(SpriteSheetInstance {
            sprite_sheet,
            memory_dc,
            target_dc,
            sprite_bitmap: Some(bitmap),
            phantom: PhantomData
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
        (Point::new(0, (sprite_idx * Self::DIGIT_HEIGHT) as isize),
         Size::new(Self::DIGIT_WIDTH, Self::DIGIT_HEIGHT))
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
        (Point::new(0, (sprite_idx * Self::BUTTON_HEIGHT) as isize),
         Size::new(Self::BUTTON_WIDTH, Self::BUTTON_HEIGHT))
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

    pub const TRANSPARENT_COLOR: RGBColor = RGBColor::MAGENTA;

    pub fn new() -> Self {
        BlockSpriteSheet {
            instances: RefCell::new(smallvec![]),
        }
    }

    pub fn calc_block_area_size(board_size: (usize, usize)) -> Size {
        Size::new(
            Self::BLOCKSIZE_X + (board_size.1 - 1) * Self::BLOCKDELTA_X,
            Self::BLOCKSIZE_Y + (board_size.0 - 1) * Self::BLOCKDELTA_Y
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

            let point = Point::new(pos.1 as isize + shape_offset.1, pos.0 as isize + shape_offset.0);
            if let Some(color) = shape_dc.get_pixel(point)? {
                if color != Self::TRANSPARENT_COLOR {
                    return Ok(true);
                }
            }

            Ok(false)

        })
    }

    pub fn calc_block_pos(minefield_pos: Point, board_item: (usize, usize)) -> Point {
        minefield_pos.offset((board_item.1 * Self::BLOCKDELTA_X) as isize,
                             (board_item.0 * Self::BLOCKDELTA_Y) as isize)
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
        (Point::new(((sprite_idx % 2) * Self::BLOCKSIZE_X) as isize,
                    ((sprite_idx / 2) * Self::BLOCKSIZE_Y) as isize),
         Size::new(Self::BLOCKSIZE_X, Self::BLOCKSIZE_Y))
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
    pub title: String,
    pub window_class_name: String,

    pub digits_sheet: DigitSpriteSheet,
    pub gamebutton_sheet: GameButtonSpriteSheet,
    pub block_sheet: BlockSpriteSheet,
}

impl Assets {
    pub fn new() -> Self {
        //FIXME.

        Assets {
            title: "CharlesMine".into(),
            window_class_name: "CharlesMineWindow".into(),
            digits_sheet: DigitSpriteSheet::new(),
            gamebutton_sheet: GameButtonSpriteSheet::new(),
            block_sheet: BlockSpriteSheet::new(),
        }
    }
}
