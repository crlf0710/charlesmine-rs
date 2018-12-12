#![windows_subsystem = "windows"]
#![allow(unused_imports, unreachable_code)]

#[macro_use]
extern crate apiw;
extern crate chrono;
extern crate clamp;
extern crate rand;
#[macro_use]
extern crate smallvec;
extern crate concerto;
extern crate domino;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::cell::RefCell;
use ui::Ui;

mod model;
mod model_config;
mod model_gamemode;
mod view;
mod view_assets;
#[path = "ui_apiw.rs"]
mod ui;
mod controller;

type GameMVC = domino::mvc::MVCSystem<model::Model, view::View, controller::Controller>;

struct Game {
    pub mvc: GameMVC,
}

impl Game {
    fn new() -> Self {
        let model = model::Model::new();
        let view = view::View::new(&model);
        let controller = controller::Controller::new(&model);
        let mvc = GameMVC::new(model, view, controller);
        Game {
            mvc,
        }
    }
}

thread_local! {
    static THE_GAME: RefCell<Game> = RefCell::new(Game::new());
}

fn main() -> apiw::Result<()> {
    use view::View;

    env_logger::init();

    Ui::initialization()?;

    Ui::run_event_loop()?;

    return Ok(());
}
