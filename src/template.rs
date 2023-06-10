use std::collections::HashMap;
use crate::ContextRef;
use crate::reader::{Reader, Token};
use crate::context::Stack;

/// Represent a compiled Mustache template.
pub struct Template {
    segments: Segments,
}

impl Template {
    /// Compile a Mustache template.
    /// 
    /// If the compilation fails, return [Result::Err] with a String giving
    /// information about the failure (TODO: diagnostics should be improved).
    /// 
    /// Otherwise return [Result::Ok] with a [Template] ready to render.
    pub fn from(input: &str) -> Result<Self, String> {
        let mut reader = Reader::new(input);
        let segments = parse(&mut reader, None)?.0;
        Ok(Template { segments })
    }

    /// Render [Template] from data supplied by [ContextRef].
    ///
    /// Instances of [Template] will always render sucessfully provided the
    /// [Context](crate::Context) does not block or [panic!].
    /// As per Mustache specification, items that are not found will be falsy
    /// in section position and render to an empty string in interpolation
    /// position.
    /// 
    /// As there is no [TemplateStore] all partials will result in context
    /// misses, producing no text.
    pub fn render(&self, context: ContextRef) -> String {
        let mut stack = Stack::new(context);
        self.render_internal(&mut stack, "", None)
    }

    /// Render [Template] using a [ContextRef] and [TemplateStore].
    /// 
    /// If the partial is not found in [TemplateStore], it is handled
    /// as a context miss (falsy/blank).
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
    reader: &mut Reader<'a>, section: Option<(&str, &str)>
) -> Result<(Segments, usize), String> {
    let mut segments = Segments::new();
    let mut before_tag: usize = 0;
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
            Token::Section(name, after_open, is_seqcheck) => {
                let qualifier = if is_seqcheck { "?" } else { "" };
                let (children, before_close) = parse(reader, Some((name, qualifier)))?;
                segments.push(
                    Segment::Section(
                        name.to_owned(), after_open, before_close, is_seqcheck, children
                    )
                )
            },
            Token::InvertedSection(name) =>
                segments.push(
                    Segment::InvertedSection(
                        name.to_owned(),
                        parse(reader, Some((name, &"")))?.0
                    )
                ),
            Token::Block(name) =>
                segments.push(
                    Segment::Block(
                        name.to_owned(),
                        parse(reader, Some((name, &"")))?.0
                    )
                ),
            Token::Parent(name, is_dynamic, indent) => {
                let parameters = parse(reader, Some((name, &"")))?.0
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
            Token::EndSection(name, qualifier, pos) => {
                if section != Some((name, qualifier)) {
                   return Err(format!("unexpected end of section {}", name));
                }
                before_tag = pos;
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
    Ok((segments, before_tag))
}


#[derive(Clone)]
enum Segment {
    Text(String, bool),
    Value(String, bool, bool),
    Section(String, usize, usize, bool, Segments),
    InvertedSection(String, Segments),
    Block(String, Segments),
    Partial(String, String, bool, Option<HashMap<String, Segments>>),
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
        Segment::Section(name, start, end, is_seqcheck, children) =>
            render_section(
                name, *is_seqcheck, children, *start, *end,
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
    name: &str, is_seqcheck: bool, children: &Segments, start: usize, end: usize,
    stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
) -> String {
    let mut result = String::new();
    let len = stack.len();
    if stack.push(name, Some((start, end))) {
        if is_seqcheck {
            let must_render = stack.in_sequence() && stack.current().is_some();
            stack.truncate(len);
            if must_render {
                result.push_str(&render_segments(children, stack, indent, partials));
            }
        } else if stack.in_sequence() || !stack.is_falsy() {
            while stack.current().is_some() {
                result.push_str(&render_segments(children, stack, indent, partials));
                stack.next();
            };
            stack.truncate(len);
        }
    }
    result
}

fn render_inverted_section(
    name: &str, children: &Segments,
    stack: &mut Stack, indent: &str, partials: Option<&dyn TemplateStore>
) -> String {
    let len = stack.len();
    let pushed = stack.push(name, None);
    let must_render = !pushed || stack.is_falsy() || stack.current().is_none();
    stack.truncate(len);
    if must_render {
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
        if let Some(template) = maybe_template {
            let next_indent = indent.to_owned() + children_indent;
            if let Some(parameters) = parameters {
                let segments = substitute(&template.segments, parameters);
                render_segments(&segments, stack, &next_indent, partials)
            } else {
                render_segments(&template.segments, stack, &next_indent, partials)
            }
        } else {
            "".to_owned()
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
    segments.iter()
        .map(|segment|
            substitute_segment(segment, parameters)
        ).collect::<Vec<_>>()
}

fn substitute_segment(segment: &Segment, parameters: &HashMap<String, Segments>) -> Segment {
    match segment {
        Segment::Text(_, _) | Segment::Value(_, _, _) =>
            segment.clone(),
        Segment::Section(name, after_open, before_close, is_seqcheck, segments) =>
            Segment::Section(
                name.to_owned(), *after_open, *before_close, *is_seqcheck, substitute(segments, parameters)
            ),
        Segment::InvertedSection(name, segments) =>
            Segment::InvertedSection(
                name.to_owned(), substitute(segments, parameters)
            ),
        Segment::Block(name, segments) => {
            let updated = parameters.get(name).map_or_else(
                || substitute(segments, parameters),
                |segments| segments.clone() 
            );
            Segment::Block(name.to_owned(), updated)
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
            Segment::Partial(name.to_owned(), indent.to_owned(), *is_dynamic, updated)
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


/// Template resolver
/// 
/// This trait is used to retreive compiled [Template] by name.
pub trait TemplateStore {
    fn get(&self, name: &str) -> Option<&Template>;
}


/// Pre-compiled [Template] instances.
pub struct TemplateMap {
    templates: HashMap<String, Template>,
}

impl TemplateMap {
    /// Create a [TemplateMap] for a map of name to Mustache source code.
    /// 
    /// If any of the Mustache template does not compile the result is a [Result::Err].
    pub fn new(input: HashMap<&str, &str>) -> Result<Self, String> {
        let mut templates = HashMap::new();
        for (name, text) in input {
            let template = match Template::from(text) {
                Ok(template) => template,
                Err(err) => return Err(format!("{}: {}", name, err))
            };
            templates.insert(name.to_owned(), template);
        }
        Ok(TemplateMap { templates })
    }
}

impl TemplateStore for TemplateMap {
    fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }
}
