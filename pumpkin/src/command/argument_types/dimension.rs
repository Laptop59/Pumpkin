use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::{
    argument_types::{
        FromStringReader,
        argument_type::{ArgumentType, JavaClientArgumentType},
    },
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
};
use pumpkin_data::dimension::Dimension;
use pumpkin_data::translation;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

pub const INVALID_VALUE_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_DIMENSION_INVALID,
    translation::java::ARGUMENT_DIMENSION_INVALID,
);

/// An argument type that parses a dimension from its [`Identifier`]..
pub struct DimensionArgumentType;

impl ArgumentType for DimensionArgumentType {
    type Item = Identifier;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        Identifier::from_reader(reader)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Dimension
    }

    fn examples(&self) -> Vec<String> {
        examples!("minecraft:overworld", "the_nether")
    }
}

impl DimensionArgumentType {
    /// Tries to get the appropriate [`Dimension`] from the given parsed argument's name.
    pub fn get(
        context: &CommandContext,
        name: &str,
    ) -> Result<&'static Dimension, CommandSyntaxError> {
        let identifier: &Identifier = context.get_argument(name)?;
        Dimension::from_name(&identifier.to_string()).ok_or_else(|| {
            INVALID_VALUE_ERROR_TYPE
                .create_without_context(TextComponent::text(identifier.to_string()))
        })
    }
}
