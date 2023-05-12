use crate::reader::Reader;
use crate::parser::process;
use crate::context::{Context, Stack};
use crate::processor::Segments;


pub struct Template<'a>{
    segments: Segments<'a>
}

impl<'a> Template<'a> {
    pub fn from(input: &'a str) -> Result<Self, String> {
        let mut reader = Reader::new(input);
        let segments = process(&mut reader, None)?;
        Ok(Template::new(segments))
    }

    pub fn render<'c, T>(&self, context: &'c T) -> String
    where &'c T: Context<'c> {
        let stack = Stack::root(&context);
        self.segments
            .iter()
            .map(|s| s.render(&stack))
            .collect::<Vec<String>>()
            .join("")
    }

    pub(crate) fn new(segments: Segments<'a>) -> Self {
        Template {
            segments
        }
    }
}
