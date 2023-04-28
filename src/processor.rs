use std::fmt::Debug;
use crate::context::{Stack};


pub(crate) trait Segment<'a>: Debug {
    fn render<'b>(&self, stack: &Stack<'b>) -> String;
    fn substitute(&self) {}
}

pub(crate) type Segments<'a> = Vec<Box<dyn Segment<'a> + 'a>>;


#[derive(Debug)]
pub(crate) struct TextSegment<'a> {
    text: &'a str
}

impl<'a> TextSegment<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        TextSegment {
            text
        }
    }
}

impl<'a> Segment<'a> for TextSegment<'a> {
    fn render<'b>(&self, _stack: &Stack<'b>) -> String {
        self.text.to_string()
    }
}


#[derive(Debug)]
pub(crate) struct ValueSegment<'a> {
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

impl<'a> Segment<'a> for ValueSegment<'a> {
    fn render<'b>(&self, stack: &Stack<'b>) -> String {
        let text = stack.resolve(self.name);
        match self.is_escaped {
            true => html_escape(text.to_string()),
            false => text.to_string()
        }
    }
}


#[derive(Debug)]
pub(crate) struct SectionSegment<'a> {
    name: &'a str,
    children: Segments<'a>
}

impl<'a> SectionSegment<'a> {
    pub(crate) fn new(name: &'a str, children: Segments<'a>) -> Self {
        SectionSegment {
            name,
            children
        }
    }
}

impl<'a> Segment<'a> for SectionSegment<'a> {
    fn render<'b>(&self, _stack: &Stack<'b>) -> String {
        self.name.to_string()
    }
}


#[derive(Debug)]
pub(crate) struct InvertedSectionSegment<'a> {
    name: &'a str,
    children: Segments<'a>
}

impl<'a> InvertedSectionSegment<'a> {
    pub(crate) fn new(name: &'a str, children: Segments<'a>) -> Self {
        InvertedSectionSegment {
            name,
            children
        }
    }
}

impl<'a> Segment<'a> for InvertedSectionSegment<'a> {
    fn render<'b>(&self, _stack: &Stack<'b>) -> String {
        self.name.to_string()
    }
}


#[derive(Debug)]
pub(crate) struct PartialSegment<'a> {
    name: &'a str,
    is_dynamic: bool,
    children: Option<Segments<'a>>
}

impl<'a> PartialSegment<'a> {
    pub(crate) fn new(name: &'a str, is_dynamic: bool, children: Option<Segments<'a>>) -> Self {
        PartialSegment {
            name,
            is_dynamic,
            children
        }
    }
}

impl<'a> Segment<'a> for PartialSegment<'a> {
    fn render<'b>(&self, _stack: &Stack<'b>) -> String {
        self.name.to_string()
    }
}


#[derive(Debug)]
pub(crate) struct BlockSegment<'a> {
    name: &'a str,
    children: Segments<'a>
}

impl<'a> BlockSegment<'a> {
    pub(crate) fn new(name: &'a str, children: Segments<'a>) -> Self {
        BlockSegment {
            name,
            children
        }
    }
}

impl<'a> Segment<'a> for BlockSegment<'a> {
    fn render<'b>(&self, _stack: &Stack<'b>) -> String {
        self.name.to_string()
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
