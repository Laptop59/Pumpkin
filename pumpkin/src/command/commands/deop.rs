use crate::command::argument_builder::{ArgumentBuilder, argument, command};
use crate::command::argument_types::game_profile::GameProfileArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;
use crate::data::SaveJSONConfiguration;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

pub const NOT_OP_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DEOP_FAILED,
    translation::bedrock::COMMANDS_DEOP_FAILED,
);

const DESCRIPTION: &str = "Revokes operator status from a player.";
const PERMISSION: &str = "minecraft:command.deop";
const ARG_TARGETS: &str = "targets";

struct DeOpCommandExecutor;

impl CommandExecutor for DeOpCommandExecutor {
    fn execute<'a>(&'a self, context: &'a CommandContext) -> CommandExecutorResult<'a> {
        Box::pin(async move {
            let server = context.server();
            let profiles = GameProfileArgumentType::get(context, ARG_TARGETS).await?;
            let mut config = server.data.operator_config.write().await;

            let mut succeeded_deops: usize = 0;
            for profile in profiles {
                if let Some(op_index) = config.ops.iter().position(|o| o.uuid == profile.id) {
                    config.ops.remove(op_index);
                    succeeded_deops += 1;

                    if let Some(player) = server.get_player_by_uuid(profile.id) {
                        let command_dispatcher = server.command_dispatcher.read().await;
                        player
                            .set_permission_lvl(
                                server,
                                pumpkin_util::PermissionLvl::Zero,
                                &command_dispatcher,
                            )
                            .await;
                    }

                    let msg = TextComponent::translate_cross(
                        translation::java::COMMANDS_DEOP_SUCCESS,
                        translation::bedrock::COMMANDS_DEOP_SUCCESS,
                        [TextComponent::text(profile.name.clone())],
                    );
                    context.source.send_feedback(msg, true).await;
                }
            }

            if succeeded_deops == 0 {
                Err(NOT_OP_ERROR_TYPE.create_without_context())
            } else {
                config.save();
                Ok(succeeded_deops as i32)
            }
        })
    }
}

struct DeOpSuggestionProvider;

impl SuggestionProvider for DeOpSuggestionProvider {
    fn suggest<'a>(
        &'a self,
        context: &'a CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult<'a> {
        Box::pin(async move {
            // Suggest every opped player.
            let ops = context.server().data.operator_config.read().await;
            for op in &ops.ops {
                builder = builder.suggest(op.name.clone());
            }
            builder.build()
        })
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &mut PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Three),
    ));

    dispatcher.register(
        command("deop", DESCRIPTION).requires(PERMISSION).then(
            argument(ARG_TARGETS, GameProfileArgumentType)
                .suggests(DeOpSuggestionProvider)
                .executes(DeOpCommandExecutor),
        ),
    );
}