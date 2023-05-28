use std::collections::HashMap;
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
        self.render_internal(&mut stack, "", None)
    }

    pub fn render_with_partials(
        &self, context: ContextRef, partials: &dyn TemplateStore
    ) -> String {
        let mut stack = Stack::new(context);
        self.render_internal(&mut stack, "", Some(partials))
    }

    pub(crate) fn render_internal(
        &self, stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>,
    ) -> String {
        self.segments.render(stack, indent, partials)
    }
}


fn parse<'a>(
    reader: &mut Reader<'a>, section: Option<&str>
) -> Result<Segments, String> {
    let mut segments = Segments::new();
    while let Some(token) = reader.pop_front() {
        match token {
            Token::Text(text, starts_new_line) => {
                segments.push(Box::new(
                    TextSegment::new(text, starts_new_line)
                ))
            },
            Token::Value(name, is_escaped, starts_new_line) => {
                segments.push(Box::new(
                    ValueSegment::new(name, is_escaped, starts_new_line)
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
            Token::Partial(name, is_dynamic, indent) => {
                segments.push(Box::new(
                    PartialSegment::new(name, is_dynamic, indent)
                ))
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
    fn render(
        &self,stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
    ) -> String;
}

type Segments = Vec<Box<dyn Segment>>;


#[derive(Debug)]
struct TextSegment {
    text: String,
    starts_new_line: bool
}

impl TextSegment {
    pub(crate) fn new(text: &str, starts_new_line: bool) -> Self {
        TextSegment {
            text: text.to_owned(),
            starts_new_line
        }
    }
}

impl Segment for TextSegment {
    fn render(
        &self, _stack: &mut Stack, indent: &str, _partials: Option<&dyn TemplateStore>
    ) -> String {
        if indent.is_empty() {
            self.text.clone()
        } else {
            let mut result = String::new();
            if self.starts_new_line {
                result.push_str(indent);
            }
            result.push_str(&self.text);
            if let Some(idx) = self.text.find('\n') {
                if idx < self.text.len() - 1 {
                    // eol inside text requires indenting next line
                    let indent_after = "\n".to_owned() + indent;
                    result = result.replace("\n", &indent_after);
                    if self.text.ends_with("\n") {
                        // trailing eol should not indent next line
                        result.truncate(result.len() - indent.len());
                    }
                }
            }
            result
        }
    }
}


#[derive(Debug)]
struct ValueSegment {
    name: String,
    is_escaped: bool,
    starts_new_line: bool
}

impl ValueSegment {
    pub(crate) fn new(name: &str, is_escaped: bool, starts_new_line: bool) -> Self {
        ValueSegment {
            name: name.to_owned(),
            is_escaped,
            starts_new_line
        }
    }
}

impl Segment for ValueSegment {
    fn render(
        &self, stack: &mut Stack, indent: &str, _partials: Option<&dyn TemplateStore>
    ) -> String {
        let value = if self.starts_new_line && !indent.is_empty() {
            let mut value = indent.to_owned();
            if let Some(text) = stack.get(&self.name) {
                value.push_str(&text);
            }
            value
        } else {
            stack.get(&self.name).unwrap_or_default()
        };
        match self.is_escaped {
            true => html_escape(value),
            false => value
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
    fn render(
        &self, stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
    ) -> String {
        let mut result = String::new();
        let len = stack.len();
        if stack.push(&self.name) && stack.is_truthy() {
            while stack.current().is_some() {
                result.push_str(&self.children.render(stack, indent, partials));
                stack.next();
            }
        };
        stack.truncate(len);
        result
    }
}

impl Segment for Segments {
    fn render(
        &self, stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
    ) -> String {
        self.iter()
            .map(|child| child.render(stack, indent, partials))
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
    fn render(
        &self, stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
    )-> String {
        let len = stack.len();
        let pushed = stack.push(&self.name);
        let falsy = !pushed || !stack.is_truthy();
        stack.truncate(len);
        if falsy {
            self.children.render(stack, indent, partials)
        } else {
            String::new()
        }
    }
}

#[derive(Debug)]
struct PartialSegment {
    name: String,
    is_dynamic: bool,
    indent: String
}

impl PartialSegment {
    fn new(name: &str, is_dynamic: bool, indent: &str) -> Self {
        PartialSegment {
            name: name.to_owned(),
            is_dynamic,
            indent: indent.to_owned()
        }
    }
}

impl Segment for PartialSegment {
    fn render(
        &self, stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
    ) -> String {
        if let Some(store) = partials {
            let maybe_template = if self.is_dynamic {
                stack.get(&self.name).map_or(None, |it| store.get(&it))
            } else {
                store.get(&self.name)
            };
            match maybe_template {
                Some(template) => {
                    let next_indent = self.indent.to_owned() + indent;
                    template.render_internal(stack, &next_indent, partials)
                },
                None => String::new()
            }
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


pub trait TemplateStore {
    fn get(&self, name: &str) -> Option<&Template>;
}


pub struct TemplateMap {
    templates: HashMap<String, Template>
}

impl TemplateMap {
    pub fn new() -> Self {
        TemplateMap { templates: HashMap::new() }
    }

    pub fn load(&mut self, name: &str, input: &str) -> Result<(), String> {
        let template = Template::from(input)?;
        self.templates.insert(name.to_owned(), template);
        Ok(())
    }
}

impl TemplateStore for TemplateMap {
    fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }
}
