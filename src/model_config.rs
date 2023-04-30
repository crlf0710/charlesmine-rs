use std::cell::RefCell;
use std::rc::Rc;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum BoardSettingKind {
    Easy,
    Normal,
    Hard,
    Custom,
}

#[derive(Clone, Debug)]
pub struct BoardSetting {
    pub x: usize,
    pub y: usize,
    pub c: usize,
    pub k: BoardSettingKind,
}

impl BoardSetting {
    pub const EASY: BoardSetting = BoardSetting {
        x: 11,
        y: 10,
        c: 10,
        k: BoardSettingKind::Easy,
    };
    pub const NORMAL: BoardSetting = BoardSetting {
        x: 21,
        y: 15,
        c: 50,
        k: BoardSettingKind::Normal,
    };
    pub const HARD: BoardSetting = BoardSetting {
        x: 41,
        y: 15,
        c: 99,
        k: BoardSettingKind::Hard,
    };

    pub fn new_custom(mut x: usize, mut y: usize, mut c: usize) -> Self {
        x = Ord::clamp(Self::EASY.x, x, Self::HARD.x);
        y = Ord::clamp(Self::EASY.y, y, Self::HARD.y);
        c = Ord::clamp(Self::EASY.c, c, (x - 1) * (y - 1));
        BoardSetting {
            x,
            y,
            c,
            k: BoardSettingKind::Custom,
        }
    }
}

impl Default for BoardSetting {
    fn default() -> Self {
        BoardSetting::EASY
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AllowMarks(pub bool);

impl Default for AllowMarks {
    fn default() -> Self {
        AllowMarks(true)
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ZoomRatio {
    Zoom1x,
    Zoom2x,
    Zoom3x,
}

impl Default for ZoomRatio {
    fn default() -> Self {
        ZoomRatio::Zoom1x
    }
}

#[derive(Default)]
pub struct Config {
    pub board_setting: BoardSetting,
    pub allow_marks: AllowMarks,
    pub zoom_ratio: ZoomRatio,
}

impl Config {
    pub fn new() -> Self {
        Default::default()
    }
}
