struct Reader {
    input: String,
    open_delimiter: String,
    close_delimiter: String,
    pos: usize,
    mark: usize
}

impl Reader {
    fn new(input: String) -> Self {
        Reader { 
            input,
            open_delimiter: String::from("{{"),
            close_delimiter: String::from("}}"),
            pos: 0,
            mark: 0
        }
    }

    fn read_until(&self, found: &str) -> bool {
        let start = self.pos;
        let after = start;
        false
    }

    fn fetch_text(&self) -> &str {
        self.input.as_str()
    }
}


trait ReaderStringOps {
    fn trim_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> &str;
    fn is_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> bool;
    fn is_standalone_open(&self, open_delimiter: &str) -> bool;
}

impl ReaderStringOps for str {
    fn trim_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> &str {
        let trimmed = self.trim();
        if trimmed.is_standalone(open_delimiter, close_delimiter) {
            trimmed
        } else {
            self
        }
    }

    fn is_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> bool {
        return if !self.is_standalone_open(open_delimiter) {
            false
        } else {
            let odl = open_delimiter.len() + 1;
            let cdl = close_delimiter.len();
            let mut after: usize = odl;
            let mut tail = &self[odl..];
            while let Some(idx) = tail.find(close_delimiter) {
                after += idx + cdl;
                tail = &tail[idx + cdl..];
                if !tail.is_empty() {
                    if !tail.is_standalone_open(open_delimiter) {
                        return false
                    }
                    after += odl;
                    tail = &tail[odl..];
                }
            }
            after == self.len()
        }
    }

    fn is_standalone_open(&self, open_delimiter: &str) -> bool {
        self.starts_with(open_delimiter)
            && open_delimiter.len() < self.len()
            && STRIPPABLE_SIGIL.contains(&self[open_delimiter.len()..open_delimiter.len() + 1])
    }
}


static STRIPPABLE_SIGIL: &'static str = "#^/>=!$<";


#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn standalone_single() {
        let s = String::from("   {{#a}}{{/a}}  ");
        let p = s[..].trim_standalone("{{", "}}");
        assert_eq!(p, s.trim())
    }
 
    #[test]
    fn standalone_multi() {
        let s = String::from("   {{#a}}{{^x}}{{/x}}{{/a}}  ");
        let p = s[..].trim_standalone("{{", "}}");
        assert_eq!(p, s.trim())
    }
 
    #[test]
    fn not_standalone_multi() {
        let s = String::from("   {{#a}}{{^b}}{{x}}{{/b}}{{/a}}  ");
        let p = s[..].trim_standalone("{{", "}}");
        assert_eq!(p, s)
    }
 
     #[test]
    fn setup_reader() {
        let input = String::from(" 123456 ");
        let expected = input.clone();
        let reader = super::Reader::new(input);

        assert_eq!(reader.read_until("{{"), false);
        assert_eq!(reader.fetch_text(), expected);
    }
}
