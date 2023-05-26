use std::fmt::Debug;
use crate::ContextRef;
use crate::reader::{Reader, Token};
use crate::context::Stack;


pub struct Template<'a>{
    segments: Segments<'a>
}

impl<'a> Template<'a> {
    pub fn from(input: &'a str) -> Result<Self, String> {
        let mut reader = Reader::new(input);
        let segments = parse(&mut reader, None)?;
        Ok(Template { segments })
    }

    pub fn render(&self, context: ContextRef<'a>) -> String {
        let mut stack = Stack::new(context);
        self.segments
            .iter()
            .map(|s| s.render(&mut stack))
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
    fn render(&self, stack: &mut Stack) -> String;
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
    fn render<'s>(&self, _stack: &mut Stack) -> String {
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
    fn render(&self, stack: &mut Stack) -> String {
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
}

impl<'a> Segment for SectionSegment<'a> {
    fn render(&self, stack: &mut Stack) -> String {
        let mut result = String::new();
        let len = stack.len();
        if stack.push(self.name) && stack.is_truthy() {
            while stack.current().is_some() {
                result.push_str( &self.children.render(stack));
                stack.next();
            }
        };
        stack.truncate(len);
        result
    }
}

impl<'a> Segment for Segments<'a> {
    fn render(&self, stack: &mut Stack) -> String {
        self.iter()
            .map(|child| child.render(stack))
            .collect::<Vec<_>>()
            .concat()
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
}

impl<'a> Segment for InvertedSectionSegment<'a> {
    fn render(&self, stack: &mut Stack)-> String {
        let len = stack.len();
        let pushed = stack.push(self.name);
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
