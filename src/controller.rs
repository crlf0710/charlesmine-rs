use apiw::graphics_subsystem::Point;
use apiw::graphics_subsystem::Rect;
use apiw::graphics_subsystem::Size;
use apiw::windows_subsystem::window::MouseEventArgs;
use concerto::ActionContextBuilder;
use concerto::{self, ActionContext};
use view::{self, View, GameTarget};
use model::{self, Model, ModelCommand};
use model_config::{self, Config};

pub struct Controller {
    action_contexts: Vec<ActionContext<Controller>>,
}

#[derive(Debug)]
pub enum ControllerInput {
    Initialize,
    ActionInput(concerto::ActionInput<Controller>),
    ModelCommand(ModelCommand),
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
    type Command = ModelCommand;
}

impl Controller {
    pub fn new(_model: &Model) -> Self {
        let action_context = ActionContextBuilder::new()
            .add_recipe(|recipe_builder| {
                recipe_builder
                    .keep_cursor_coordinate_input(GameTarget::GameButton)
                    .add_key_down_input(KeyKind::LButton)
                    .issue_effect(
                        ModelCommand::EffectNewGameButtonDown,
                        ModelCommand::EffectNewGameButtonUp,
                    )
                    .add_key_up_input(KeyKind::LButton)
                    .issue_command(ModelCommand::NewGame)
                    .build()
            })
            .add_recipe(|recipe_builder| {
                recipe_builder
                    .keep_key_not_pressed(KeyKind::RButton)
                    .add_cursor_coordinate_filtered_input(|t| match t {
                        GameTarget::FieldBlock { .. } => true,
                        _ => false,
                    })
                    .add_key_down_input(KeyKind::LButton)
                    .enable_starting_nest_recipe(|_, nest_recipe_builder| {
                        nest_recipe_builder
                            .keep_cursor_coordinate_filtered_input(|t| match t {
                                GameTarget::FieldBlock { .. } => true,
                                _ => false,
                            })
                            .check_key_pressed(KeyKind::LButton)
                            .issue_effect_with(|x| {
                                let target = x.cursor_coordinate();
                                let (y, x) = match target {
                                    Some(GameTarget::FieldBlock { y, x }) => (*y, *x),
                                    _ => panic!("Unexpected error"),
                                };
                                (
                                    ModelCommand::EffectPushBlock { y, x },
                                    ModelCommand::EffectPopBlock { y, x },
                                )
                            })
                            .add_key_up_input(KeyKind::LButton)
                            .issue_command_with(|x| {
                                let target = x.cursor_coordinate();
                                let (y, x) = match target {
                                    Some(GameTarget::FieldBlock { y, x }) => (*y, *x),
                                    _ => panic!("Unexpected error"),
                                };
                                ModelCommand::OpenBlock(y as _, x as _)
                            })
                            .build()
                    })
                    .add_key_up_input(KeyKind::LButton)
                    .build()
            })
            .add_recipe(|recipe_builder| {
                recipe_builder
                    .keep_key_not_pressed(KeyKind::LButton)
                    .keep_cursor_coordinate_filtered_input(|t| match t {
                        GameTarget::FieldBlock { .. } => true,
                        _ => false,
                    })
                    .add_key_down_input(KeyKind::RButton)
                    .issue_command_with(|x| {
                        let target = x.cursor_coordinate();
                        let (y, x) = match target {
                            Some(GameTarget::FieldBlock { y, x }) => (*y, *x),
                            _ => panic!("Unexpected error"),
                        };
                        ModelCommand::RotateBlockState(y as _, x as _)
                    })
                    .add_key_up_input(KeyKind::RButton)
                    .build()
            })
            .add_recipe(|recipe_builder| {
                recipe_builder
                    .add_cursor_coordinate_filtered_input(|t| match t {
                        GameTarget::FieldBlock { .. } => true,
                        _ => false,
                    })
                    .add_unordered_multiple_key_down_input(&[KeyKind::LButton, KeyKind::RButton])
                    .enable_starting_nest_recipe(|_, nest_recipe_builder| {
                        nest_recipe_builder
                            .keep_cursor_coordinate_filtered_input(|t| match t {
                                GameTarget::FieldBlock { .. } => true,
                                _ => false,
                            })
                            .issue_effect_with(|x| {
                                let target = x.cursor_coordinate();
                                let (y, x) = match target {
                                    Some(GameTarget::FieldBlock { y, x }) => (*y, *x),
                                    _ => panic!("Unexpected error"),
                                };
                                (
                                    ModelCommand::EffectBlastDownBlock { y, x },
                                    ModelCommand::EffectBlastUpBlock { y, x },
                                )
                            })
                            .add_one_of_multiple_key_up_input(&[KeyKind::LButton, KeyKind::RButton])
                            .issue_command_with(|x| {
                                let target = x.cursor_coordinate();
                                let (y, x) = match target {
                                    Some(GameTarget::FieldBlock { y, x }) => (*y, *x),
                                    _ => panic!("Unexpected error"),
                                };
                                ModelCommand::BlastBlock(y as _, x as _)
                            })
                            .build()
                    })
                    .add_unordered_multiple_key_up_input(&[KeyKind::LButton, KeyKind::RButton])
                    .build()
            })
            .build();
        let capture_tracker = ActionContextBuilder::new()
            .add_recipe(|recipe_builder| {
                recipe_builder
                    .add_key_down_input(KeyKind::LButton)
                    .issue_effect(
                        ModelCommand::EffectCapture,
                        ModelCommand::EffectUnCapture,
                    )
                    .add_key_up_input(KeyKind::LButton)
                    .build()
            })
            .build();

        Controller {
            action_contexts: vec![action_context, capture_tracker],
        }
    }
}

type ControllerToken<'a> = ::domino::mvc::ControllerToken<'a, model::Model, view::View, Controller>;

impl ::domino::mvc::Controller<model::Model, view::View> for Controller {
    type Command = ControllerInput;
    type Notification = ModelCommand;

    fn process_command(mut token: ControllerToken, command: ControllerInput) {
        match command {
            ControllerInput::Initialize => {
                token.manipulate_model_next(ModelCommand::Initialize);
            }
            ControllerInput::ActionInput(input) => {
                let new_commands = {
                    let mut new_commands = Vec::new();
                    let controller = token.controller_mut();
                    for action_context in controller.action_contexts.iter_mut() {
                        action_context.process_input(&input);
                        if let Some(commands) = action_context.collect_commands() {
                            new_commands.extend(commands);
                        }
                    }
                    new_commands
                };

                for model_command in new_commands {
                    token.manipulate_model_next(model_command);
                }
            },
            ControllerInput::ModelCommand(model_command) => {
                token.manipulate_model_next(model_command);
            }
        }
    }
}
