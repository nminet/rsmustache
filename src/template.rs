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
        render_segments(&self.segments, stack, indent, partials)
    }
}


fn parse<'a>(
    reader: &mut Reader<'a>, section: Option<&str>
) -> Result<Segments, String> {
    let mut segments = Segments::new();
    while let Some(token) = reader.pop_front() {
        match token {
            Token::Text(text, starts_new_line) =>
                segments.push(
                    Segment::Text(
                        text.to_owned(),
                        starts_new_line
                    )
                ),
            Token::Value(name, is_escaped, starts_new_line) =>
                segments.push(
                    Segment::Value(
                        name.to_owned(),
                        is_escaped, starts_new_line
                    )
                ),
            Token::Section(name) =>
                segments.push(
                    Segment::Section(
                        name.to_owned(),
                        parse(reader, Some(name))?
                    )
                ),
            Token::InvertedSection(name) =>
                segments.push(
                    Segment::InvertedSection(
                        name.to_owned(),
                        parse(reader, Some(name))?
                    )
                ),
            Token::Block(name) =>
                segments.push(
                    Segment::Block(
                        name.to_owned(),
                        parse(reader, Some(name))?
                    )
                ),
            Token::Parent(name, is_dynamic, indent) => {
                let parameters = parse(reader, Some(name))?
                    .into_iter()
                    .filter_map(|s|
                        match s {
                            Segment::Block(name, children) => Some((name, children)),
                            _ => None
                        }
                    ).collect::<HashMap<_, _>>();
                segments.push(
                    Segment::Partial(
                        name.to_owned(),
                        indent.to_owned(),
                        is_dynamic,
                        Some(parameters)
                    )
                )
            },
            Token::EndSection(name) => {
                if section != Some(name) {
                   return Err(format!("unexpected end of section {}", name));
                }
                break;
            },
            Token::Partial(name, is_dynamic, indent) =>
                segments.push(
                    Segment::Partial(
                        name.to_owned(),
                        indent.to_owned(),
                        is_dynamic,
                        None
                    )
                ),
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


#[derive(Debug, Clone)]
enum Segment {
    Text(String, bool),
    Value(String, bool, bool),
    Section(String, Segments),
    InvertedSection(String, Segments),
    Block(String, Segments),
    Partial(String, String, bool, Option<HashMap<String, Segments>>)
}

type Segments = Vec<Segment>;


fn render_segment(
    segment: &Segment,
    stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
) -> String {
    match segment {
        Segment::Text(text, starts_new_line) =>
            render_text(
                text, *starts_new_line,
                indent
            ),
        Segment::Value(name, is_escaped, starts_new_line) =>
            render_value(
                name, *is_escaped, *starts_new_line,
                stack, indent
            ),
        Segment::Section(name, children) =>
            render_section(
                name, children,
                stack, indent, partials
            ),
        Segment::InvertedSection(name, children) =>
            render_inverted_section(
                name, children,
                stack, indent, partials
            ),
        Segment::Block(_, segments) =>
            render_segments(
                segments,
                stack, indent, partials
            ),
        Segment::Partial(name, children_indent, is_dynamic, parameters) =>
            render_partial(
                name, children_indent, *is_dynamic, parameters,
                stack, indent, partials
            )
    }
}

fn render_text(
    text: &str, starts_new_line: bool,
    indent: &str
) -> String {
    if indent.is_empty() {
        text.to_owned()
    } else {
        let mut result = String::new();
        if starts_new_line {
            result.push_str(indent);
        }
        result.push_str(text);
        if let Some(idx) = text.find('\n') {
            if idx < text.len() - 1 {
                // eol inside text requires indenting next line
                let indent_after = "\n".to_owned() + indent;
                result = result.replace("\n", &indent_after);
                if text.ends_with("\n") {
                    // trailing eol should not indent next line
                    result.truncate(result.len() - indent.len());
                }
            }
        }
        result
    }
}

fn render_value(
    name: &str, is_escaped: bool, starts_new_line: bool,
    stack: &mut Stack, indent: &str
) -> String {
    let value = if starts_new_line && !indent.is_empty() {
        let mut value = indent.to_owned();
        if let Some(text) = stack.get(name) {
            value.push_str(&text);
        }
        value
    } else {
        stack.get(name).unwrap_or_default()
    };
    match is_escaped {
        true => html_escape(value),
        false => value
    }
}

fn render_section(
    name: &str, children: &Segments,
    stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
) -> String {
    let mut result = String::new();
    let len = stack.len();
    if stack.push(name) && stack.is_truthy() {
        while stack.current().is_some() {
            result.push_str(&render_segments(children, stack, indent, partials));
            stack.next();
        }
    };
    stack.truncate(len);
    result
}

fn render_inverted_section(
    name: &str, children: &Segments,
    stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
) -> String {
    let len = stack.len();
    let pushed = stack.push(name);
    let falsy = !pushed || !stack.is_truthy();
    stack.truncate(len);
    if falsy {
        render_segments(children, stack, indent, partials)
    } else {
        "".to_owned()
    }    
}

fn render_partial(
    name: &str, children_indent: &str, is_dynamic: bool, parameters: &Option<HashMap<String, Segments>>,
    stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
) -> String {
    if let Some(store) = partials {
        let maybe_template = if is_dynamic {
            stack.get(name).map_or(None, |it| store.get(&it))
        } else {
            store.get(name)
        };
        match maybe_template {
            Some(template) => {
                let next_indent = indent.to_owned() + children_indent;
                if let Some(parameters) = parameters {
                    let segments = substitute(&template.segments, parameters);
                    render_segments(&segments, stack, &next_indent, partials)
                } else {
                    render_segments(&template.segments, stack, &next_indent, partials)
                }  
            },
            None => "".to_owned()
        }
    } else {
        "".to_owned()
    }
}

fn render_segments(
    segments: &Segments,
    stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
) -> String {
    segments.iter()
        .map(|segment| render_segment(segment, stack, indent, partials))
        .collect::<Vec<_>>()
        .concat()
}


fn substitute(segments: &Segments, parameters: &HashMap<String, Segments>) -> Segments {
    let mut result = Vec::new();
    for segment in segments {
        match segment {
            Segment::Text(_, _) | Segment::Value(_, _, _) => {
                result.push(
                    segment.clone()
                )
            },
            Segment::Section(name, segments) => {
                result.push(
                    Segment::Section(name.to_owned(), substitute(segments, parameters))
                )
            },
            Segment::InvertedSection(name, segments) => {
                result.push(
                    Segment::InvertedSection(name.to_owned(), substitute(segments, parameters))
                )
            },
            Segment::Block(name, segments) => {
                let updated = parameters.get(name).map_or_else(
                    || substitute(segments, parameters),
                    |segments| segments.clone() 
                );
                result.push(
                    Segment::Block(name.to_owned(), updated)
                );
            },
            Segment::Partial(name, indent, is_dynamic, partial_parameters) => {
                let updated = if let Some(partial_parameters) = partial_parameters {
                    let mut updated = HashMap::new();
                    updated.extend(partial_parameters.clone().into_iter());
                    updated.extend(parameters.clone().into_iter());
                    Some(updated)
                } else {
                    None
                };
                result.push(
                    Segment::Partial(name.to_owned(), indent.to_owned(), *is_dynamic, updated)
                )
            }
        }
    };
    result
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
