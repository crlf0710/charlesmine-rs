use crate::model::BlockShape;
use crate::model::GameButtonDisplayKind;
use std::cell::Cell;
use std::num::NonZeroUsize;
use crate::model_config::{self, Config};
use crate::view_assets::{self};
use crate::view_assets::BlockSpriteSheet;
use crate::view_assets::GameButtonSpriteSheet;
use crate::view_assets::{Assets, DigitSpriteSheet};
use crate::model::Block;
use crate::model::BlockDisplayKind;
use crate::model::Board;
use crate::model::Model;
use crate::model_config::BoardSettingKind;
use crate::model;
use crate::controller;
use crate::ui::{self, Size, Point, Rect, RGBColor};
use crate::ui::UiResult;
use crate::ui::UiScopedDC;
use crate::ui::UiWindow;
use crate::ui::UiLocalDC;
use crate::ui::UiDraw;
use crate::model_gamemode;
use crate::ui::ui_alert;

#[derive(Debug)]
pub enum AlertFailure {
    FileIOError
}

pub struct LayoutData {
    block_area_dims: (usize, usize),
    area_size: Size,
    block_area_size: Size,
    digit_pos_1: Point,
    digit_pos_2: Point,
    button_pos: Point,
}

impl LayoutData {
    pub const DIGITEDGE_LEFT: usize = 15;
    pub const DIGITEDGE_RIGHT: usize = 15;
    pub const DIGITEDGE_TOP: usize = 15;

    pub const BUTTONEDGE_TOP: usize = 15;

    pub const MIDDLE_BANNER: usize = 10;

    pub const BLOCK_AREA_EDGE_X: usize = 12;
    pub const BLOCK_AREA_EDGE_Y: usize = 12;
    pub const BLOCK_AREA_EDGE_TOP: usize = 12;

    pub const BLOCK_AREA_X: usize = Self::BLOCK_AREA_EDGE_X;
    pub const BLOCK_AREA_Y: usize = Self::DIGITEDGE_TOP
        + DigitSpriteSheet::DIGIT_HEIGHT
        + Self::MIDDLE_BANNER
        + Self::BLOCK_AREA_EDGE_TOP;
}

impl LayoutData {
    fn new(block_area_dims: (usize, usize)) -> Self {
        let block_area_size = BlockSpriteSheet::calc_block_area_size(block_area_dims);
        let area_size = Size::new(
            Self::BLOCK_AREA_X + block_area_size.cx() + Self::BLOCK_AREA_EDGE_X,
            Self::BLOCK_AREA_Y + block_area_size.cy() + Self::BLOCK_AREA_EDGE_Y,
        );

        let digit_pos_1 = Point::new(Self::DIGITEDGE_LEFT as _, Self::DIGITEDGE_TOP as _);

        let digit_pos_2 = Point::new(
            (area_size.cx()
                - Self::DIGITEDGE_RIGHT
                - DigitSpriteSheet::DIGIT_WIDTH * DigitPanel::DIGITCOUNT) as _,
            Self::DIGITEDGE_TOP as _,
        );

        let button_pos = Point::new(
            (area_size.cx() - GameButtonSpriteSheet::BUTTON_WIDTH) as isize / 2,
            Self::BUTTONEDGE_TOP as _,
        );
        Self {
            block_area_dims,
            area_size,
            block_area_size,
            digit_pos_1,
            digit_pos_2,
            button_pos,
        }
    }
}

#[derive(PartialEq)]
pub enum BorderPosition {
    Inner,
    Outer,
}

pub struct ThreeDimBorder {
    pub rect: Rect,
    pub border_pos: BorderPosition,
    pub color_nw: RGBColor,
    pub color_se: RGBColor,
}

impl ThreeDimBorder {
    const BORDER_WIDTH: usize = 2;
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

impl UiDraw for ThreeDimBorder {
    fn draw(self, dc: &mut UiScopedDC) -> UiResult<()> {
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

        dc.select_pen(pen1)?
            .move_to(rect.top_right())?
            .line_to(rect.top_left())?
            .line_to(rect.bottom_left())?
            .select_pen(pen2)?
            .line_to(rect.bottom_right())?
            .line_to(rect.top_right())?;

        Ok(())
    }
}

impl<'a> UiDraw for GameButton<'a> {
    fn draw(self, dc: &mut UiScopedDC) -> UiResult<()> {
        use crate::view_assets::SpriteSheet;

        let mut game_button_sheet = self.assets.gamebutton_sheet.instantiate(dc)?;
        let sprite_idx = GameButtonSpriteSheet::sprite_index(self.state);
        game_button_sheet.draw_sprite(self.pos, sprite_idx)?;

        Ok(())
    }
}


impl<'a> UiDraw for MineBlock<'a> {
    fn draw(self, dc: &mut UiScopedDC) -> UiResult<()> {
        use crate::view_assets::SpriteSheet;

        let mut block_sheet = self.assets.block_sheet.instantiate(dc)?;
        let sprite_idx =
            BlockSpriteSheet::sprite_index(self.block_shape_dir, self.block_display_kind);
        let block_draw_pos = BlockSpriteSheet::calc_block_pos(self.minefield_pos, self.block_pos);
        block_sheet.draw_sprite(block_draw_pos, sprite_idx)?;

        Ok(())
    }
}

impl<'a> UiDraw for DigitPanel<'a> {
    fn draw(self, dc: &mut UiScopedDC) -> UiResult<()> {
        use crate::view_assets::SpriteSheet;

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

            digits_sheet.draw_sprite(
                self.pos
                    .offset((i * DigitSpriteSheet::DIGIT_WIDTH) as isize, 0),
                sprite_idx,
            )?;
        }

        Ok(())
    }
}

struct GameButton<'a> {
    pos: Point,
    state: GameButtonDisplayKind,
    assets: &'a Assets,
}

struct MineBlock<'a> {
    minefield_pos: Point,
    block_pos: (usize, usize),
    block_shape_dir: BlockShape,
    block_display_kind: BlockDisplayKind,
    assets: &'a Assets,
}

impl Board {
    fn display_value_mine_left(&self) -> isize {
        self.goal_mark_count() as isize - self.cur_mark_count() as isize
    }

    fn display_value_time(&self) -> isize {
        use super::model::BoardStatus;
        use chrono::Local;
        use std::cmp::min;
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
    FieldBlock { y: usize, x: usize },
    Other,
}


struct LayoutState {
    button_pressed: Cell<bool>,
    window_captured: Cell<bool>,
    block_pressed: Cell<Option<(usize, usize, bool)>>,
}

impl LayoutState {
    fn new() -> Self {
        LayoutState {
            button_pressed: Cell::new(false),
            window_captured: Cell::new(false),
            block_pressed: Cell::new(None),
        }
    }
}

struct LayoutZoom {
    ratio: NonZeroUsize,
}

impl LayoutZoom {
    fn new() -> Self {
        Self::new_with_ratio(1)
    }

    fn new_with_ratio(v: usize) -> Self {
        LayoutZoom {
            ratio: NonZeroUsize::new(v).unwrap()
        }
    }
    fn update_dc(&self, dc: &mut UiScopedDC) -> UiResult<()> {
        use apiw::extensions::draw_ext::{GraphicsMode, Transform};
        let v = self.ratio.get();
        if v == 1 {
            return Ok(());
        }
        let v = v as f32;
        dc.set_graphics_mode(GraphicsMode::ADVANCED)?;
        dc.set_world_transform(&Transform::new_with_values(&[
            v, 0.0, 0.0, v, 0.0, 0.0
        ]))?;

        Ok(())
    }

    fn zoom_size(&self, size: Size) -> Size {
        let v = self.ratio.get();
        if v == 1 {
            return size;
        }
        Size::new(size.cx() * v, size.cy() * v)
    }

    fn unzoom_point(&self, point: Point) -> Point {
        let v = self.ratio.get();
        if v == 1 {
            return point;
        }
        Point::new(point.x() / v as isize, point.y() / v as isize)
    }
}

impl From<model_config::ZoomRatio> for LayoutZoom {
    fn from(r: model_config::ZoomRatio) -> Self {
        match r {
            model_config::ZoomRatio::Zoom1x => Self::new_with_ratio(1),
            model_config::ZoomRatio::Zoom2x => Self::new_with_ratio(2),
            model_config::ZoomRatio::Zoom3x => Self::new_with_ratio(3),
        }
    }
}


pub struct View {
    assets: Assets,
    window: Option<ui::UiWindow>,
    layout_data: LayoutData,
    layout_state: LayoutState,
    layout_zoom: LayoutZoom,
}

impl View {
    pub fn new(model: &Model) -> Self {
        let assets = view_assets::Assets::new();
        let block_area_dims = model.size();
        let layout_data = LayoutData::new(block_area_dims);
        let layout_state= LayoutState::new();
        let layout_zoom = LayoutZoom::new();

        View {
            assets,
            layout_data,
            layout_state,
            layout_zoom,

            window: None,
        }
    }

    pub fn update_zoom_ratio(&mut self, ratio: model_config::ZoomRatio) -> UiResult<()> {
        self.layout_zoom = LayoutZoom::from(ratio);
        self.adjust_window_layout().unwrap();
        Ok(())
    }

    pub fn regenerate_layout_data(&mut self, (y, x): (usize, usize)) {
        self.layout_data = LayoutData::new((y, x));
    }

    pub fn draw(&self, dc: &mut UiScopedDC, model: &Model, assets: &Assets) -> UiResult<()> {
        self.layout_zoom.update_dc(dc)?;
        dc
            .draw(ThreeDimBorder {
                rect: Rect::new(Point::ORIGIN, self.layout_data.area_size),
                border_pos: BorderPosition::Inner,
                color_nw: RGBColor::WHITE,
                color_se: RGBColor::GRAY,
            })?
            .draw(ThreeDimBorder {
                rect: Rect::new(
                    Point::new(LayoutData::BLOCK_AREA_X as _, LayoutData::BUTTONEDGE_TOP as _),
                    Size::new(
                        self.layout_data.block_area_size.cx(),
                        GameButtonSpriteSheet::BUTTON_HEIGHT,
                    ),
                ),
                border_pos: BorderPosition::Outer,
                color_nw: RGBColor::GRAY,
                color_se: RGBColor::WHITE,
            })?
            .draw(ThreeDimBorder {
                rect: Rect::new(
                    Point::new(LayoutData::BLOCK_AREA_X as _, LayoutData::BLOCK_AREA_Y as _),
                    self.layout_data.block_area_size,
                ),
                border_pos: BorderPosition::Outer,
                color_nw: RGBColor::GRAY,
                color_se: RGBColor::WHITE,
            })?
            .draw(DigitPanel {
                pos: self.layout_data.digit_pos_1,
                value: model.display_value_mine_left(),
                assets: &assets,
            })?
            .draw(DigitPanel {
                pos: self.layout_data.digit_pos_2,
                value: model.display_value_time(),
                assets: &assets,
            })?
            .draw(GameButton {
                pos: self.layout_data.button_pos,
                state: model.game_button_display_kind(self.layout_state.button_pressed.get(), self.layout_state.window_captured.get()),
                assets: &assets,
            })?
            .draw_from_iter((0..model.size().0).flat_map(move |y| {
                (0..model.size().1).map(move |x| MineBlock {
                    minefield_pos: Point::new(
                        LayoutData::BLOCK_AREA_X as isize,
                        LayoutData::BLOCK_AREA_Y as isize,
                    ),
                    block_pos: (y, x),
                    block_shape_dir: Board::block_shape(y, x),
                    block_display_kind: model.block_display_kind((y, x), self.layout_state.block_pressed.get()),
                    assets: &assets,
                })
            }))?;
        Ok(())
    }

    pub fn hit_test(&self, point: Point) -> GameTarget {
        let point = self.layout_zoom.unzoom_point(point);
        let button_size = Size::new(
            GameButtonSpriteSheet::BUTTON_WIDTH,
            GameButtonSpriteSheet::BUTTON_HEIGHT,
        );
        let button_rect = Rect::new(self.layout_data.button_pos, button_size);
        if button_rect.contains(point) {
            return GameTarget::GameButton;
        }

        // this is div_euc that is not stabilized yet.
        fn floor_div(lhs: isize, rhs: isize) -> isize {
            let q = lhs / rhs;
            if lhs % rhs < 0 {
                return if rhs > 0 { q - 1 } else { q + 1 };
            }
            q
        }

        let y_idx = floor_div(
            point.y() - LayoutData::BLOCK_AREA_Y as isize,
            BlockSpriteSheet::BLOCKSIZE_Y as isize,
        ) as isize;
        if 0 <= y_idx && y_idx < self.layout_data.block_area_dims.0 as isize {
            let x_idx_min = floor_div(
                point.x()
                    - LayoutData::BLOCK_AREA_X as isize
                    - (BlockSpriteSheet::BLOCKSIZE_X as isize - 1),
                BlockSpriteSheet::BLOCKDELTA_X as _,
            ) as isize;
            let x_idx_max = floor_div(
                point.x() - LayoutData::BLOCK_AREA_X as isize,
                BlockSpriteSheet::BLOCKDELTA_X as _,
            ) as isize;
            for x_idx in x_idx_min..=x_idx_max {
                if 0 <= x_idx && x_idx < self.layout_data.block_area_dims.1 as isize {
                    let x_offset = point.x() as usize
                        - LayoutData::BLOCK_AREA_X
                        - BlockSpriteSheet::BLOCKDELTA_X * x_idx as usize;
                    let y_offset = point.y() as usize
                        - LayoutData::BLOCK_AREA_Y
                        - BlockSpriteSheet::BLOCKDELTA_Y * y_idx as usize;
                    if x_offset >= BlockSpriteSheet::BLOCKSIZE_X {
                        continue;
                    }
                    if BlockSpriteSheet::hit_test_shape(
                        Board::block_shape(y_idx as usize, x_idx as usize),
                        (y_offset, x_offset),
                    ).expect("Failed to hit test.")
                        {
                            return GameTarget::FieldBlock {
                                y: y_idx as usize,
                                x: x_idx as usize,
                            };
                        }
                }
            }
        }
        GameTarget::Other
    }

    pub fn set_button_pressed(&self, pressed: bool) {
        self.layout_state.button_pressed.set(pressed);
    }

    pub fn set_window_captured(&self, window_captured: bool) {
        self.layout_state.window_captured.set(window_captured);
    }

    pub fn set_block_pressed(&self, y: usize, x: usize, blast: bool) {
        self.layout_state.block_pressed.set(Some((y, x, blast)));
    }

    pub fn unset_block_pressed(&self, y: usize, x: usize, blast: bool) {
        if self.layout_state.block_pressed.get() == Some((y, x, blast)) {
            self.layout_state.block_pressed.set(None)
        }
    }

    pub fn window(&self) -> Option<&UiWindow> {
        self.window.as_ref()
    }

    fn set_window(&mut self, window: UiWindow) {
        self.window = Some(window);
        self.adjust_window_layout().unwrap();
    }
}

#[derive(Debug)]
pub enum ViewCommand {
    Initialize,
    UpdateZoomRatio(model_config::ZoomRatio),
    UpdateUIBoardSetting(model_config::BoardSetting),
    UpdateUIAllowMarks(model_config::AllowMarks),
    UpdateUIZoomRatio(model_config::ZoomRatio),
    UpdateUIGameMode(model_gamemode::GameMode),
    SetButtonPressed(bool),
    SetBlockPressed(usize, usize, bool),
    UnsetBlockPressed(usize, usize, bool),
    AlertFailure(AlertFailure),
    Refresh,
    SetCapture,
    ReleaseCapture,
}

type ViewToken<'a> = ::domino::mvc::ViewToken<'a, model::Model, View, controller::Controller>;

impl ::domino::mvc::View<model::Model, controller::Controller> for View {
    type Command = ViewCommand;
    type OutputTarget = UiWindow;
    type OutputParameter = UiLocalDC;

    #[allow(unused_variables)]
    fn process_command(token: ViewToken, command: ViewCommand) {
        let _ = process_command_inner(token, command);

        fn process_command_inner(mut token: ViewToken, command: ViewCommand) -> UiResult<()> {
            match command {
                ViewCommand::Initialize => {
                    let allow_marks = token.model().config().allow_marks.clone();
                    token.exec_command_next(ViewCommand::UpdateUIAllowMarks(allow_marks));
                    let zoom_ratio = token.model().config().zoom_ratio.clone();
                    token.exec_command_next(ViewCommand::UpdateUIZoomRatio(zoom_ratio));
                    let board_setting = token.model().config().board_setting.clone();
                    token.exec_command_next(ViewCommand::UpdateUIBoardSetting(board_setting));
                    let game_mode = token.model().game_mode();
                    token.exec_command_next(ViewCommand::UpdateUIGameMode(game_mode));
                },
                ViewCommand::UpdateZoomRatio(v) => {
                    let view = token.view_mut();
                    view.update_zoom_ratio(v)?;
                    if let Some(window) = view.window() {
                        window.invalidate()?;
                    }
                },
                ViewCommand::UpdateUIBoardSetting(v) => {
                    let view = token.view_mut();
                    if let Some(window) = view.window() {
                        if let Some(mut menu) = window.menu().unwrap_or(None) {
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_FILE_GAME_EASY as _)
                                .set_checked(v.k == BoardSettingKind::Easy);
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_FILE_GAME_MEDIUM as _)
                                .set_checked(v.k == BoardSettingKind::Normal);
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_FILE_GAME_HARD as _)
                                .set_checked(v.k == BoardSettingKind::Hard);
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_FILE_GAME_CUSTOM as _)
                                .set_checked(v.k == BoardSettingKind::Custom);
                        }
                    }
                    view.regenerate_layout_data((v.y, v.x));
                    view.adjust_window_layout()?;
                },
                ViewCommand::UpdateUIAllowMarks(v) => {
                    let view = token.view_mut();
                    if let Some(window) = view.window() {
                        if let Some(mut menu) = window.menu().unwrap_or(None) {
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_FILE_MARK as _)
                                .set_checked(v.0);
                        }
                    }
                },
                ViewCommand::UpdateUIZoomRatio(v) => {
                    let view = token.view_mut();
                    if let Some(window) = view.window() {
                        if let Some(mut menu) = window.menu().unwrap_or(None) {
                            for &(e, menu_item) in &[
                                (model_config::ZoomRatio::Zoom1x, view_assets::resources::IDM_ADVANCED_ZOOM_1x),
                                (model_config::ZoomRatio::Zoom2x, view_assets::resources::IDM_ADVANCED_ZOOM_2x),
                                (model_config::ZoomRatio::Zoom3x, view_assets::resources::IDM_ADVANCED_ZOOM_3x),
                                ] {
                                let _ = menu
                                    .item_by_command(menu_item as _)
                                    .set_checked(v == e);
                            }
                        }
                    }
                },
                ViewCommand::UpdateUIGameMode(v) => {
                    let view = token.view_mut();
                    if let Some(window) = view.window() {
                        if let Some(mut menu) = window.menu().unwrap_or(None) {
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_ADVANCED_LOADMAP as _)
                                .set_enabled(v.is_normal() || v.is_predefined());
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_ADVANCED_SAVEMAP as _)
                                .set_enabled(v.is_normal() || v.is_predefined());
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_ADVANCED_RESTART as _)
                                .set_enabled(v.is_normal() || v.is_predefined());
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_ADVANCED_RECORD_PLAY as _)
                                .set_enabled(v.is_normal() || v.is_predefined());
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_ADVANCED_RECORD_RECORD as _)
                                .set_enabled(v.is_normal() || v.is_predefined());
                            let _ = menu
                                .item_by_command(view_assets::resources::IDM_ADVANCED_RECORD_STOP as _)
                                .set_enabled(!v.is_normal() && !v.is_predefined());
                        }
                    }
                }
                ViewCommand::AlertFailure(f) => {
                    ui_alert(&format!("{:?}", f));
                }
                ViewCommand::SetButtonPressed(v) => {
                    let view = token.view_mut();
                    view.set_button_pressed(v);
                }
                ViewCommand::SetBlockPressed(y, x, blast) => {
                    let view = token.view_mut();
                    view.set_block_pressed(y, x, blast);
                }
                ViewCommand::UnsetBlockPressed(y, x, blast) => {
                    let view = token.view_mut();
                    view.unset_block_pressed(y, x, blast);
                }
                ViewCommand::Refresh => {
                    let view = token.view_mut();
                    if let Some(window) = view.window() {
                        window.invalidate()?;
                    }
                },
                ViewCommand::SetCapture => {
                    let view = token.view_mut();
                    view.set_window_captured(true);
                    if let Some(window) = view.window() {
                        UiWindow::set_captured(Some(window))?;
                    }
                },
                ViewCommand::ReleaseCapture => {
                    let view = token.view_mut();
                    view.set_window_captured(false);
                    if let Some(window) = view.window() {
                        UiWindow::set_captured(None)?;
                    }
                }
            }
            Ok(())
        }
    }

    fn translate_model_notification(model_notification: ViewCommand) -> Option<Self::Command> {
        Some(model_notification)
    }

    fn redirect_output_target(&mut self, target: Option<Self::OutputTarget>) {
        if let Some(target) = target {
            self.set_window(target);
        }
    }

    #[allow(unused_variables)]
    fn sync_output_with_parameter(
        &self, model: &model::Model, param: &mut Self::OutputParameter
    ) {
        let _ = self.draw(param, model, &self.assets);
    }
}

impl View {
    fn adjust_window_layout(&self) -> UiResult<()> {
        if let Some(window) = self.window() {
            let rect = Rect::new(Point::ORIGIN, self.layout_zoom.zoom_size(self.layout_data.area_size));
            let new_rect = UiWindow::predict_window_rect_from_client_rect_and_window(rect, window)?;
            window.reposition_set_size(new_rect.size())?;
            window.invalidate_and_erase()?;
        }
        Ok(())
    }
}
