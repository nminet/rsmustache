use crate::context::Stack;
use crate::reader::{
    Reader
};
use crate::{parser, Context};
use crate::processor::Segments;


pub struct Template<'a>{
    segments: Segments<'a>
}

impl<'a> Template<'a> {
    pub fn from(input: &'a str) -> Result<Self, String> {
        let mut reader = Reader::new(input);
        let segments = parser::process(&mut reader, None)?;
        Ok(Template::new(segments))
    }

    pub fn render<'b>(&self, context: Context<'b>) -> String {
        let stack = Stack::from(context);
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
