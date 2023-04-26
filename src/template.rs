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

    pub fn render(&self, context: &Context) -> String {
        self.segments
            .iter()
            .map(|s| s.render(context))
            .collect::<Vec<String>>()
            .join("")
    }

    pub(crate) fn new(segments: Segments<'a>) -> Self {
        Template {
            segments
        }
    }
}
