use crate::processor::Segment;

pub struct Template {
    segments: Vec<Box<dyn Segment>>
}

impl Template {
    pub fn new() -> Self {
        Template {
            segments: Vec::new()
        }
    }

    pub fn render() -> String {
        String::from("ok")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dummy() {
        let t = Template::new();
        assert_eq!(true, false);
    }
}
