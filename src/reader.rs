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
        self.is_standalone_open(open_delimiter) && {
            let odl = open_delimiter.len() + 1;
            let cdl = close_delimiter.len();
            let mut after: usize = odl;
            let mut tail = &self[odl..];
            while let Some(idx) = tail.find(close_delimiter) {
                after += idx + cdl;
                tail = &tail[idx + cdl..];
                if tail.is_empty() || !tail.is_standalone_open(open_delimiter) {
                    break
                }
                after += odl;
                tail = &tail[odl..];
            }
            after == self.len()
        }
    }

    fn is_standalone_open(&self, open_delimiter: &str) -> bool {
        static STRIPPABLE_SIGILS: &str = "#^/>=!$<";
        self.starts_with(open_delimiter)
            && open_delimiter.len() < self.len()
            && STRIPPABLE_SIGILS.contains(&self[open_delimiter.len()..open_delimiter.len() + 1])
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn standalone_single_is_trimed() {
        let s = String::from("   {{/a}}  ");
        let r = s.trim_standalone("{{", "}}");
        assert_eq!(r, s.trim())
    }
 
    #[test]
    fn standalone_multi_is_trimed() {
        let s = String::from("   {{#a}}{{^x}}{{/x}}{{/a}}  ");
        let r = s.trim_standalone("{{", "}}");
        assert_eq!(r, s.trim())
    }
 
    #[test]
    fn not_standalone_single_is_not_trimed() {
        let s = String::from("   {{x}}");
        let r = s.trim_standalone("{{", "}}");
        assert_eq!(r, s)
    }
 
    #[test]
    fn not_standalone_multi_is_not_trimed() {
        let s = String::from("   {{#a}}{{^b}}{{x}}{{/b}}{{/a}}  ");
        let r = s.trim_standalone("{{", "}}");
        assert_eq!(r, s)
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
