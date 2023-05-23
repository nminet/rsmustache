use std::fmt::Debug;
use crate::reader::{Reader, Token};
use crate::context::{Context, Stack, PushResult};


pub struct Template<'a>{
    segments: Segments<'a>
}

impl<'a> Template<'a> {
    pub fn from(input: &'a str) -> Result<Self, String> {
        let mut reader = Reader::new(input);
        let segments = parse(&mut reader, None)?;
        Ok(Template { segments })
    }

    pub fn render<'c, T>(&self, context: &'c T) -> String
    where &'c T: Context<'c> {
        let stack = Stack::root(&context);
        self.segments
            .iter()
            .map(|s| s.render(&stack))
            .collect::<Vec<_>>()
            .concat()
    }
}


fn parse<'a, 's: 'a>(
    reader: &mut Reader<'a>, section: Option<&'s str>
) -> Result<Segments<'a>, String> {
    let mut segments = Segments::new();
    while let Some(token) = reader.pop_front() {
        match token {
            Token::Text(text) => {
                segments.push(Box::new(
                    TextSegment::new(text)
                ))
            },
            Token::Value(name, is_escaped) => {
                segments.push(Box::new(
                    ValueSegment::new(name, is_escaped)
                ))
            },
            Token::Section(name) => {
                segments.push(Box::new(
                    SectionSegment::new(name, parse(reader, Some(name))?)
                ))
            },
            Token::InvertedSection(name) => {
                segments.push(Box::new(
                    InvertedSectionSegment::new(name, parse(reader, Some(name))?)
                ))
            },
            Token::EndSection(name) => {
                if section != Some(name) {
                   return Err(format!("unexpected end of section {}", name));
                }
                break;
            },
            Token::Delimiters(od, cd) => {
                reader.set_delimiters(od, cd);
            },
            Token::Comment(_) => {
            },
            Token::Error(error) => {
                return Err(format!("reader error: {}", error));
            }
        }
    }
    Ok(segments)
}


trait Segment: Debug {
    fn render(&self, stack: &Stack) -> String;
    fn substitute(&self) {}
}

type Segments<'a> = Vec<Box<dyn Segment + 'a>>;


#[derive(Debug)]
struct TextSegment<'a> {
    text: &'a str
}

impl<'a> TextSegment<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        TextSegment {
            text
        }
    }
}

impl<'a> Segment for TextSegment<'a> {
    fn render<'s>(&self, _stack: &Stack) -> String {
        self.text.to_string()
    }
}


#[derive(Debug)]
struct ValueSegment<'a> {
    name: &'a str,
    is_escaped: bool
}

impl<'a> ValueSegment<'a> {
    pub(crate) fn new(name: &'a str, is_escaped: bool) -> Self {
        ValueSegment {
            name,
            is_escaped
        }
    }
}

impl<'a> Segment for ValueSegment<'a> {
    fn render(&self, stack: &Stack) -> String {
        let text = stack.get(self.name).unwrap_or_default();
        match self.is_escaped {
            true => html_escape(text.to_string()),
            false => text.to_string()
        }
    }
}


#[derive(Debug)]
struct SectionSegment<'a> {
    name: &'a str,
    children: Segments<'a>
}

impl<'a> SectionSegment<'a> {
    fn new(name: &'a str, children: Segments<'a>) -> Self {
        SectionSegment {
            name,
            children
        }
    }

    fn render_single(&self, stack: &Stack) -> String {
        if stack.is_truthy() {
        self.children
            .iter()
            .map(|child| child.render(stack))
            .collect::<Vec<String>>()
            .concat()
        } else {
            String::new()
        }
    }
}

impl<'a> Segment for SectionSegment<'a> {
    fn render(&self, stack: &Stack) -> String {
        match stack.push(self.name) {
            PushResult::Single(stack) =>
                self.render_single(&stack),
            PushResult::List(stacks) =>
                stacks.iter()
                    .map(|stack| self.render_single(stack))
                    .collect::<_>(),
            PushResult::None => String::new()
        }
    }
}


#[derive(Debug)]
struct InvertedSectionSegment<'a> {
    name: &'a str,
    children: Segments<'a>
}

impl<'a> InvertedSectionSegment<'a> {
    fn new(name: &'a str, children: Segments<'a>) -> Self {
        InvertedSectionSegment {
            name,
            children
        }
    }

    fn render_inverted(&self, is_falsy: bool, stack:&Stack) -> String {
        if is_falsy {
            self.children
                .iter()
                .map(|child| child.render(stack))
                .collect::<Vec<String>>()
                .concat()
         } else {
            String::new()
         }
    }
}

impl<'a> Segment for InvertedSectionSegment<'a> {
    fn render(&self, stack: &Stack)-> String {
        match stack.push(self.name) {
            PushResult::Single(it) => self.render_inverted(!it.is_truthy(), &stack),
            PushResult::List(it) => self.render_inverted(it.is_empty(), &stack),
            PushResult::None => self.render_inverted(true, &stack)
        }
    }
}


fn html_escape(input: String) -> String {
    input.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#39;")
        .replace("/", "&#47;")
        .replace("=", "&#61;")
        .replace("`", "&#96;")
}
