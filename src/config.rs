#[derive(Copy, Clone)]
pub struct BoardSetting {
    pub x: usize,
    pub y: usize,
    pub c: usize,
}

use clamp::clamp;

impl BoardSetting {
    const EASY: BoardSetting = BoardSetting {
        x: 11,
        y: 10,
        c: 10,
    };
    const NORMAL: BoardSetting = BoardSetting {
        x: 21,
        y: 15,
        c: 50,
    };
    const HARD: BoardSetting = BoardSetting {
        x: 41,
        y: 15,
        c: 99,
    };

    pub fn new_normalized(mut x: usize, mut y: usize, mut c: usize) -> Self {
        x = clamp(Self::EASY.x, x, Self::HARD.x);
        y = clamp(Self::EASY.y, y, Self::HARD.y);
        c = clamp(Self::EASY.c, c, (x - 1) * (y - 1));
        BoardSetting { x, y, c }
    }
}

/*
#define MAPX_MIN       MODE_EASY_MAPX
#define MAPX_MAX       MODE_HARD_MAPX
#define MAPY_MIN       MODE_EASY_MAPY
#define MAPY_MAX       MODE_HARD_MAPY
#define MINE_MIN       MODE_EASY_MINE
#define MINE_MAX(x,y)  ((x-1)*(y-1))

*/

pub struct Config;

impl Config {
    pub fn new() -> Self {
        Config
    }

    pub fn board_setting(&self) -> BoardSetting {
        //FIXME
        BoardSetting::EASY
    }
}
