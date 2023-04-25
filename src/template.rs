use crate::reader::{
    Reader
};
use crate::parser;
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

    pub fn render(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.render())
            .collect::<Vec<String>>()
            .join("")
    }

    pub(crate) fn new(segments: Segments<'a>) -> Self {
        Template {
            segments
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_text_only() {
        let text = "hello, world!\n";
        let template = Template::from(text).unwrap();
        assert_eq!(template.render(), text);
    }
}
