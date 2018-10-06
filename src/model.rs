#![allow(dead_code)]

use chrono::{DateTime, Local};
use rand::{self, distributions::Uniform, Rng};

use config::Config;
use std::cell::Cell;

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
}

impl Board {
    pub fn new(config: &Config) -> Board {
        let board_setting = config.board_setting();
        Self::new_inner(board_setting.y, board_setting.x, board_setting.c)
    }

    pub(crate) fn new_inner(y: usize, x: usize, c: usize) -> Board {
        Board {
            size: (y, x),
            count: c,
            rest_count: y * x,
            mark_count: 0,
            status: BoardStatus::Ready,
            blocks: vec![Default::default(); y * x],
        }
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

        let mapsize = self.size.0 * self.size.1;
        let chosen_idx = self.block_data_idx(y, x);

        self.blocks.clear();
        self.blocks
            .resize(self.size.0 * self.size.1, Default::default());

        let mut rng = rand::thread_rng();
        let mut mine_counter = 0;
        for mine_idx in Rng::sample_iter(&mut rng, &Uniform::new_inclusive(0, mapsize - 2))
            .map(|x| if x < chosen_idx { x } else { x + 1 })
            {
                if self.blocks[mine_idx].has_mine {
                    continue;
                }
                self.blocks[mine_idx].has_mine = true;
                mine_counter += 1;
                if mine_counter == self.count {
                    break;
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
        }{
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
                },
                _ => {},
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
            | BoardStatus::Finished(..)
            | BoardStatus::Died(..) => return,
            | _ => {},
        };

        match self.block_status(y, x) {
            BlockStatus::Open => {},
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
                BlockStatus::Normal | BlockStatus::MarkedQuestionable => {},
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

    pub(crate) fn rotate_block_state(&mut self, y: usize, x: usize) {
        let idx = self.block_data_idx(y, x);
        match self.status {
            | BoardStatus::Finished(..)
            | BoardStatus::Died(..) => return,
            | _ => {},
        };

        match self.blocks[idx].status {
            BlockStatus::Normal => {
                self.blocks[idx].status = BlockStatus::MarkedMine;
                self.mark_count += 1;
            },
            BlockStatus::MarkedMine => {
                self.blocks[idx].status = BlockStatus::MarkedQuestionable;
                self.mark_count -= 1;
            },
            BlockStatus::MarkedQuestionable => {
                self.blocks[idx].status = BlockStatus::Normal;
            },
            _ => {},
        }
    }

    pub(crate) fn block_display_kind(&self, pos: (usize, usize), focus: Option<(usize, usize, bool)>) -> BlockDisplayKind {
        let (y, x) = pos;
        let board_status = self.status();
        let block_display_number = self.block_display_number(y, x);
        let block_status = self.block_status(y, x);

        let (pressed, blast, focus_pos) =
            focus.as_ref()
                .map(|(focus_y, focus_x, blast)| (true, *blast, Some((*focus_y, *focus_x))))
                .unwrap_or((false, false, None));

        match block_status {
            BlockStatus::Normal => {
                match board_status {
                    BoardStatus::Died(..) => {
                        if block_display_number.is_none() {
                            BlockDisplayKind::NotMarkedMine
                        } else {
                            BlockDisplayKind::Normal
                        }
                    },
                    BoardStatus::Finished(..) => {
                        BlockDisplayKind::Normal
                    },
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
                    },
                }
            },
            BlockStatus::Open => {
                if let Some(n) = block_display_number {
                    BlockDisplayKind::OpenWithNumber(n)
                } else {
                    BlockDisplayKind::ExplodedMine
                }
            },
            BlockStatus::MarkedMine => {
                match board_status {
                    BoardStatus::Died(..) => {
                        if let Some(_) = block_display_number {
                            BlockDisplayKind::WrongMarkedMine
                        } else {
                            BlockDisplayKind::MarkedMine
                        }
                    },
                    _ => {
                        BlockDisplayKind::MarkedMine
                    }
                }
            }
            BlockStatus::MarkedQuestionable => {
                BlockDisplayKind::MarkedQuestionable
            }
        }
    }

    pub(crate) fn game_button_display_kind(&self, pressed: bool) -> GameButtonDisplayKind {
        match self.status {
            BoardStatus::Finished(..) => GameButtonDisplayKind::Finished,
            BoardStatus::Died(..) => GameButtonDisplayKind::Died,
            _ => {
                if pressed {
                    GameButtonDisplayKind::Pushed
                } else {
                    GameButtonDisplayKind::Normal
                }
            },
        }
    }
}

pub type Model = Board;