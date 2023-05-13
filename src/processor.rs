use std::fmt::Debug;
use crate::context::{Stack, PushResult};


pub(crate) trait Segment: Debug {
    fn render(&self, stack: &Stack) -> String;
    fn substitute(&self) {}
}

pub(crate) type Segments<'a> = Vec<Box<dyn Segment + 'a>>;


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

impl<'a> Segment for TextSegment<'a> {
    fn render<'s>(&self, _stack: &Stack) -> String {
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

    fn render_single(&self, stack: &Stack) -> String {
//        println!("render {:?} with {:?}", self, stack);
        self.children
            .iter()
            .map(|child| child.render(stack))
            .collect::<Vec<String>>()
            .concat()
    }
}

impl<'a> Segment for SectionSegment<'a> {
    fn render(&self, stack: &Stack) -> String {
        match stack.push(self.name, true) {
            PushResult::Single(stack) =>
                 if stack.is_truthy() {
                    self.render_single(&stack)
                 } else {
                    String::new()
                 },
            PushResult::List(stacks) =>
                stacks.iter()
                    .map(|stack| self.render_single(stack))
                    .collect::<_>(),
            PushResult::None => String::new()
        }
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
        match stack.push(self.name, true) {
            PushResult::Single(it) => self.render_inverted(!it.is_truthy(), &stack),
            PushResult::List(it) =>self.render_inverted(it.is_empty(), &stack),
            PushResult::None => String::new()
        }
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

impl<'a> Segment for PartialSegment<'a> {
    fn render(&self, _stack: &Stack) -> String {
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

impl<'a> Segment for BlockSegment<'a> {
    fn render(&self, _stack: &Stack) -> String {
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
