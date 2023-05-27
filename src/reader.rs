use std::cmp::min;

#[derive(Clone)]
pub(crate) struct Reader<'a> {
    input: &'a str,
    open_delimiter: &'a str,
    close_delimiter: &'a str,
    pos: usize,
    after_standalone: usize
}

impl<'a> Reader<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        let open_delimiter = "{{";
        let close_delimiter = "}}";
        let after_standalone = input.span_standalone(open_delimiter, close_delimiter);
        let pos = if after_standalone > 0 {
            input.find(open_delimiter).unwrap()
        } else {
            0
        };
        Reader { 
            input,
            open_delimiter,
            close_delimiter,
            pos,
            after_standalone,
        }
    }

    pub(crate) fn pop_front(&mut self) -> Option<Token<'a>> {
        if self.pos == self.input.len() {
            None
        } else {
            let tail = &self.input[self.pos..];
            let token = if tail.starts_with(&self.open_delimiter) {
                self.read_tag(tail)
            } else {
                self.read_text(tail)
            };
            Some(token)
        }
    }

    fn read_text(&mut self, tail: &'a str) -> Token<'a> {
        let (text, after_text, after_standalone) = tail.span_text(&self.open_delimiter, &self.close_delimiter);
        self.after_standalone = self.pos + after_standalone;
        self.pos += after_text;
        Token::text(&text)
    }

    fn read_tag(&mut self, tail: &'a str) -> Token<'a> {
        if let Some((text, after_tag)) = tail.span_tag(&self.open_delimiter, &self.close_delimiter) {
            self.pos += after_tag;
            if self.pos < self.after_standalone {
                self.pos = match self.input[self.pos..self.after_standalone].find(self.open_delimiter) {
                    Some(p) if self.pos + p < self.after_standalone => self.pos + p,
                    _ => self.after_standalone
                }
            }
            Token::tag(text)
        } else {
            self.pos = self.input.len();
            Token::error("missing close delimiter")
        }
    }

    pub(crate) fn set_delimiters<'s: 'a>(&mut self, od: &'s str, cd: &'s str) {
        self.open_delimiter = od;
        self.close_delimiter = cd;
        self.after_standalone = self.input[self.pos..].span_standalone(od, cd);
    }
}



 #[derive(PartialEq, Debug)]
pub(crate) enum Token<'a> {
    Text(&'a str),
    Section(&'a str),
    InvertedSection(&'a str),
    EndSection(&'a str),
    Value(&'a str, bool),
    Comment(&'a str),
    Delimiters(&'a str, &'a str),
    Error(String)
}

impl<'a> Token<'a> {
    fn text(text: &str) -> Token {
        Token::Text(text)
    }
    
    fn tag(text: &str) -> Token {
        if let Some(s) = text.chars().nth(0) {
            match s {
                '#' => Token::section(text.trim_sigil()),
                '^' => Token::inverted_section(text.trim_sigil()),           
                '/' => Token::end_section(text.trim_sigil()),           
                '=' => Token::delimiters(text.trim_sigil()),
                '!' => Token::Comment(text.trim_sigil()),
                '&' | '{' => Token::value(text.trim_sigil(), false),
                _ => Token::value(text, true)
            }
        } else {
            Token::error("empty open-close pair")
        }
    }

    fn section(text: &str) -> Token {
        Token::tag_with_label(Token::Section, text)
    }

    fn inverted_section(text: &str) -> Token {
        Token::tag_with_label(Token::InvertedSection, text)
    }

    fn end_section(text: &str) -> Token {
        Token::tag_with_label(Token::EndSection, text)
    }

    fn delimiters(text: &str) -> Token {
        let words: Vec<&str> = text.split(" ").filter(|s| !s.is_empty()).collect();
        if words.len() == 2 && words[0].find('=') == None && words[1].find('=') == None {
            Token::Delimiters(words[0], words[1])
        } else {
            Token::error("invalid delimiters tag")
        }
    }

    fn value(text: &str, escaped: bool) -> Token {
        Token::tag_with_label(|t| Token::Value(t, escaped), text)
    }

    fn error(text: &str) -> Token {
        Token::Error(text.to_string())
    }

    fn tag_with_label<F: Fn(&'a str) -> Token<'a>>(make: F, text: &'a str) -> Token {
        if text.len() > 0 {
            make(text)
        } else {
            Token::error("missing name")
        }
    }

}


trait ReaderStringOps {
    fn span_text(&self, open_delimiter: &str, close_delimiter: &str) -> (&str, usize, usize);
    fn span_tag(&self, open_delimiter: &str, close_delimiter: &str) -> Option<(&str, usize)>;
    fn span_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> usize;
    fn is_standalone_open(&self, open_delimiter: &str) -> bool;
    fn trim_sigil(&self) -> &str;
    fn is_blank(&self, start: usize, len: usize) -> bool;
}

impl ReaderStringOps for str {
    // return
    // - the position after the current text
    // - the position after the current text not part of a sequence of standalone tags
    fn span_text(&self, open_delimiter: &str, close_delimiter: &str) -> (&str, usize, usize) {
        let after_text = self.find(open_delimiter).unwrap_or(self.len());
        let mut end_of_text = after_text;
        let mut after_standalone = after_text;
        if let Some(eol_in_text) = self[..after_text].rfind("\n") {
            let p = self[eol_in_text + 1..].span_standalone(open_delimiter, close_delimiter);
            if p > 0 {
                end_of_text = eol_in_text + 1;
                after_standalone = end_of_text + p;
            }
        };
        (&self[..end_of_text], after_text, after_standalone)
    }

    // return the tag starting at beginning of the string and the position after the tag
    // return None if the string does not start with a tag
    fn span_tag(&self, open_delimiter: &str, close_delimiter: &str) -> Option<(&str, usize)> {
        let odl = open_delimiter.len();
        if let Some(c) = self.chars().nth(odl) {
            let (cd, cdl) = match c {
                '{' => (format!("{}{}", '}', close_delimiter), close_delimiter.len() + 1),
                '=' => (format!("{}{}", '=', close_delimiter), close_delimiter.len() + 1),
                _ => (close_delimiter.to_string(), close_delimiter.len())

            };
            if let Some(p) = self[odl..].find(&cd) {
                Some((&self[odl..odl + p].trim(), odl + p + cdl))
            } else {
                None
            }
        } else {
            // no text after open delimiter
            None
        }
    }

    // return the position following a sequence of standalone tags
    // return 0 if the string does not start with a sequence of standalone tags
    fn span_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> usize {
        let mut pos: usize = 0;
        let mut after: usize = 0;
        let mut od = match self.find(open_delimiter) {
            Some(p)  => p,
            _ => return 0
        };
        let mut cd: usize;
        let odl = open_delimiter.len();
        let cdl = close_delimiter.len();
        loop {
            if !self.is_blank(pos, od) {
                break
            };
            if !self[od..].is_standalone_open(open_delimiter) {
                break;
            }
            cd = match self[od + odl..].find(close_delimiter) {
                Some(p) => od + odl + p + cdl,
                _ => break
            };
            pos = cd;
            let x0 = self[cd..].find(open_delimiter);
            let x1 = self[cd..].find('\n');
            od = match (x0, x1)  {
                (Some(od), Some(eol)) => {
                    if !self.is_blank(cd, cd + min(od, eol)) {
                        break
                    };
                    if eol < od {
                        after = cd + eol + 1;
                        if !self.is_blank(after, cd + od) {
                            break
                        };
                    };
                    cd + od
                },
                (Some(od), None) => {
                    if !self.is_blank(cd, cd + od) {
                        break
                    };
                    cd + od
                }
                (None, Some(eol)) => {
                    if !self.is_blank(cd, cd + eol) {
                        break
                    };
                    after = cd + eol + 1;
                    break
                }
                _ => {
                    if self.is_blank(cd, self.len()) {
                        after = self.len();
                    }
                    break
                }
            };
        }
        after
    }

    fn is_standalone_open(&self, open_delimiter: &str) -> bool {
        static STRIPPABLE_SIGILS: &str = "#^/>=!$<";
        self.starts_with(open_delimiter)
            && open_delimiter.len() < self.len()
            && STRIPPABLE_SIGILS.contains(&self[open_delimiter.len()..].trim()[0..1])
    }

    fn trim_sigil(&self) -> &str {
        self[1..].trim_start()
    }

    fn is_blank(&self, start: usize, after: usize) -> bool {
        after == start || self[start..after].chars().all(
            |c| c.is_ascii_whitespace()
        )
    }
}



#[cfg(test)]
mod tests {
    use super::*;
     
    #[test]
    fn text_only() {
        expect_sequence(
            " 123456 ",
            vec![
                Token::text(" 123456 ")
            ]
        );
    }

    #[test]
    fn standalone_single_is_trimed() {
        expect_sequence(
            "x\n   {{/a}}  \ny",
            vec![
                Token::Text("x\n"),
                Token::EndSection("a"),
                Token::Text("y")
            ]
        )
    }
 
    #[test]
    fn standalone_multi_is_trimed() {
        expect_sequence(
            "x\n   {{ # a }}{{^x}}{{/x}}{{ / a }}  \ny",
            vec![
                Token::Text("x\n"),
                Token::Section("a"),
                Token::InvertedSection("x"),
                Token::EndSection("x"),
                Token::EndSection("a"),
                Token::Text("y")
            ]
        )
    }

    #[test]
    fn not_standalone_multi_is_not_trimed() {
        expect_sequence(
            "x\n   {{ #a }}{{^b }}{{{x}}}{{ /b}}{{/a}}  \ny",
            vec![
                Token::Text("x\n   "),
                Token::Section("a"),
                Token::InvertedSection("b"),
                Token::Value("x", false),
                Token::EndSection("b"),
                Token::EndSection("a"),
                Token::Text("  \ny")
            ]
        )
    }

    #[test]
    fn update_delimiters() {
        expect_sequence(
            "{{=| |=}}",
            vec![
                Token::Delimiters("|", "|")
            ]
        )
    }

    #[test]
    fn delimiters_are_trimed() {
        expect_sequence(
            "{{= +++   --- =}}",
            vec![
                Token::Delimiters("+++", "---")
            ]
        )
    }

    #[test]
    fn missing_delimiters_close() {
        expect_sequence(
            "{{= +++   --- }}",
            vec![
                Token::error("missing close delimiter")
            ]
        )
    }

    #[test]
    fn invalid_open_delimiter_value() {
        expect_sequence(
            "{{= |=   | =}}",
            vec![
                Token::error("invalid delimiters tag")
            ]
        )
    }

    #[test]
    fn invalid_close_delimiter_value() {
        expect_sequence(
            "{{= |   =| =}}",
            vec![
                Token::error("invalid delimiters tag")
            ]
        )
    }

    #[test]
    fn value_missing_name() {
        expect_sequence(
            "{{ & }}",
            vec![
                Token::error("missing name")
            ]
        )
    }
       
    #[test]
    fn value_with_escape() {
        expect_sequence(
            "{{ v }}",
            vec![
                Token::Value("v", true)
            ]
        )
    }
 
    #[test]
    fn value_without_escape() {
        expect_sequence(
            "{{{ v }}}",
            vec![
                Token::Value("v", false)
            ]
        )
    }


    #[test]
    fn inner_section() {
        expect_sequence(
            "{{#a}}\n{{#b}}\n{{#c}}\n\n",
            vec![
                Token::Section("a"),
                Token::Section("b"),
                Token::Section("c"),
                Token::Text("\n")
            ]
        )
    }


    fn expect_sequence(input: &str, tokens:Vec<Token<'_>>) {
        let mut reader = Reader::new(input);
        let mut expected = tokens.into_iter();
        loop {
            let token = reader.pop_front();
            assert_eq!(token, expected.next());
            if token == None {
                break;
            }
        }
    }
}
