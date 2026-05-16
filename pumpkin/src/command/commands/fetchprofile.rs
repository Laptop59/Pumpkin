use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::uuid::UuidArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Fetches a player's profile with some useful utilities.";
const PERMISSION: &str = "minecraft:command.fetchprofile";

const ARG_NAME: &str = "name";
const ARG_ID: &str = "id";
const ARG_ENTITY: &str = "entity";

struct FetchProfileNameExecutor;

impl CommandExecutor for FetchProfileNameExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move { Ok(0) })
    }
}

struct FetchProfileIdExecutor;

impl CommandExecutor for FetchProfileIdExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move { Ok(0) })
    }
}

struct FetchProfileEntityExecutor;

impl CommandExecutor for FetchProfileEntityExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move { Ok(0) })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("fetchprofile", DESCRIPTION)
            .requires(PERMISSION)
            .then(
                literal("name").then(
                    argument(ARG_NAME, StringArgumentType::GreedyPhrase)
                        .executes(FetchProfileNameExecutor),
                ),
            )
            .then(
                literal("id")
                    .then(argument(ARG_ID, UuidArgumentType).executes(FetchProfileIdExecutor)),
            )
            .then(
                literal("entity").then(
                    argument(ARG_ENTITY, EntityArgumentType::Entity)
                        .executes(FetchProfileEntityExecutor),
                ),
            ),
    );
}
