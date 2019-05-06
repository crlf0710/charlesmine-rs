#![allow(unused_imports, unreachable_code, unused_variables, dead_code)]

use apiw;
use apiw::shared::ManagedStrategy;
use apiw::Result;

use crate::Game;
use crate::THE_GAME;

use crate::controller::{self, ControllerInput};
use crate::model;
use crate::view;
use crate::view::ViewCommand;
use crate::view_assets;
use apiw::application_support_functions::MessageBoxBuilder;
use apiw::timer_proc;
use apiw::window_proc;
use domino::mvc::ViewToken;

pub type UiWindow = apiw::windows_subsystem::window::ForeignWindow;
pub type UiScopedDC<'a> = apiw::graphics_subsystem::device_context::ScopedDeviceContext<'a>;
pub type UiLocalDC = apiw::graphics_subsystem::device_context::LocalDeviceContext;
pub type UiResult<T> = apiw::Result<T>;

pub type Size = apiw::graphics_subsystem::Size;
pub type Point = apiw::graphics_subsystem::Point;
pub type Rect = apiw::graphics_subsystem::Rect;
pub type RGBColor = apiw::graphics_subsystem::RGBColor;

use crate::model_config;
use apiw::application_support_functions::OpenFileDialogBuilder;
use apiw::application_support_functions::OpenFileDialogFlags;
use apiw::application_support_functions::SaveFileDialogBuilder;
use apiw::application_support_functions::SaveFileDialogFlags;
pub use apiw::graphics_subsystem::draw::Draw as UiDraw;
use std::path::PathBuf;

pub fn ui_alert(msg: &str) {
    MessageBoxBuilder::new().message(msg).invoke().unwrap();
}

pub struct Ui;

impl Ui {
    pub(crate) fn initialization() -> apiw::Result<()> {
        #[cfg(windows)]
        {
            use apiw::full_windows_api::shared::ntdef::LANG_ENGLISH;
            use apiw::full_windows_api::shared::ntdef::SUBLANG_ENGLISH_US;
            use apiw::full_windows_api::um::winnls::SetThreadUILanguage;
            use apiw::full_windows_api::um::winnt::MAKELANGID;

            //unsafe {
            // SetThreadUILanguage(MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US));
            //}
        }

        Self::create_main_window()?;

        THE_GAME.with(|game| {
            let mut game = game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
            let game = &mut *game;
            game.mvc.process_input(ControllerInput::Initialize);
            Ok(())
        })?;

        Ok(())
    }

    fn call_open_file_dialog(
        parent: &UiWindow,
        filter_res_id: usize,
        default_ext: &str,
    ) -> Option<PathBuf> {
        //FIXME: filter.
        OpenFileDialogBuilder::new()
            .parent(parent)
            .default_extension(default_ext)
            .flags(
                OpenFileDialogFlags::SHOW_HELP
                    | OpenFileDialogFlags::PATH_MUST_EXIST
                    | OpenFileDialogFlags::FILE_MUST_EXIST
                    | OpenFileDialogFlags::EXPLORER,
            )
            .show_dialog()
            .expect("Error occurred")
    }

    fn call_save_file_dialog(
        parent: &UiWindow,
        filter_res_id: usize,
        default_ext: &str,
    ) -> Option<PathBuf> {
        //FIXME: filter.
        SaveFileDialogBuilder::new()
            .parent(parent)
            .default_extension(default_ext)
            .flags(
                SaveFileDialogFlags::SHOW_HELP
                    | SaveFileDialogFlags::PATH_MUST_EXIST
                    | SaveFileDialogFlags::EXPLORER,
            )
            .show_dialog()
            .expect("Error occurred")
    }
    /*

    BOOL HandleMapFile(bool bSave, UINT nFilterResID, LPCTSTR lpszDefExt, LPTSTR lpszFile)
    {
        OPENFILENAME ofn;
        TCHAR szFilter[MAX_LOADSTRING];
        ZeroMemory(&ofn, sizeof(ofn));
        ofn.hwndOwner = hGame_MainWnd;
        ofn.lStructSize = sizeof(ofn);
        ofn.lpstrFile = lpszFile;
        ofn.lpstrFile[0] = '\0';
        ofn.nMaxFile = MAX_PATH;
        LoadString(hInst, nFilterResID, szFilter, MAX_LOADSTRING);
        for (int i=0; szFilter[i]!='\0'; i++)
            if (szFilter[i] == '|')
                szFilter[i] = '\0';
        ofn.lpstrFilter = szFilter;
        ofn.nFilterIndex = 1;
        ofn.lpstrFileTitle = NULL;
        ofn.nMaxFileTitle = 0;
        ofn.lpstrInitialDir = NULL;
        ofn.lpstrDefExt = lpszDefExt;
        if(bSave)
        {
            ofn.Flags = OFN_SHOWHELP | OFN_OVERWRITEPROMPT | OFN_EXPLORER;
            return GetSaveFileName(&ofn);
        }
        else
        {
            ofn.Flags = OFN_SHOWHELP | OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_EXPLORER;
            return GetOpenFileName(&ofn);
        }
    }

            */

    fn create_main_window() -> apiw::Result<()> {
        use apiw;
        use apiw::windows_subsystem::window::ForeignWindow;
        use apiw::windows_subsystem::window::WindowBuilder;
        use apiw::windows_subsystem::window::WindowClassBuilder;
        use apiw::windows_subsystem::window::WindowProcRequest;
        use apiw::windows_subsystem::window::WindowStyles;

        fn main_window_handler(mut request: WindowProcRequest) {
            use apiw::windows_subsystem::window::CommandEventArgs;
            use apiw::windows_subsystem::window::MouseEventArgs;

            request
                .route_create(|window: &ForeignWindow, _| -> apiw::Result<bool> {
                    THE_GAME.with(|game| {
                        let mut game = game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
                        let game = &mut *game;
                        game.mvc.redirect_output_target(Some(window.clone()));
                        window.invalidate()?;
                        Ok(())
                    })?;
                    Ok(true)
                })
                .route_paint(|window: &ForeignWindow| -> apiw::Result<()> {
                    let mut paint_dc = window.do_paint()?;

                    THE_GAME.with(|game| {
                        let game = game.try_borrow().or_else(|_| apiw::internal_error())?;
                        game.mvc.sync_output_with_parameter(&mut paint_dc);
                        Ok(())
                    })?;

                    Ok(())
                })
                .route_mouse(
                    |_window: &ForeignWindow, mouse_args: MouseEventArgs| -> apiw::Result<bool> {
                        THE_GAME.with(|game| {
                            use apiw::windows_subsystem::window::MouseEventArgType;

                            let mut game =
                                game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
                            let game = &mut *game;

                            let mut target = None;
                            if let Some(point) = mouse_args.cursor_coordinate() {
                                target = Some(game.mvc.view().hit_test(point));
                            }

                            use crate::controller::KeyKind;
                            use concerto::ActionInput;

                            if let Some(target) = target.as_ref() {
                                game.mvc.process_input(ControllerInput::ActionInput(
                                    ActionInput::CursorCoordinate(target.clone()),
                                ));
                            }

                            if let Some(key_input) = match mouse_args.kind() {
                                Some(MouseEventArgType::LeftButtonDown) => {
                                    Some(ActionInput::KeyDown(KeyKind::LButton))
                                }
                                Some(MouseEventArgType::LeftButtonUp) => {
                                    Some(ActionInput::KeyUp(KeyKind::LButton))
                                }
                                Some(MouseEventArgType::RightButtonDown) => {
                                    Some(ActionInput::KeyDown(KeyKind::RButton))
                                }
                                Some(MouseEventArgType::RightButtonUp) => {
                                    Some(ActionInput::KeyUp(KeyKind::RButton))
                                }
                                _ => None,
                            } {
                                game.mvc
                                    .process_input(ControllerInput::ActionInput(key_input));
                                if let Some(target) = target.as_ref() {
                                    game.mvc.process_input(ControllerInput::ActionInput(
                                        ActionInput::CursorCoordinate(target.clone()),
                                    ));
                                }
                            }

                            Ok(())
                        })?;
                        Ok(true)
                    },
                )
                .route_command(
                    |window: &ForeignWindow, args: CommandEventArgs| -> apiw::Result<()> {
                        use crate::model::ModelCommand;
                        use crate::view_assets::resources;
                        match args.id() as isize {
                            resources::IDM_FILE_NEW => {
                                THE_GAME.with(|game| {
                                    let mut game =
                                        game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
                                    let game = &mut *game;
                                    game.mvc.process_input(ControllerInput::ModelCommand(
                                        ModelCommand::NewGame,
                                    ));
                                    Ok(())
                                })?;
                            }
                            resources::IDM_FILE_GAME_EASY
                            | resources::IDM_FILE_GAME_MEDIUM
                            | resources::IDM_FILE_GAME_HARD => {
                                THE_GAME.with(|game| {
                                    let mut game =
                                        game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
                                    let game = &mut *game;
                                    let boardsetting = match args.id() as isize {
                                        resources::IDM_FILE_GAME_EASY => {
                                            model_config::BoardSetting::EASY
                                        }
                                        resources::IDM_FILE_GAME_MEDIUM => {
                                            model_config::BoardSetting::NORMAL
                                        }
                                        resources::IDM_FILE_GAME_HARD => {
                                            model_config::BoardSetting::HARD
                                        }
                                        _ => unreachable!(),
                                    };
                                    game.mvc.process_input(ControllerInput::ModelCommand(
                                        ModelCommand::NewGameWithBoard(boardsetting),
                                    ));
                                    Ok(())
                                })?;
                            }
                            resources::IDM_FILE_MARK => {
                                THE_GAME.with(|game| {
                                    let mut game =
                                        game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
                                    let game = &mut *game;

                                    game.mvc.process_input(ControllerInput::ModelCommand(
                                        ModelCommand::ToggleAllowMarks,
                                    ));
                                    Ok(())
                                })?;
                            }
                            resources::IDM_ADVANCED_ZOOM_1x
                            | resources::IDM_ADVANCED_ZOOM_2x
                            | resources::IDM_ADVANCED_ZOOM_3x => {
                                THE_GAME.with(|game| {
                                    let mut game =
                                        game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
                                    let game = &mut *game;

                                    game.mvc.process_input(ControllerInput::ModelCommand(
                                        ModelCommand::UpdateZoomRatio(match args.id() as isize {
                                            resources::IDM_ADVANCED_ZOOM_1x => {
                                                model_config::ZoomRatio::Zoom1x
                                            }
                                            resources::IDM_ADVANCED_ZOOM_2x => {
                                                model_config::ZoomRatio::Zoom2x
                                            }
                                            resources::IDM_ADVANCED_ZOOM_3x => {
                                                model_config::ZoomRatio::Zoom3x
                                            }
                                            _ => unreachable!(),
                                        }),
                                    ));
                                    Ok(())
                                })?;
                            }
                            resources::IDM_FILE_EXIT => {
                                window.destroy()?;
                            }
                            resources::IDM_ADVANCED_LOADMAP => {
                                if let Some(path) = Ui::call_open_file_dialog(window, 0, "cmm") {
                                    THE_GAME.with(|game| {
                                        let mut game = game
                                            .try_borrow_mut()
                                            .or_else(|_| apiw::internal_error())?;
                                        let game = &mut *game;

                                        game.mvc.process_input(ControllerInput::ModelCommand(
                                            ModelCommand::LoadMap(path),
                                        ));
                                        Ok(())
                                    })?;
                                }
                            }
                            resources::IDM_ADVANCED_SAVEMAP => {
                                if let Some(path) = Ui::call_save_file_dialog(window, 0, "cmm") {
                                    THE_GAME.with(|game| {
                                        let mut game = game
                                            .try_borrow_mut()
                                            .or_else(|_| apiw::internal_error())?;
                                        let game = &mut *game;

                                        game.mvc.process_input(ControllerInput::ModelCommand(
                                            ModelCommand::SaveMap(path),
                                        ));
                                        Ok(())
                                    })?;
                                }
                            }
                            resources::IDM_ADVANCED_RESTART => {
                                THE_GAME.with(|game| {
                                    let mut game =
                                        game.try_borrow_mut().or_else(|_| apiw::internal_error())?;
                                    let game = &mut *game;

                                    game.mvc.process_input(ControllerInput::ModelCommand(
                                        ModelCommand::RestartGame,
                                    ));
                                    Ok(())
                                })?;
                            }
                            resources::IDM_HELP_ABOUT => {
                                use apiw::windows_subsystem::dialog::DialogBuilder;

                                DialogBuilder::new_from_resource_id(resources::IDD_ABOUTBOX as _)
                                    .invoke()?;
                            }
                            _ => {}
                        }
                        Ok(())
                    },
                )
                .route_destroy(|_window: &ForeignWindow| -> apiw::Result<()> {
                    use apiw::windows_subsystem::message::ForeignMessageLoop;

                    ForeignMessageLoop::request_quit();
                    
                    Ok(())
                });
        }

        use apiw::windows_subsystem::window::TimerProcRequest;

        fn main_window_timer_handler(request: TimerProcRequest) {
            if let Some(window) = request.window() {
                let _ = window.invalidate();
            }
        }

        let window_class = WindowClassBuilder::new("CharlesMineWnd")
            .background_brush_from_syscolor(apiw::windows_subsystem::window::SysColor::BUTTON_FACE)
            .cursor_from_syscursor(apiw::windows_subsystem::window::SysCursor::ARROW)
            .icon_from_resource_id(view_assets::resources::IDI_CHARLESMINE as _)
            .menu_from_resource_id(view_assets::resources::IDC_CHARLESMINE as _)
            .window_proc(window_proc!(main_window_handler))
            .create_managed()?;
        /*
            wcex.style			= CS_HREDRAW | CS_VREDRAW;
        */

        let window = WindowBuilder::new(&window_class)
            .name("CharlesMine")
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
            .set_timer_with_id(
                NonZeroUsize::new(1).unwrap(),
                Duration::from_millis(100),
                timer_proc!(main_window_timer_handler),
            )?
            .show(apiw::shared::exe_cmd_show())?
            .update()?;

        Ok(())
    }

    pub(crate) fn run_event_loop() -> apiw::Result<()> {
        use apiw::windows_subsystem::message::ForeignMessageLoop;
        let mut msgloop = ForeignMessageLoop::for_current_thread();

        while let Some(msg) = msgloop.poll_wait()?.not_quit() {
            let _ = msg.dispatch();
        }

        Ok(())
    }
}
