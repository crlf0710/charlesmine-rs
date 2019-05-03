#![windows_subsystem = "windows"]
#![allow(unused_imports, unreachable_code, unused_variables, dead_code)]

use crate::ui::Ui;
use std::cell::RefCell;

mod controller;
mod model;
mod model_config;
mod model_gamemode;
#[path = "ui_apiw.rs"]
mod ui;
mod view;
mod view_assets;

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
        Game { mvc }
    }
}

thread_local! {
    static THE_GAME: RefCell<Game> = RefCell::new(Game::new());
}

fn main() -> apiw::Result<()> {
    use crate::view::View;

    env_logger::init();

    Ui::initialization()?;

    Ui::run_event_loop()?;

    return Ok(());
}
