use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::translation::COMMANDS_EXPERIENCE_SET_POINTS_INVALID;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::experience;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::entity_selector::EntitySelector;
use crate::command::context::command_context::CommandContext;
use crate::command::context::command_source::CommandSource;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::entity::player::Player;

const DESCRIPTION: &str = "Add, set or query player experience.";
const PERMISSION: &str = "minecraft:command.experience";
const ARG_TARGET: &str = "target";
const ARG_AMOUNT: &str = "amount";

const SET_POINTS_INVALID_ERROR_TYPE: CommandErrorType<0> =
    CommandErrorType::new(COMMANDS_EXPERIENCE_SET_POINTS_INVALID);

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Add,
    Set,
    Query,
}

#[derive(Clone, Copy, PartialEq)]
enum ExpType {
    Points,
    Levels,
}

struct Executor {
    mode: Mode,
    exp_type: Option<ExpType>,
}

impl Executor {
    async fn handle_query(
        &self,
        source: &CommandSource,
        target: &Player,
        exp_type: ExpType,
    ) -> i32 {
        match exp_type {
            ExpType::Levels => {
                let level = target.experience_level.load(Ordering::Relaxed);
                source
                    .send_feedback(
                        TextComponent::translate(
                            "commands.experience.query.levels",
                            [
                                target.get_display_name().await,
                                TextComponent::text(level.to_string()),
                            ],
                        ),
                        false,
                    )
                    .await;
                level
            }
            ExpType::Points => {
                let points = target.experience_points.load(Ordering::Relaxed);
                source
                    .send_feedback(
                        TextComponent::translate(
                            "commands.experience.query.points",
                            [
                                target.get_display_name().await,
                                TextComponent::text(points.to_string()),
                            ],
                        ),
                        false,
                    )
                    .await;
                points
            }
        }
    }

    async fn get_success_message(
        mode: Mode,
        exp_type: ExpType,
        amount: i32,
        targets_len: usize,
        first_player: &Arc<Player>,
    ) -> TextComponent {
        match (mode, exp_type) {
            (Mode::Add, ExpType::Points) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.add.points.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.add.points.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            first_player.get_display_name().await,
                        ],
                    )
                }
            }
            (Mode::Add, ExpType::Levels) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.add.levels.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.add.levels.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            first_player.get_display_name().await,
                        ],
                    )
                }
            }
            (Mode::Set, ExpType::Points) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.set.points.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.set.points.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            first_player.get_display_name().await,
                        ],
                    )
                }
            }
            (Mode::Set, ExpType::Levels) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.set.levels.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.set.levels.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            first_player.get_display_name().await,
                        ],
                    )
                }
            }
            (Mode::Query, _) => unreachable!("Query mode doesn't use success messages"),
        }
    }

    /// Returns `true` if successful. Otherwise, there was a problem setting the points of a player.
    async fn handle_modify(
        &self,
        target: &Arc<Player>,
        amount: i32,
        exp_type: ExpType,
        mode: Mode,
    ) -> bool {
        match exp_type {
            ExpType::Levels => {
                if mode == Mode::Add {
                    target.add_experience_levels(amount).await;
                } else {
                    target.set_experience_level(amount, true).await;
                }
            }
            ExpType::Points => {
                if mode == Mode::Add {
                    target.add_experience_points(amount).await;
                } else {
                    let current_level = target.experience_level.load(Ordering::Relaxed);
                    let current_max_points = experience::points_in_level(current_level);

                    if amount > current_max_points {
                        return false;
                    }

                    target.set_experience_points(amount).await;
                }
            }
        }
        true
    }
}

impl CommandExecutor for Executor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let selector: &EntitySelector = context.get_argument(ARG_TARGET)?;
            let targets = selector.find_players(&context.source).await?;

            match self.mode {
                Mode::Query => Ok(self
                    .handle_query(&context.source, &targets[0], self.exp_type.unwrap())
                    .await),
                Mode::Add | Mode::Set => {
                    let amount: i32 = *context.get_argument(ARG_AMOUNT)?;

                    let mut successes: i32 = 0;
                    for target in &targets {
                        let succeeded = self
                            .handle_modify(target, amount, self.exp_type.unwrap(), self.mode)
                            .await;

                        if succeeded {
                            successes += 1;
                        }
                    }

                    if successes == 0 {
                        Err(SET_POINTS_INVALID_ERROR_TYPE.create_without_context())
                    } else {
                        // This should not panic as we already check the number of successes to not be equal to `0`.
                        let msg = Self::get_success_message(
                            self.mode,
                            self.exp_type.unwrap(),
                            amount,
                            targets.len(),
                            targets
                                .first()
                                .expect("expected at least one player in targets"),
                        )
                        .await;
                        context.source.send_feedback(msg, true).await;

                        Ok(successes)
                    }
                }
            }
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register_with_aliases(
        command("experience", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("add").then(
                    argument(ARG_TARGET, EntityArgumentType::Players).then(
                        argument(ARG_AMOUNT, IntegerArgumentType::any())
                            .executes(Executor {
                                mode: Mode::Add,
                                exp_type: Some(ExpType::Points),
                            })
                            .then(literal("levels").executes(Executor {
                                mode: Mode::Add,
                                exp_type: Some(ExpType::Levels),
                            }))
                            .then(literal("points").executes(Executor {
                                mode: Mode::Add,
                                exp_type: Some(ExpType::Points),
                            })),
                    ),
                ),
            )
            .then(
                literal("set").then(
                    argument(ARG_TARGET, EntityArgumentType::Players).then(
                        argument(ARG_AMOUNT, IntegerArgumentType::with_min(0))
                            .executes(Executor {
                                mode: Mode::Set,
                                exp_type: Some(ExpType::Points),
                            })
                            .then(literal("levels").executes(Executor {
                                mode: Mode::Set,
                                exp_type: Some(ExpType::Levels),
                            }))
                            .then(literal("points").executes(Executor {
                                mode: Mode::Set,
                                exp_type: Some(ExpType::Points),
                            })),
                    ),
                ),
            )
            .then(
                literal("query").then(
                    argument(ARG_TARGET, EntityArgumentType::Player)
                        .then(literal("levels").executes(Executor {
                            mode: Mode::Query,
                            exp_type: Some(ExpType::Levels),
                        }))
                        .then(literal("points").executes(Executor {
                            mode: Mode::Query,
                            exp_type: Some(ExpType::Points),
                        })),
                ),
            ),
        &["xp"],
    );
}
