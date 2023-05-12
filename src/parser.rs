use crate::reader::{
    Reader,
    Token
};
use crate::processor::{
    Segments,
    TextSegment,
    SectionSegment,
    InvertedSectionSegment,
    BlockSegment,
    PartialSegment,
    ValueSegment,
    Segment
};


pub(crate) fn process<'a, 's: 'a>(
    reader: &mut Reader<'a>, section: Option<&'s str>
) -> Result<Segments<'a>, String> {
    let mut segments = Segments::new();
    while let Some(token) = reader.pop_front() {
        match token {
            Token::Text(text) => {
                segments.add_item(
                    TextSegment::new(text)
                )
            },
            Token::Value(name, is_escaped) => {
                segments.add_item(
                    ValueSegment::new(name, is_escaped)
                )
            },
            Token::Section(name) => {
                segments.add_item(
                    SectionSegment::new(name, process(reader, Some(name))?)
                );
            },
            Token::InvertedSection(name) => {
                segments.add_item(
                    InvertedSectionSegment::new(name, process(reader, Some(name))?)
                )
            },
            Token::Partial(name, is_dynamic, is_parent) => {
                let children = if is_parent {
                    Some(process(reader, Some(name))?)
                } else {
                    None
                };
                segments.add_item(
                    PartialSegment::new(name, is_dynamic, children)
                )
            },
            Token::Block(name) => {
                segments.add_item(
                    BlockSegment::new(name, process(reader, Some(name))?)
                )
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



trait SegmentsOps<'a> {
    fn add_item(&mut self, item: impl Segment + 'a);
}

impl<'a> SegmentsOps<'a> for Segments<'a> {
    fn add_item(&mut self, item: impl Segment + 'a) {
        self.push(Box::new(item))
    }
}
