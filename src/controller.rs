use layout::Layout;
use model::Board;
use apiw::windows_subsystem::window::MouseEventArgs;
use apiw::graphics_subsystem::Point;
use apiw::graphics_subsystem::Size;
use layout::GameTarget;
use apiw::graphics_subsystem::Rect;
use concerto::{self, ActionContext};
use concerto::ActionContextBuilder;

pub struct Controller {
    action_context: ActionContext<Controller>,
    commands: Vec<ControllerCommand>,
}

#[derive(Clone, Debug)]
pub enum ControllerCommand {
    NewGame,
    OpenBlock(usize, usize),
    BlastBlock(usize, usize),
    RotateBlockState(usize, usize),

    EffectNewGameButtonDown,
    EffectNewGameButtonUp,
    EffectPushBlock{x: usize, y: usize},
    EffectPopBlock{x: usize, y: usize},
    EffectBlastDownBlock{x: usize, y: usize},
    EffectBlastUpBlock{x: usize, y: usize},
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum KeyKind {
    LButton,
    RButton,
}

impl concerto::ActionConfiguration for Controller {
    type Target = GameTarget;
    type KeyKind = KeyKind;
    type CursorPos = Point;
    type Command = ControllerCommand;
}


impl Controller {
    pub fn new() -> Self {
        let action_context =
            ActionContextBuilder::new()
                .add_recipe(|recipe_builder| {
                    recipe_builder
                        .keep_cursor_coordinate_input(GameTarget::GameButton)
                        .add_key_down_input(KeyKind::LButton)
                        .issue_effect(ControllerCommand::EffectNewGameButtonDown, ControllerCommand::EffectNewGameButtonUp)
                        .add_key_up_input(KeyKind::LButton)
                        .issue_command(ControllerCommand::NewGame)
                        .build()
                })
                .add_recipe(|recipe_builder| {
                    recipe_builder
                        .keep_key_not_pressed(KeyKind::RButton)
                        .keep_cursor_coordinate_filtered_input(|t| {
                            match t {
                                GameTarget::FieldBlock {..} => true,
                                _ => false
                            }
                        })
                        .check_key_pressed(KeyKind::LButton)
                        .issue_effect_with(|x| {
                            let target = x.cursor_coordinate();
                            let (y, x) = match target {
                                Some(GameTarget::FieldBlock{y, x}) => (*y, *x),
                                _ => panic!("Unexpected error"),
                            };
                            (ControllerCommand::EffectPushBlock{y, x}, ControllerCommand::EffectPopBlock{y, x})
                        })
                        .add_key_up_input(KeyKind::LButton)
                        .issue_command_with(|x| {
                            let target = x.cursor_coordinate();
                            let (y, x) = match target {
                                Some(GameTarget::FieldBlock{y, x}) => (*y, *x),
                                _ => panic!("Unexpected error"),
                            };
                            ControllerCommand::OpenBlock(y as _, x as _)
                        })
                        .build()
                })
                .add_recipe(|recipe_builder| {
                    recipe_builder
                        .keep_key_not_pressed(KeyKind::LButton)
                        .keep_cursor_coordinate_filtered_input(|t| {
                            match t {
                                GameTarget::FieldBlock {..} => true,
                                _ => false
                            }
                        })
                        .add_key_down_input(KeyKind::RButton)
                        .issue_command_with(|x| {
                            let target = x.cursor_coordinate();
                            let (y, x) = match target {
                                Some(GameTarget::FieldBlock{y, x}) => (*y, *x),
                                _ => panic!("Unexpected error"),
                            };
                            ControllerCommand::RotateBlockState(y as _, x as _)
                        })
                        .add_key_up_input(KeyKind::RButton)
                        .build()
                })
                .add_recipe(|recipe_builder| {
                    recipe_builder
                        .keep_cursor_coordinate_filtered_input(|t| {
                            match t {
                                GameTarget::FieldBlock {..} => true,
                                _ => false
                            }
                        })
                        .add_sequential_multiple_key_down_input(&[KeyKind::LButton, KeyKind::RButton])
                        .issue_effect_with(|x| {
                            let target = x.cursor_coordinate();
                            let (y, x) = match target {
                                Some(GameTarget::FieldBlock{y, x}) => (*y, *x),
                                _ => panic!("Unexpected error"),
                            };
                            (ControllerCommand::EffectBlastDownBlock{y, x}, ControllerCommand::EffectBlastUpBlock{y, x})
                        })
                        .add_unordered_multiple_key_up_input(&[KeyKind::LButton, KeyKind::RButton])
                        .issue_command_with(|x| {
                            let target = x.cursor_coordinate();
                            let (y, x) = match target {
                                Some(GameTarget::FieldBlock{y, x}) => (*y, *x),
                                _ => panic!("Unexpected error"),
                            };
                            ControllerCommand::BlastBlock(y as _, x as _)
                        })
                        .build()
                })
                .build();
        Controller {
            action_context,
            commands: Vec::new(),
        }
    }

    pub fn send_mouse_input(&mut self, mouse_args: MouseEventArgs, layout: &Layout, board: &Board) -> bool {
        use concerto::ActionInput;
        use apiw::windows_subsystem::window::MouseEventArgType;

        let mut target = None;
        if let Some(point) = mouse_args.cursor_coordinate() {
            target = Some(layout.hit_test(point));
        }

        if let Some(target) = target.as_ref() {
            self.action_context.process_input(&ActionInput::CursorCoordinate(target.clone()));
        }

        if let Some(key_input) = match mouse_args.kind() {
            Some(MouseEventArgType::LeftButtonDown) => Some(ActionInput::KeyDown(KeyKind::LButton)),
            Some(MouseEventArgType::LeftButtonUp) => Some(ActionInput::KeyUp(KeyKind::LButton)),
            Some(MouseEventArgType::RightButtonDown) => Some(ActionInput::KeyDown(KeyKind::RButton)),
            Some(MouseEventArgType::RightButtonUp) => Some(ActionInput::KeyUp(KeyKind::RButton)),
            _ => None,
        } {
            self.action_context.process_input(&key_input);
            if let Some(target) = target.as_ref() {
                self.action_context.process_input(&ActionInput::CursorCoordinate(target.clone()));
            }
        }

        if let Some(new_commands) = self.action_context.collect_commands() {
            self.commands.extend(new_commands);
            return true;
        }

        false
    }

    pub fn flush_commands(&mut self, layout: &mut Layout, board: &mut Board) -> bool {
        let mut new_command = false;
        for command in self.commands.drain(..) {
            match command {
                ControllerCommand::NewGame => {
                    let (y, x) = board.size();
                    let c = board.goal_mark_count();
                    *board = Board::new_inner(y, x, c);
                    new_command = true;
                },
                ControllerCommand::OpenBlock(y, x) => {
                    board.open_block(y, x);
                    new_command = true;
                },
                ControllerCommand::BlastBlock(y, x) => {
                    board.blast_block(y, x);
                    new_command = true;
                },
                ControllerCommand::RotateBlockState(y, x) => {
                    board.rotate_block_state(y, x);
                    new_command = true;
                },
                ControllerCommand::EffectNewGameButtonDown => {
                    layout.set_button_pressed(true);
                    new_command = true;
                },
                ControllerCommand::EffectNewGameButtonUp => {
                    layout.set_button_pressed(false);
                    new_command = true;
                },
                ControllerCommand::EffectPushBlock {y, x}  => {
                    layout.set_block_pressed(y, x, false);
                    new_command = true;
                },
                ControllerCommand::EffectPopBlock {y, x}  => {
                    layout.unset_block_pressed(y, x, false);
                    new_command = true;
                },
                ControllerCommand::EffectBlastDownBlock {y, x}  => {
                    layout.set_block_pressed(y, x, true);
                    new_command = true;
                },
                ControllerCommand::EffectBlastUpBlock {y, x}  => {
                    layout.unset_block_pressed(y, x, true);
                    new_command = true;
                },
            }
        }
        new_command
    }
}

