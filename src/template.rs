use std::fmt::Debug;
use crate::ContextRef;
use crate::reader::{Reader, Token};
use crate::context::Stack;


pub struct Template {
    segments: Segments
}

impl Template {
    pub fn from(input: &str) -> Result<Self, String> {
        let mut reader = Reader::new(input);
        let segments = parse(&mut reader, None)?;
        Ok(Template { segments })
    }

    pub fn render(&self, context: ContextRef) -> String {
        let mut stack = Stack::new(context);
        self.segments.render(&mut stack)
    }
}


fn parse<'a>(
    reader: &mut Reader<'a>, section: Option<&str>
) -> Result<Segments, String> {
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
    fn render(&self, stack: &mut Stack) -> String;
}

type Segments = Vec<Box<dyn Segment>>;


#[derive(Debug)]
struct TextSegment {
    text: String
}

impl TextSegment {
    pub(crate) fn new(text: &str) -> Self {
        TextSegment {
            text: text.to_owned()
        }
    }
}

impl Segment for TextSegment {
    fn render(&self, _stack: &mut Stack) -> String {
        self.text.clone()
    }
}


#[derive(Debug)]
struct ValueSegment {
    name: String,
    is_escaped: bool
}

impl ValueSegment {
    pub(crate) fn new(name: &str, is_escaped: bool) -> Self {
        ValueSegment {
            name: name.to_owned(),
            is_escaped
        }
    }
}

impl Segment for ValueSegment {
    fn render(&self, stack: &mut Stack) -> String {
        let text = stack.get(&self.name).unwrap_or_default();
        match self.is_escaped {
            true => html_escape(text),
            false => text
        }
    }
}


#[derive(Debug)]
struct SectionSegment {
    name: String,
    children: Segments
}

impl SectionSegment {
    fn new(name: &str, children: Segments) -> Self {
        SectionSegment {
            name: name.to_owned(),
            children
        }
    }
}

impl<'a> Segment for SectionSegment {
    fn render(&self, stack: &mut Stack) -> String {
        let mut result = String::new();
        let len = stack.len();
        if stack.push(&self.name) && stack.is_truthy() {
            while stack.current().is_some() {
                result.push_str(&self.children.render(stack));
                stack.next();
            }
        };
        stack.truncate(len);
        result
    }
}

impl Segment for Segments {
    fn render(&self, stack: &mut Stack) -> String {
        self.iter()
            .map(|child| child.render(stack))
            .collect::<Vec<_>>()
            .concat()
    }
}

#[derive(Debug)]
struct InvertedSectionSegment {
    name: String,
    children: Segments
}

impl InvertedSectionSegment {
    fn new(name: &str, children: Segments) -> Self {
        InvertedSectionSegment {
            name: name.to_owned(),
            children
        }
    }
}

impl Segment for InvertedSectionSegment {
    fn render(&self, stack: &mut Stack)-> String {
        let len = stack.len();
        let pushed = stack.push(&self.name);
        let falsy = !pushed || !stack.is_truthy();
        stack.truncate(len);
        if falsy {
            self.children.render(stack)
        } else {
            String::new()
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
