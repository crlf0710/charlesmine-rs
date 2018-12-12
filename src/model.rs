#![allow(dead_code)]

use chrono::{DateTime, Local};
use rand::{self, distributions::Uniform, Rng};

use std::ops;
use view::{self, ViewCommand};
use controller;
use model_config::{self, Config};
use model_gamemode::{self, BoardSaved, GameMode};
use std::cell::Cell;
use std::rc::Rc;
use std::collections::BTreeSet;
use std::path::PathBuf;
use view::AlertFailure;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum BlockStatus {
    Normal,
    Open,
    MarkedMine,
    MarkedQuestionable,
}

impl Default for BlockStatus {
    fn default() -> Self {
        BlockStatus::Normal
    }
}

#[derive(Clone, Default, Debug)]
pub struct Block {
    pub has_mine: bool,
    pub cached_number: Cell<Option<u8>>,
    pub status: BlockStatus,
}

pub enum BlockShape {
    DeltaLike,
    RevDeltaLike,
}

pub enum GameButtonDisplayKind {
    Normal,
    Pushed,
    Danger,
    Finished,
    Died,
}

pub enum BlockDisplayKind {
    Normal,
    MarkedMine,
    MarkedQuestionable,
    ExplodedMine,
    WrongMarkedMine,
    NotMarkedMine,
    PushMarkedQuestionable,
    OpenWithNumber(u8),
    PushNormal,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BoardStatus {
    Ready,
    Going(DateTime<Local>),
    Finished(DateTime<Local>, DateTime<Local>),
    Died(DateTime<Local>, DateTime<Local>),
}

pub struct Board {
    size: (usize, usize),
    count: usize,
    rest_count: usize,
    mark_count: usize,
    status: BoardStatus,
    blocks: Vec<Block>,
    fixed_mine_pos: Option<Rc<Vec<usize>>>,
    allow_marks: bool,
}

impl Board {
    pub(crate) fn new(y: usize, x: usize, c: usize) -> Board {
        Board {
            size: (y, x),
            count: c,
            rest_count: y * x,
            mark_count: 0,
            status: BoardStatus::Ready,
            blocks: vec![Default::default(); y * x],
            fixed_mine_pos: None,
            allow_marks: true,
        }
    }

    pub(crate) fn renew(&self) -> Board {
        let size = self.size();
        let count = self.goal_mark_count();
        let fixed_mine_pos = self.fixed_mine_pos.clone();
        let mut board = Board::new(size.0, size.1, count);
        board.fixed_mine_pos = fixed_mine_pos;
        board
    }

    pub fn size(&self) -> (usize, usize) {
        self.size
    }

    pub fn goal_mark_count(&self) -> usize {
        self.count
    }

    pub fn rest_count(&self) -> usize {
        self.rest_count
    }

    pub fn cur_mark_count(&self) -> usize {
        self.mark_count
    }

    pub fn status(&self) -> BoardStatus {
        self.status.clone()
    }

    pub fn fixed_mine_pos_list(&self) -> Option<&Rc<Vec<usize>>> {
        self.fixed_mine_pos.as_ref()
    }

    pub fn update_fixed_mine_pos_list(&mut self, list: Option<Rc<Vec<usize>>>) {
        self.fixed_mine_pos = list;
    }

    pub fn snapshot_mine_pos_list(&self) -> Option<Vec<usize>> {
        if self.status == BoardStatus::Ready {
            None
        } else {
            let mut result = Vec::new();
            for (mine_idx, block) in self.blocks.iter().enumerate() {
                if block.has_mine {
                    result.push(mine_idx);
                }
            }
            Some(result)
        }
    }

    pub fn allocate_mine_pos_list(&self, exclude_pos: Option<(usize, usize)>) -> Vec<usize> {
        let mapsize = self.size.0 * self.size.1;
        let mut exclude_set = BTreeSet::new();
        if let Some((y, x)) = exclude_pos {
            exclude_set.insert(self.block_data_idx(y, x));
        };
        let mut rng = rand::thread_rng();
        let mut result = Vec::new();
        for mine_idx in Rng::sample_iter(&mut rng, &Uniform::new_inclusive(0, mapsize - 1))
        {
            if exclude_set.replace(mine_idx).is_none() {
                result.push(mine_idx);
            }
            if result.len() == self.count {
                break;
            }
        }
        result
    }

    #[inline]
    fn block_data_idx(&self, y: usize, x: usize) -> usize {
        self.size.1 * y + x
    }

    pub fn block(&self, y: usize, x: usize) -> &Block {
        let idx = self.block_data_idx(y, x);
        &self.blocks[idx]
    }

    fn block_mut(&mut self, y: usize, x: usize) -> &mut Block {
        let idx = self.block_data_idx(y, x);
        &mut self.blocks[idx]
    }

    fn start_game_with(&mut self, y: usize, x: usize) {
        debug_assert!(y < self.size.0);
        debug_assert!(x < self.size.1);

        debug_assert!(self.status == BoardStatus::Ready);

        assert_eq!(self.blocks.len(), self.size.0 * self.size.1);
        if let Some(fixed_mine_pos) = self.fixed_mine_pos.as_ref() {
            for &mine_idx in fixed_mine_pos.iter() {
                assert!(self.blocks[mine_idx].status == BlockStatus::Normal);
                if self.blocks[mine_idx].has_mine {
                    continue;
                }
                self.blocks[mine_idx].has_mine = true;
            }
        } else {
            let mine_pos_list = self.allocate_mine_pos_list(Some((y, x)));
            for mine_idx in mine_pos_list {
                assert!(self.blocks[mine_idx].status == BlockStatus::Normal);
                if self.blocks[mine_idx].has_mine {
                    continue;
                }
                self.blocks[mine_idx].has_mine = true;
            }
        }

        for block in self.blocks.iter_mut() {
            if !block.has_mine {
                block.cached_number.set(None);
            }
        }
        //self.blocks = vec![Default::default()]
    }

    pub fn block_shape(y: usize, x: usize) -> BlockShape {
        match (y + x) % 2 {
            0 => BlockShape::RevDeltaLike,
            _ => BlockShape::DeltaLike,
        }
    }

    fn is_surrounding(pos: (usize, usize), check: (usize, usize)) -> bool {
        let (y, x) = pos;
        let (check_y, check_x) = check;
        let shape = Self::block_shape(y, x);
        if check_y == y || match shape {
            BlockShape::DeltaLike => check_y == y + 1,
            BlockShape::RevDeltaLike => check_y + 1 == y,
        } {
            if check_x + 2 >= x && x + 2 >= check_x {
                return true;
            }
        } else if match shape {
            BlockShape::DeltaLike => check_y + 1 == y,
            BlockShape::RevDeltaLike => check_y == y + 1,
        } {
            if check_x + 1 >= x && x + 1 >= check_x {
                return true;
            }
        }
        false
    }

    fn surrounding_blocks(y: isize, x: isize) -> [(isize, isize); 12] {
        [
            (y - 1, x - 1),
            (y - 1, x + 0),
            (y - 1, x + 1),
            (
                match Self::block_shape(y as usize, x as usize) {
                    BlockShape::DeltaLike => y + 1,
                    BlockShape::RevDeltaLike => y - 1,
                },
                x - 2,
            ),
            (y + 0, x - 2),
            (y + 0, x - 1),
            (y + 0, x + 1),
            (y + 0, x + 2),
            (
                match Self::block_shape(y as usize, x as usize) {
                    BlockShape::DeltaLike => y + 1,
                    BlockShape::RevDeltaLike => y - 1,
                },
                x + 2,
            ),
            (y + 1, x - 1),
            (y + 1, x + 0),
            (y + 1, x + 1),
        ]
    }

    fn is_index_in_range(board: &Board, y: isize, x: isize) -> bool {
        return y >= 0 && y < board.size.0 as isize && x >= 0 && x < board.size.1 as isize;
    }

    pub(crate) fn block_status(&self, y: usize, x: usize) -> BlockStatus {
        debug_assert!(y < self.size.0);
        debug_assert!(x < self.size.1);
        let idx = self.block_data_idx(y, x);
        self.blocks[idx].status
    }

    fn prepare_for_finish(&mut self) {
        for block in self.blocks.iter_mut() {
            match block.status {
                BlockStatus::Normal | BlockStatus::MarkedQuestionable => {
                    block.status = BlockStatus::MarkedMine;
                    self.mark_count += 1;
                }
                _ => {}
            }
        }
    }

    pub(crate) fn block_display_number(&self, y: usize, x: usize) -> Option<u8> {
        debug_assert!(y < self.size.0);
        debug_assert!(x < self.size.1);
        let idx = self.block_data_idx(y, x);
        if self.blocks[idx].has_mine {
            return None;
        }

        if self.blocks[idx].cached_number.get().is_none() {
            let mut number = 0_u8;
            for &(y, x) in &Self::surrounding_blocks(y as isize, x as isize) {
                if !Self::is_index_in_range(&self, y, x) {
                    continue;
                }
                if self.block(y as usize, x as usize).has_mine {
                    number += 1;
                }
            }
            self.blocks[idx].cached_number.set(Some(number));
            return Some(number);
        }
        return self.blocks[idx].cached_number.get();
    }

    pub(crate) fn blast_block(&mut self, y: usize, x: usize) {
        use std::collections::VecDeque;

        match self.status {
            | BoardStatus::Finished(..) | BoardStatus::Died(..) => return,
            | _ => {}
        };

        match self.block_status(y, x) {
            BlockStatus::Open => {}
            _ => return,
        }

        let mut marked_number = 0;
        for &(y, x) in &Self::surrounding_blocks(y as isize, x as isize) {
            if !Self::is_index_in_range(&self, y, x) {
                continue;
            }
            if self.block(y as usize, x as usize).status == BlockStatus::MarkedMine {
                marked_number += 1;
            }
        }

        if self.block_display_number(y, x) != Some(marked_number) {
            return;
        }

        let mut queue = VecDeque::new();
        for &(y, x) in &Self::surrounding_blocks(y as isize, x as isize) {
            if !Self::is_index_in_range(&self, y, x) {
                continue;
            }
            queue.push_back((y as usize, x as usize));
        }

        let mut exploded = false;
        while let Some((y, x)) = queue.pop_front() {
            let n = self.block_display_number(y, x);
            if self.block(y, x).status != BlockStatus::Normal {
                continue;
            }

            self.block_mut(y, x).status = BlockStatus::Open;
            self.rest_count -= 1;

            if n == Some(0) {
                for &(y, x) in &Self::surrounding_blocks(y as isize, x as isize) {
                    if !Self::is_index_in_range(&self, y, x) {
                        continue;
                    }

                    queue.push_back((y as usize, x as usize));
                }
            } else if n == None {
                exploded = true;
            }
        }
        if let BoardStatus::Going(start_time) = self.status {
            if self.rest_count == self.count {
                self.prepare_for_finish();
                self.status = BoardStatus::Finished(start_time, Local::now());
            } else if exploded {
                self.status = BoardStatus::Died(start_time, Local::now());
            }
        }
    }

    pub(crate) fn open_block(&mut self, y: usize, x: usize) {
        debug_assert!(y < self.size.0);
        debug_assert!(x < self.size.1);

        if self.status == BoardStatus::Ready {
            self.start_game_with(y, x);
            self.status = BoardStatus::Going(Local::now());
        }

        if let BoardStatus::Going(_) = self.status {
            match self.block_status(y, x) {
                BlockStatus::Normal | BlockStatus::MarkedQuestionable => {}
                _ => return,
            }

            if let Some(n) = self.block_display_number(y, x) {
                self.block_mut(y, x).status = BlockStatus::Open;
                self.rest_count -= 1;

                if let BoardStatus::Going(start_time) = self.status {
                    if self.rest_count == self.count {
                        self.prepare_for_finish();
                        self.status = BoardStatus::Finished(start_time, Local::now());
                    }
                }

                if n == 0 {
                    self.blast_block(y, x);
                }
            } else {
                self.block_mut(y, x).status = BlockStatus::Open;
                if let BoardStatus::Going(start_time) = self.status {
                    self.status = BoardStatus::Died(start_time, Local::now());
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub(crate) fn set_allow_marks(&mut self, allow_marks: bool) {
        self.allow_marks = allow_marks;
    }

    pub(crate) fn rotate_block_state(&mut self, y: usize, x: usize) {
        let idx = self.block_data_idx(y, x);
        match self.status {
            | BoardStatus::Finished(..) | BoardStatus::Died(..) => return,
            | _ => {}
        };

        match self.blocks[idx].status {
            BlockStatus::Normal => {
                self.blocks[idx].status = BlockStatus::MarkedMine;
                self.mark_count += 1;
            }
            BlockStatus::MarkedMine if self.allow_marks == true => {
                self.blocks[idx].status = BlockStatus::MarkedQuestionable;
                self.mark_count -= 1;
            }
            BlockStatus::MarkedMine if self.allow_marks == false => {
                self.blocks[idx].status = BlockStatus::Normal;
                self.mark_count -= 1;
            }
            BlockStatus::MarkedQuestionable => {
                self.blocks[idx].status = BlockStatus::Normal;
            }
            _ => {}
        }
    }

    pub(crate) fn block_display_kind(
        &self,
        pos: (usize, usize),
        focus: Option<(usize, usize, bool)>,
    ) -> BlockDisplayKind {
        let (y, x) = pos;
        let board_status = self.status();
        let block_display_number = self.block_display_number(y, x);
        let block_status = self.block_status(y, x);

        let (pressed, blast, focus_pos) = focus
            .as_ref()
            .map(|(focus_y, focus_x, blast)| (true, *blast, Some((*focus_y, *focus_x))))
            .unwrap_or((false, false, None));

        match block_status {
            BlockStatus::Normal => match board_status {
                BoardStatus::Died(..) => {
                    if block_display_number.is_none() {
                        BlockDisplayKind::NotMarkedMine
                    } else {
                        BlockDisplayKind::Normal
                    }
                }
                BoardStatus::Finished(..) => BlockDisplayKind::Normal,
                _ => {
                    if pressed {
                        if blast {
                            if Self::is_surrounding((y, x), focus_pos.unwrap()) {
                                BlockDisplayKind::PushNormal
                            } else {
                                BlockDisplayKind::Normal
                            }
                        } else {
                            if Some((y, x)) == focus_pos {
                                BlockDisplayKind::PushNormal
                            } else {
                                BlockDisplayKind::Normal
                            }
                        }
                    } else {
                        BlockDisplayKind::Normal
                    }
                }
            },
            BlockStatus::Open => {
                if let Some(n) = block_display_number {
                    BlockDisplayKind::OpenWithNumber(n)
                } else {
                    BlockDisplayKind::ExplodedMine
                }
            }
            BlockStatus::MarkedMine => match board_status {
                BoardStatus::Died(..) => {
                    if let Some(_) = block_display_number {
                        BlockDisplayKind::WrongMarkedMine
                    } else {
                        BlockDisplayKind::MarkedMine
                    }
                }
                _ => BlockDisplayKind::MarkedMine,
            },
            BlockStatus::MarkedQuestionable => match board_status {
                BoardStatus::Died(..) => {
                    if block_display_number.is_none() {
                        BlockDisplayKind::NotMarkedMine
                    } else {
                        BlockDisplayKind::MarkedQuestionable
                    }
                }
                BoardStatus::Finished(..) => BlockDisplayKind::MarkedQuestionable,
                _ => {
                    if pressed {
                        if blast {
                            if Self::is_surrounding((y, x), focus_pos.unwrap()) {
                                BlockDisplayKind::PushMarkedQuestionable
                            } else {
                                BlockDisplayKind::MarkedQuestionable
                            }
                        } else {
                            if Some((y, x)) == focus_pos {
                                BlockDisplayKind::PushMarkedQuestionable
                            } else {
                                BlockDisplayKind::MarkedQuestionable
                            }
                        }
                    } else {
                        BlockDisplayKind::MarkedQuestionable
                    }
                }
            },
        }
    }

    pub(crate) fn game_button_display_kind(&self, pressed: bool, captured: bool) -> GameButtonDisplayKind {
        if pressed {
            GameButtonDisplayKind::Pushed
        } else {
            match self.status {
                BoardStatus::Finished(..) => GameButtonDisplayKind::Finished,
                BoardStatus::Died(..) => GameButtonDisplayKind::Died,
                _ => if captured {
                    GameButtonDisplayKind::Danger
                } else {
                    GameButtonDisplayKind::Normal
                },
            }
        }
    }
}

pub struct Model {
    config: Config,
    game_mode: GameMode,
    board: Board,
}

impl Model {
    pub fn new() -> Model {
        let config = Config::new();

        let game_mode = GameMode::Normal;

        let board = {
            let board_setting = &config.board_setting;
            let mut board= Board::new(board_setting.y, board_setting.x, board_setting.c);
            let allow_marks = &config.allow_marks;
            board.allow_marks = allow_marks.0;
            board
        };

        Model {
            config,
            board,
            game_mode,
        }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn game_mode(&self) -> GameMode { self.game_mode.clone() }
}


#[derive(Clone, Debug)]
pub enum ModelCommand {
    Initialize,

    NewGame,
    NewGameWithBoard(model_config::BoardSetting),

    OpenBlock(usize, usize),
    BlastBlock(usize, usize),
    RotateBlockState(usize, usize),

    ToggleAllowMarks,

    SaveMap(PathBuf),
    LoadMap(PathBuf),
    RestartGame,

    EffectNewGameButtonDown,
    EffectNewGameButtonUp,
    EffectPushBlock { x: usize, y: usize },
    EffectPopBlock { x: usize, y: usize },
    EffectBlastDownBlock { x: usize, y: usize },
    EffectBlastUpBlock { x: usize, y: usize },

    EffectCapture,
    EffectUnCapture,
}

impl ops::Deref for Model {
    type Target = Board;

    fn deref(&self) -> &Board {
        &self.board
    }
}

impl ops::DerefMut for Model {
    fn deref_mut(&mut self) -> &mut Board {
        &mut self.board
    }
}

type ModelToken<'a> = ::domino::mvc::ModelToken<'a, Model, view::View, controller::Controller>;

impl ::domino::mvc::Model<view::View, controller::Controller> for Model {
    type Command = ModelCommand;
    type Notification = view::ViewCommand;

    fn process_command(mut token: ModelToken, command: ModelCommand) {
            match command {
                ModelCommand::Initialize => {
                    token.update_view_next(ViewCommand::Initialize);
                },
                ModelCommand::NewGameWithBoard(v) => {
                    {
                        let model = token.model_mut();
                        model.board = Board::new(v.y, v.x, v.c);
                    }
                    token.update_view_next(
                        ViewCommand::UpdateUIBoardSetting(v.clone()));
                },
                ModelCommand::NewGame => {
                    let model = token.model_mut();
                    let size = model.board.size();
                    let count = model.board.goal_mark_count();
                    model.board =  Board::new(size.0, size.1, count);
                }
                ModelCommand::LoadMap(path) => {
                    let new_gamemode;
                    {
                        let board_saved = if let Some(board_saved) = BoardSaved::import_from_file(&path) {
                            board_saved
                        } else {
                            token.update_view_next(ViewCommand::AlertFailure(AlertFailure::FileIOError));
                            return;
                        };
                        let model = token.model_mut();
                        model.board = Board::new(board_saved.board_size.0, board_saved.board_size.1, board_saved.mine_pos.len());
                        model.fixed_mine_pos = Some(board_saved.mine_pos.clone());

                        model.game_mode = GameMode::BoardPredefined(board_saved);
                        new_gamemode = model.game_mode();
                    }
                    token.update_view_next(ViewCommand::UpdateUIGameMode(new_gamemode));
                }
                ModelCommand::SaveMap(path) => {
                    let new_gamemode;
                    {
                        if !token.model_mut().game_mode.is_predefined() {
                            let model = token.model_mut();
                            let board_saved = BoardSaved::import_from_board(&mut model.board);
                            model.game_mode = GameMode::BoardPredefined(board_saved);
                        }
                        new_gamemode = token.model_mut().game_mode();
                        if !new_gamemode.board_saved().unwrap()
                            .export_to_file(&path).is_err() {
                            token.update_view_next(ViewCommand::AlertFailure(AlertFailure::FileIOError));
                        }
                    }
                    token.update_view_next(ViewCommand::UpdateUIGameMode(new_gamemode));
                }
                ModelCommand::RestartGame => {
                    let new_gamemode;
                    {
                        let model = token.model_mut();
                        if !model.game_mode.is_predefined() {
                            let board_saved = BoardSaved::import_from_board(&mut model.board);
                            model.game_mode = GameMode::BoardPredefined(board_saved);
                        }
                        new_gamemode = model.game_mode();
                        model.board = model.board.renew();
                    }
                    token.update_view_next(ViewCommand::UpdateUIGameMode(new_gamemode));
                }
                ModelCommand::OpenBlock(y, x) => {
                    let model = token.model_mut();
                    model.open_block(y, x);
                }
                ModelCommand::BlastBlock(y, x) => {
                    let model = token.model_mut();
                    model.blast_block(y, x);
                }
                ModelCommand::RotateBlockState(y, x) => {
                    let model = token.model_mut();
                    model.rotate_block_state(y, x);
                }
                ModelCommand::ToggleAllowMarks => {
                    let new_state;
                    {
                        let model = token.model_mut();
                        new_state = !model.config.allow_marks.0;
                        model.board.set_allow_marks(new_state);
                        model.config.allow_marks = model_config::AllowMarks(new_state);
                    }
                    token.update_view_next(ViewCommand::UpdateUIAllowMarks(model_config::AllowMarks(new_state)));
                }
                ModelCommand::EffectNewGameButtonDown => {
                    token.update_view_next(ViewCommand::SetButtonPressed(true));
                }
                ModelCommand::EffectNewGameButtonUp => {
                    token.update_view_next(ViewCommand::SetButtonPressed(false));
                }
                ModelCommand::EffectPushBlock { y, x } => {
                    token.update_view_next(ViewCommand::SetBlockPressed(y, x, false));
                }
                ModelCommand::EffectPopBlock { y, x } => {
                    token.update_view_next(ViewCommand::UnsetBlockPressed(y, x, false));
                }
                ModelCommand::EffectBlastDownBlock { y, x } => {
                    token.update_view_next(ViewCommand::SetBlockPressed(y, x, true));
                }
                ModelCommand::EffectBlastUpBlock { y, x } => {
                    token.update_view_next(ViewCommand::UnsetBlockPressed(y, x, true));
                }
                ModelCommand::EffectCapture => {
                    token.update_view_next(ViewCommand::SetCapture);
                }
                ModelCommand::EffectUnCapture => {
                    token.update_view_next(ViewCommand::ReleaseCapture);
                }
            }

            token.update_view_next(ViewCommand::Refresh);
    }

    fn translate_controller_notification(controller_notification: ModelCommand) -> Option<Self::Command> {
        Some(controller_notification)
    }
}
