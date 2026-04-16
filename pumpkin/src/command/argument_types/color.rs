use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use std::pin::Pin;

pub const INVALID_VALUE_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(translation::ARGUMENT_COLOR_INVALID);

/// Denotes the result of parsing a color from [`ColorArgumentType`].
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ColorResult {
    /// The 'reset' color, which is usually the default.
    #[default]
    Reset,

    /// An actual color, one of the 16 named colors.
    Named(NamedColor),
}

/// An argument type that parses either a [`NamedColor`], or a reset color.
///
/// When the 'reset' color is parsed, the argument type returns [`None`], while for
/// any parsed [`NamedColor`], it will get wrapped in a [`Some`] before being returned.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ColorArgumentType;

impl ArgumentType for ColorArgumentType {
    type Item = ColorResult;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let id = reader.read_unquoted_string()?;
        Ok(match id.as_str() {
            "reset" => ColorResult::Reset,
            id_str => ColorResult::Named(NamedColor::try_from(id_str).map_err(|()| {
                INVALID_VALUE_ERROR_TYPE.create_without_context(TextComponent::text(id))
            })?),
        })
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        mut suggestions_builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send>> {
        for color in NamedColor::VALUES {
            suggestions_builder = suggestions_builder.suggest(color.name());
        }
        Box::pin(async move { suggestions_builder.build() })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Color
    }

    fn examples(&self) -> Vec<String> {
        examples!("yellow", "red", "dark_purple")
    }
}

impl ColorArgumentType {
    /// Returns a [`CommandContext`]'s parsed color argument.
    pub fn get(context: &CommandContext, name: &str) -> Result<ColorResult, CommandSyntaxError> {
        Ok(*context.get_argument(name)?)
    }
}

#[cfg(test)]
mod test {
    use crate::command::argument_types::argument_type::ArgumentType;
    use crate::command::argument_types::color::{
        ColorArgumentType, ColorResult, INVALID_VALUE_ERROR_TYPE,
    };
    use crate::command::string_reader::StringReader;
    use pumpkin_util::text::color::NamedColor;

    #[test]
    fn test() {
        let mut reader = StringReader::new("reset");
        assert_parse_ok_reset!(reader, ColorArgumentType, ColorResult::Reset);

        let mut reader = StringReader::new("red");
        assert_parse_ok_reset!(
            reader,
            ColorArgumentType,
            ColorResult::Named(NamedColor::Red)
        );
        let mut reader = StringReader::new("dark_gray");
        assert_parse_ok_reset!(
            reader,
            ColorArgumentType,
            ColorResult::Named(NamedColor::DarkGray)
        );
        let mut reader = StringReader::new("yellow");
        assert_parse_ok_reset!(
            reader,
            ColorArgumentType,
            ColorResult::Named(NamedColor::Yellow)
        );
        let mut reader = StringReader::new("light_purple");
        assert_parse_ok_reset!(
            reader,
            ColorArgumentType,
            ColorResult::Named(NamedColor::LightPurple)
        );

        let mut reader = StringReader::new("bold");
        assert_parse_err_reset!(reader, ColorArgumentType, &INVALID_VALUE_ERROR_TYPE);
        let mut reader = StringReader::new("italic");
        assert_parse_err_reset!(reader, ColorArgumentType, &INVALID_VALUE_ERROR_TYPE);
    }
}
