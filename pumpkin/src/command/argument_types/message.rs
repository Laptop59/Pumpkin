use pumpkin_data::translation::java::ARGUMENT_MESSAGE_TOO_LONG;
use pumpkin_util::text::TextComponent;

use crate::command::{
    argument_types::{
        argument_type::{ArgumentType, JavaClientArgumentType},
        entity::ENTITY_SELECTOR_PERMISSION,
        entity_selector::{
            EntitySelector,
            parser::{
                EntitySelectorParser, MISSING_SELECTOR_TYPE_ERROR_TYPE,
                UNKNOWN_SELECTOR_TYPE_ERROR_TYPE,
            },
        },
    },
    context::{
        command_context::CommandContext, command_source::CommandSource, string_range::StringRange,
    },
    errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType},
    string_reader::StringReader,
};

pub const TOO_LONG_ERROR_TYPE: CommandErrorType<2> =
    CommandErrorType::new(ARGUMENT_MESSAGE_TOO_LONG, ARGUMENT_MESSAGE_TOO_LONG);

pub struct MessageArgumentType;

impl ArgumentType for MessageArgumentType {
    type Item = Message;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        Self::parse_with_allow_selectors(reader, true)
    }

    fn parse_with_source<'a>(
        &'a self,
        reader: &'a mut StringReader,
        source: &'a CommandSource,
    ) -> std::pin::Pin<Box<dyn Future<Output = Result<Self::Item, CommandSyntaxError>> + Send + 'a>>
    {
        Box::pin(async move {
            Self::parse_with_allow_selectors(
                reader,
                source.has_permission(ENTITY_SELECTOR_PERMISSION).await,
            )
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType<'_> {
        JavaClientArgumentType::Message
    }
}

impl MessageArgumentType {
    /// Gets the required message from the argument name for this type.
    pub async fn get(
        context: &CommandContext<'_>,
        name: &str,
    ) -> Result<TextComponent, CommandSyntaxError> {
        context
            .get_argument::<Message>(name)?
            .resolve_text_component(&context.source)
            .await
    }

    fn parse_with_allow_selectors(
        reader: &mut StringReader,
        allow_selectors: bool,
    ) -> Result<Message, CommandSyntaxError> {
        // Check if the UTF-16 length of this string exceeds 256 characters:
        let text = reader.remaining_part();
        let utf16_length = text.encode_utf16().count();
        if utf16_length > 256 {
            return Err(TOO_LONG_ERROR_TYPE.create(
                reader,
                TextComponent::text(utf16_length.to_string()),
                TextComponent::text(256.to_string()),
            ));
        }
        let text = text.to_string();

        if allow_selectors {
            let mut parts = Vec::new();
            let offset = reader.cursor();

            loop {
                let (start, parsed_selector) = loop {
                    match reader.peek() {
                        Some('@') => {
                            let start = reader.cursor();

                            let parser = EntitySelectorParser::new(reader, true);
                            match parser.parse_and_consume() {
                                Ok(parsed_selector) => break (start, parsed_selector),
                                Err(error) => {
                                    if !error.is(&MISSING_SELECTOR_TYPE_ERROR_TYPE)
                                        && error.is(&UNKNOWN_SELECTOR_TYPE_ERROR_TYPE)
                                    {
                                        return Err(error);
                                    }
                                    reader.set_cursor(start + 1);
                                }
                            }
                        }
                        Some(_) => reader.skip(),
                        None => {
                            return Ok(Message { text, parts });
                        }
                    }
                };

                parts.push(MessagePart {
                    range: StringRange {
                        start: start - offset,
                        end: reader.cursor() - offset,
                    },
                    entity_selector: parsed_selector,
                });
            }
        } else {
            reader.set_cursor(reader.total_length());
            Ok(Message {
                text,
                parts: Vec::new(),
            })
        }
    }
}

pub struct Message {
    pub text: String,
    pub parts: Vec<MessagePart>,
}

impl Message {
    /// Resolves this message into a text component with reference to a [`CommandSource`].
    pub async fn resolve_text_component(
        &self,
        source: &CommandSource,
    ) -> Result<TextComponent, CommandSyntaxError> {
        let allow_selectors = source.has_permission(ENTITY_SELECTOR_PERMISSION).await;
        if !self.parts.is_empty() && allow_selectors {
            let mut reading_to = self.parts[0].range.start;
            let mut result = TextComponent::text(self.text[0..reading_to].to_string());

            for part in &self.parts {
                let component = part.to_text_component(source).await?;
                if reading_to < part.range.start {
                    result = result.add_text(self.text[reading_to..part.range.start].to_string());
                }
                result = result.add_child(component);
                reading_to = part.range.end;
            }

            if reading_to < self.text.len() {
                result = result.add_text(self.text[reading_to..].to_string());
            }

            Ok(result)
        } else {
            Ok(TextComponent::text(self.text.clone()))
        }
    }
}

pub struct MessagePart {
    pub range: StringRange,
    pub entity_selector: EntitySelector,
}

impl MessagePart {
    /// Converts this part into a text component with reference to a [`CommandSource`].
    pub async fn to_text_component(
        &self,
        source: &CommandSource,
    ) -> Result<TextComponent, CommandSyntaxError> {
        let entities = self.entity_selector.find_entities(source).await?;

        let display_name_futures: Vec<_> = entities
            .iter()
            .map(|entity| entity.get_display_name())
            .collect();

        Ok(TextComponent::join_with_comma(
            futures::future::join_all(display_name_futures).await,
        ))
    }
}
