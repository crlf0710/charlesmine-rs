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
#[macro_use] extern crate log;
extern crate env_logger;

mod assets;
mod model;
mod config;
mod layout;
mod controller;

struct Game {
    pub assets: assets::Assets,
    pub config: config::Config,
    pub model: model::Model,
    pub layout: layout::Layout,
    pub controller: controller::Controller,
}

impl Game {
    fn new() -> Self {
        let assets = assets::Assets::new();
        let config = config::Config::new();
        let model = model::Model::new(&config);
        let layout = layout::Layout::new(&model);
        let controller = controller::Controller::new();
        Game {
            config,
            assets,
            model,
            layout,
            controller,
        }
    }
}

use apiw::graphics_subsystem::device_context::ScopedDeviceContext;

impl<'a> apiw::graphics_subsystem::draw::Draw for &'a Game {
    fn draw(
        self,
        dc: &mut ScopedDeviceContext,
    ) -> apiw::Result<()> {
        self.layout.draw(dc, &self.model, &self.assets)
    }
}

use std::cell::RefCell;

thread_local! {
    static NEW_GAME: RefCell<Game> = RefCell::new(Game::new());
}

fn main() -> apiw::Result<()> {
    env_logger::init();

    create_main_window()?;

    run_event_loop()?;

    return Ok(());

    fn create_main_window() -> apiw::Result<()> {
        use apiw::windows_subsystem::window::WindowBuilder;
        use apiw::windows_subsystem::window::WindowClassBuilder;
        use apiw::windows_subsystem::window::WindowProcRequest;
        use apiw::windows_subsystem::window::WindowStyles;

        fn main_window_handler(mut request: WindowProcRequest) {
            use apiw;
            use apiw::windows_subsystem::window::ForeignWindow;
            use apiw::windows_subsystem::window::MouseEventArgs;

            request
                .route_paint(|window: &ForeignWindow| -> apiw::Result<()> {
                    let mut paint_dc = window.do_paint()?;

                    NEW_GAME.with(|game| {
                        let game = game.try_borrow().or_else(|_| apiw::last_error())?;
                        paint_dc.draw(&*game)
                    })?;

                    Ok(())
                })
                .route_mouse(|window: &ForeignWindow, args: MouseEventArgs| -> apiw::Result<bool> {
                    NEW_GAME.with(|game| {
                        let mut game = game.try_borrow_mut().or_else(|_| apiw::last_error())?;
                        let game = &mut *game;
                        if game.controller.send_mouse_input(args, &game.layout, &game.model) {
                            if game.controller.flush_commands(&mut game.layout, &mut game.model) {
                                window.invalidate()?;
                            }
                        }
                        Ok(())
                    })?;
                    Ok(true)
                })
                .route_close(|_window: &ForeignWindow| -> apiw::Result<()> {
                    unsafe {
                        apiw::full_windows_api::um::winuser::PostQuitMessage(0);
                    }

                    Ok(())
                });
        }

        use apiw::windows_subsystem::window::TimerProcRequest;

        fn main_window_timer_handler(mut request: TimerProcRequest) {
            if let Some(window) = request.window() {
                let _ = window.invalidate();
            }
        }

        NEW_GAME.with(|game| {
            use layout::AdjustWithLayout;
            let game = game.borrow();

            let window_class = WindowClassBuilder::new(&game.assets.window_class_name)
                .syscolor_background_brush(apiw::windows_subsystem::window::SysColor::BUTTON_FACE)
                .syscursor(apiw::windows_subsystem::window::SysCursor::ARROW)
                .window_proc(window_proc!(main_window_handler))
                .create_managed()?;
            /*
			wcex.style			= CS_HREDRAW | CS_VREDRAW;
			wcex.hIcon			= LoadIcon(hInstance, MAKEINTRESOURCE(IDI_CHARLESMINE));
			wcex.lpszMenuName	= MAKEINTRESOURCE(IDC_CHARLESMINE);
			wcex.hIconSm		= wcex.hIcon;
		*/

            let window = WindowBuilder::new(&window_class)
                .name(&game.assets.title)
                .style(
                    WindowStyles::CAPTION
                        | WindowStyles::VISIBLE
                        | WindowStyles::CLIPSIBLINGS
                        | WindowStyles::SYSMENU
                        | WindowStyles::OVERLAPPED
                        | WindowStyles::MINIMIZEBOX,
                )
                .create()?;

            use std::num::NonZeroUsize;
            use std::time::Duration;

            window
                .adjust_with_layout(&game.layout)?
                .set_timer_with_id(NonZeroUsize::new(1).unwrap(),
                    Duration::from_millis(100),
                    timer_proc!(main_window_timer_handler)
                )?
                .show(apiw::utils::exe_cmd_show())?
                .update()?;
            Ok(())
        })
    }

    fn run_event_loop() -> apiw::Result<()> {
        use apiw::windows_subsystem::ForeignMessageLoop;
        let mut msgloop = ForeignMessageLoop::for_current_thread();

        while let Some(msg) = msgloop.poll_wait()?.not_quit() {
            let _ = msg.dispatch();
        }

        Ok(())
    }
}
