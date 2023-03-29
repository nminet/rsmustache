
pub(crate) trait Segment {
    fn render(&self) -> String;
    fn substitute(&self);
}
