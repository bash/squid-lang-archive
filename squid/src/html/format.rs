use super::builders::Builder;
use super::super::ast::{HeadingLevel, Text, Inline};
use std::fmt::Debug;

///
/// HTML generation might need to be customized to invividual use.
/// A `Format` allows to customize output generation (e.g. custom tags, classes, ...)
///
// TODO: I need some feedback on the name `Format`
pub trait Format: Debug {
    fn heading(&self, builder: &mut Builder, level: HeadingLevel, content: String) {
        let tag = match level {
            HeadingLevel::Level1 => "h1",
            HeadingLevel::Level2 => "h2",
            HeadingLevel::Level3 => "h3",
            HeadingLevel::__NonExhaustive => unreachable!(),
        };

        builder.tag_start(tag).finish().text(content).tag_end(tag);
    }

    fn paragraph(&self, builder: &mut Builder, text: Text) {
        builder.tag_start("p").finish();

        self.text(builder, text);

        builder.tag_end("p");
    }

    fn text(&self, builder: &mut Builder, text: Text) {
        for inline in text {
            self.inline(builder, inline);
        }
    }

    fn inline(&self, builder: &mut Builder, inline: Inline) {
        match inline {
            Inline::LineBreak => {
                builder.tag_start("br").finish();
            }
            Inline::Chunk(text) => {
                builder.text(text);
            }
        }
    }
}

#[derive(Debug)]
pub struct DefaultFormat;

impl Format for DefaultFormat {}

#[cfg(test)]
mod tests {
    use super::*;
    use ast::HeadingLevel;

    #[test]
    fn default_heading_works() {
        let format = DefaultFormat;
        let mut builder = Builder::new();

        format.heading(&mut builder, HeadingLevel::Level1, "hello world".into());

        format.heading(&mut builder, HeadingLevel::Level2, "level 2".into());

        format.heading(&mut builder, HeadingLevel::Level3, "level 3".into());

        assert_eq!(
            "<h1>hello world</h1><h2>level 2</h2><h3>level 3</h3>",
            format!("{}", builder.consume())
        );
    }
}