use chrono::{DateTime, Local};
use std::fs::File;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;
use model::ModelCommand;
use model::Board;

#[derive(Clone, Debug)]
pub struct BoardSaved {
    pub board_size: (usize, usize),
    pub mine_pos: Rc<Vec<usize>>,
}

impl BoardSaved {
    pub fn import_from_board(board: &mut Board) -> Self {
        let board_size = board.size();
        let mine_pos =
            if let Some(mine_pos) = board.fixed_mine_pos_list().cloned() {
                mine_pos
            } else {
                let mine_pos_list = Rc::new(
                    board.snapshot_mine_pos_list().unwrap_or_else(|| board.allocate_mine_pos_list(None)));
                board.update_fixed_mine_pos_list(Some(mine_pos_list.clone()));
                mine_pos_list
            };
        BoardSaved {
            board_size,
            mine_pos,
        }
    }

    pub fn import_from_file(path: &Path) -> Option<Self> {
        unimplemented!()
    }

    pub fn export_to_file(&self, path: &Path) -> Result<(), ()> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub enum GameMode {
    Normal,
    GameRecording(DateTime<Local>, Rc<RefCell<File>>),
    BoardPredefined(BoardSaved),
    GamePlayback(BoardSaved, DateTime<Local>, Vec<ModelCommand>),
}

impl GameMode {
    pub fn is_normal(&self) -> bool {
        match self {
            GameMode::Normal => true,
            _ => false,
        }
    }

    pub fn is_predefined(&self) -> bool {
        match self {
            GameMode::BoardPredefined(_) => true,
            _ => false,
        }
    }

    pub fn is_recording(&self) -> bool {
        match self {
            GameMode::GameRecording(..) => true,
            _ => false,
        }
    }

    pub fn is_playback(&self) -> bool {
        match self {
            GameMode::GamePlayback(..) => true,
            _ => false,
        }
    }

    pub fn board_saved(&self) -> Option<&BoardSaved> {
        match self {
            GameMode::Normal => None,
            GameMode::GameRecording(..) => None,
            GameMode::BoardPredefined(board) => Some(board),
            GameMode::GamePlayback(board, ..) => Some(board),
        }
    }
}

