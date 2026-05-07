use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::translation::java::ARGUMENT_COLOR_INVALID;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use std::pin::Pin;

pub const INVALID_VALUE_ERROR_TYPE: CommandErrorType<1> =
    CommandErrorType::new(ARGUMENT_COLOR_INVALID, ARGUMENT_COLOR_INVALID);

/// A color that can either be `Reset` or a [`NamedColor`].
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum NonRgbColor {
    #[default]
    Reset,
    Named(NamedColor),
}

pub struct ColorArgumentType;

impl ArgumentType for ColorArgumentType {
    type Item = NonRgbColor;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let id = reader.read_unquoted_string();
        // As Minecraft removes non-alphabetic characters before parsing, even
        // words like 're_set', 'blue...', etc. will parse correctly.
        // TODO: Revisit this when 26.2 gets released
        let checked_id = id.to_ascii_lowercase().replace(
            [
                '.', '_', '+', '-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            ],
            "",
        );

        match checked_id.as_str() {
            "black" => Ok(NonRgbColor::Named(NamedColor::Black)),
            "darkblue" => Ok(NonRgbColor::Named(NamedColor::DarkBlue)),
            "darkgreen" => Ok(NonRgbColor::Named(NamedColor::DarkGreen)),
            "darkaqua" => Ok(NonRgbColor::Named(NamedColor::DarkAqua)),
            "darkred" => Ok(NonRgbColor::Named(NamedColor::DarkRed)),
            "darkpurple" => Ok(NonRgbColor::Named(NamedColor::DarkPurple)),
            "gold" => Ok(NonRgbColor::Named(NamedColor::Gold)),
            "gray" => Ok(NonRgbColor::Named(NamedColor::Gray)),
            "darkgray" => Ok(NonRgbColor::Named(NamedColor::DarkGray)),
            "blue" => Ok(NonRgbColor::Named(NamedColor::Blue)),
            "green" => Ok(NonRgbColor::Named(NamedColor::Green)),
            "aqua" => Ok(NonRgbColor::Named(NamedColor::Aqua)),
            "red" => Ok(NonRgbColor::Named(NamedColor::Red)),
            "lightpurple" => Ok(NonRgbColor::Named(NamedColor::LightPurple)),
            "yellow" => Ok(NonRgbColor::Named(NamedColor::Yellow)),
            "white" => Ok(NonRgbColor::Named(NamedColor::White)),

            "reset" => Ok(NonRgbColor::Reset),

            _ => {
                Err(INVALID_VALUE_ERROR_TYPE
                    .create_without_context(TextComponent::text(checked_id)))
            }
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Color
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move {
            builder
                .filter_and_suggest(&[
                    "black",
                    "dark_blue",
                    "dark_green",
                    "dark_aqua",
                    "dark_red",
                    "dark_purple",
                    "gold",
                    "gray",
                    "dark_gray",
                    "blue",
                    "green",
                    "aqua",
                    "red",
                    "light_purple",
                    "yellow",
                    "white",
                    "reset",
                ])
                .build()
        })
    }

    fn examples(&self) -> Vec<String> {
        examples!("red", "blue", "yellow")
    }
}

impl_copy_get!(ColorArgumentType, NonRgbColor);
