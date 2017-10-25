use std::collections::VecDeque;
use super::constants;
use super::tokens::{Line, LineType};

macro_rules! parse_starter {
  ($line:expr, $starter:expr, $variant: ident) => {
    if $line.starts_with($starter) {
        Some(Line::$variant(
            $line.chars()
                .skip($starter.len())
                .collect()
        ))
    } else {
        None
    }
  }
}

macro_rules! detect_line_starter {
  ($line:expr, $starter:expr, $variant: ident) => {
    if $line.starts_with($starter) {
      return LineType::$variant;
    }
  }
}

#[derive(Debug)]
pub struct BlockTokenizer {
    lines: VecDeque<String>,
}

fn parse_decorator(line: &str) -> Line {
    let trimmed = line.trim();

    Line::Decorator(trimmed.chars().skip(1).take(trimmed.len() - 2).collect())
}

fn is_decorator(line: &str) -> bool {
    line.starts_with(constants::ANNOTATION_PREFIX_TOKEN) &&
        line.trim().ends_with(constants::ANNOTATION_SUFFIX_TOKEN)
}

fn is_divider(line: &str) -> bool {
    line.starts_with("---") && line.trim().chars().all(|c| c == '-')
}

pub fn is_blank(line: &str) -> bool {
    line.chars().all(char::is_whitespace)
}

fn get_line_type(line: &str) -> LineType {
    if is_divider(line) {
        return LineType::Divider;
    }

    if is_decorator(line) {
        return LineType::Decorator;
    }

    if is_blank(line) {
        return LineType::Blank;
    }

    detect_line_starter!(line, constants::HEADING1_TOKEN, Heading1);
    detect_line_starter!(line, constants::HEADING2_TOKEN, Heading2);
    detect_line_starter!(line, constants::HEADING3_TOKEN, Heading3);
    detect_line_starter!(line, constants::QUOTE_TOKEN, Quote);
    detect_line_starter!(line, constants::UNORDERED_LIST_TOKEN, UnorderedList);
    detect_line_starter!(line, constants::ORDERED_LIST_TOKEN, OrderedList);

    LineType::Text
}

impl Line {
    pub fn value(self) -> Option<String> {
        match self {
            Line::Blank => None,
            Line::Divider => None,
            Line::Heading1(value) => Some(value),
            Line::Heading2(value) => Some(value),
            Line::Heading3(value) => Some(value),
            Line::Text(value) => Some(value),
            Line::Quote(value) => Some(value),
            Line::Decorator(value) => Some(value),
            Line::UnorderedList(value) => Some(value),
            Line::OrderedList(value) => Some(value),
        }
    }
}

impl BlockTokenizer {
    pub fn new<S: Into<String>>(input: S) -> Self {
        let lines = input.into().lines().map(|line| line.to_string()).collect();

        BlockTokenizer { lines }
    }

    pub fn peek(&self) -> Option<LineType> {
        match self.lines.get(0) {
            None => None,
            Some(line) => Some(get_line_type(line)),
        }
    }

    pub fn consume(&mut self, line_type: LineType) -> Option<Line> {
        match self.consume_raw() {
            None => None,
            Some(line) => {
                match line_type {
                    LineType::Blank => Some(Line::Blank),
                    LineType::Divider => Some(Line::Divider),
                    LineType::Text => Some(Line::Text(line)),
                    LineType::Decorator => Some(parse_decorator(&line)),
                    LineType::Heading1 => parse_starter!(line, constants::HEADING1_TOKEN, Heading1),
                    LineType::Heading2 => parse_starter!(line, constants::HEADING2_TOKEN, Heading2),
                    LineType::Heading3 => parse_starter!(line, constants::HEADING3_TOKEN, Heading3),
                    LineType::Quote => parse_starter!(line, constants::QUOTE_TOKEN, Quote),
                    LineType::UnorderedList => {
                        parse_starter!(line, constants::UNORDERED_LIST_TOKEN, UnorderedList)
                    }
                    LineType::OrderedList => {
                        parse_starter!(line, constants::ORDERED_LIST_TOKEN, OrderedList)
                    }
                }
            }
        }
    }

    pub fn consume_raw(&mut self) -> Option<String> {
        self.lines.pop_front()
    }

    pub fn consume_line(&mut self) -> Option<Line> {
        match self.peek() {
            None => None,
            Some(line_type) => self.consume(line_type),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_works() {
        let mut tokenizer = BlockTokenizer::new("hello\nworld");

        assert_eq!(tokenizer.consume_line(), Some(Line::Text("hello".into())));
        assert_eq!(tokenizer.consume_line(), Some(Line::Text("world".into())));
    }

    #[test]
    fn heading_1_works() {
        let mut tokenizer = BlockTokenizer::new("# hello\nworld");

        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Heading1("hello".into()))
        );
        assert_eq!(tokenizer.consume_line(), Some(Line::Text("world".into())));
    }

    #[test]
    fn heading_2_works() {
        let mut tokenizer = BlockTokenizer::new("## heading 2");

        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Heading2("heading 2".into()))
        );
    }

    #[test]
    fn heading_3_works() {
        let mut tokenizer = BlockTokenizer::new("### lorem ipsum");

        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Heading3("lorem ipsum".into()))
        );
    }

    #[test]
    fn text_with_hash_works() {
        let mut tokenizer = BlockTokenizer::new(" # lorem ipsum");

        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Text(" # lorem ipsum".into()))
        );
    }

    #[test]
    fn quote_works() {
        let mut tokenizer = BlockTokenizer::new("> quote\n > quote\n>quote");

        assert_eq!(tokenizer.consume_line(), Some(Line::Quote("quote".into())));
        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Text(" > quote".into()))
        );
        assert_eq!(tokenizer.consume_line(), Some(Line::Text(">quote".into())));
    }

    #[test]
    fn decorator_works() {
        let mut tokenizer = BlockTokenizer::new("[code]\n[code]   \n [code] \n  [code]  \n[code");

        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Decorator("code".into()))
        );
        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Decorator("code".into()))
        );
        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Text(" [code] ".into()))
        );
        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Text("  [code]  ".into()))
        );
        assert_eq!(tokenizer.consume_line(), Some(Line::Text("[code".into())));
    }

    #[test]
    fn unordered_list_works() {
        let mut tokenizer = BlockTokenizer::new("- item\n - item\n-item");

        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::UnorderedList("item".into()))
        );
        assert_eq!(tokenizer.consume_line(), Some(Line::Text(" - item".into())));
        assert_eq!(tokenizer.consume_line(), Some(Line::Text("-item".into())));
    }

    #[test]
    fn ordered_list_works() {
        let mut tokenizer = BlockTokenizer::new(".  item\n . item\n.item");

        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::OrderedList(" item".into()))
        );
        assert_eq!(tokenizer.consume_line(), Some(Line::Text(" . item".into())));
        assert_eq!(tokenizer.consume_line(), Some(Line::Text(".item".into())));
    }

    #[test]
    fn divider_works() {
        let mut tokenizer = BlockTokenizer::new("---\n------- \n--\n ---\n---foobar");

        assert_eq!(tokenizer.consume_line(), Some(Line::Divider));
        assert_eq!(tokenizer.consume_line(), Some(Line::Divider));
        assert_eq!(tokenizer.consume_line(), Some(Line::Text("--".into())));
        assert_eq!(tokenizer.consume_line(), Some(Line::Text(" ---".into())));
        assert_eq!(
            tokenizer.consume_line(),
            Some(Line::Text("---foobar".into()))
        );
    }

    #[test]
    fn empty_works() {
        let mut tokenizer = BlockTokenizer::new("   \t");

        assert_eq!(tokenizer.consume_line(), Some(Line::Blank));
    }
}
