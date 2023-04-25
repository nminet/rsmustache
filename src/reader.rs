use std::{ops::Add};

#[derive(Clone)]
pub(crate) struct Reader<'a> {
    input: &'a str,
    open_delimiter: &'a str,
    close_delimiter: &'a str,
    pos: usize,
    after_standalone: usize,
    before_close: usize
}

impl<'a> Reader<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Reader { 
            input,
            open_delimiter: "{{",
            close_delimiter: "}}",
            pos: 0,
            after_standalone: 0,
            before_close: 0
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
        self.after_standalone = if after_standalone > 0 {
            self.pos + after_standalone
        } else {
            0
        };
        self.pos += after_text;
        Token::text(&text)
    }

    fn read_tag(&mut self, tail: &'a str) -> Token<'a> {
        if let Some((text, after_tag)) = tail.span_tag(&self.open_delimiter, &self.close_delimiter) {
            if text.starts_with("/") {
                self.before_close = self.pos
            }
            self.pos = if self.after_standalone > 0 && !self.input[self.pos + after_tag..].starts_with(&self.open_delimiter) {
                self.after_standalone
            } else {
                self.pos + after_tag
            };
            Token::tag(text)
        } else {
            self.pos = self.input.len();
            Token::error("missing close delimiter")
        }
    }

    pub(crate) fn set_delimiters<'s: 'a>(&mut self, od: &'s str, cd: &'s str) {
        self.open_delimiter = od;
        self.close_delimiter = cd
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
    Partial(&'a str, bool, bool),
    Block(&'a str),
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
                '>' => Token::partial(text.trim_sigil(), false),
                '<' => Token::partial(text.trim_sigil(), true),
                '$' => Token::block(text.trim_sigil()),
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

    fn partial(text: &str, is_parent: bool) -> Token {
        match text.chars().nth(0) {
            Some('*') => Token::tag_with_label(|t| Token::Partial(t, true, is_parent), text.trim_sigil()),
            Some(_) => Token::tag_with_label(|t| Token::Partial(t, false, is_parent), text),
            None => Token::error("missing tag name")
        }
    }

    fn block(text: &str) -> Token {
        Token::tag_with_label(Token::Block, text)
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
    fn is_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> bool;
    fn is_standalone_open(&self, open_delimiter: &str) -> bool;
    fn trim_sigil(&self) -> &str;
}

impl ReaderStringOps for str {
    fn span_text(&self, open_delimiter: &str, close_delimiter: &str) -> (&str, usize, usize) {
        let after_text = self.find(open_delimiter).unwrap_or(self.len());
        let mut end_of_text = after_text;
        let mut after_standalone = 0;
        if let Some(eol_in_text) = self[..after_text].rfind("\n") {
            if self[eol_in_text + 1..after_text].chars().all(|c: char| c.is_ascii_whitespace()) {
                let after_next_eol = if let Some(p) = self[eol_in_text + 1..].find("\n") {
                    eol_in_text + 1 + p + 1
                } else {
                    self.len()
                };
                if self[eol_in_text + 1..after_next_eol].trim().is_standalone(open_delimiter, close_delimiter) {
                    // discard whitespace belonging to a line of standalone tags
                    end_of_text = eol_in_text + 1;
                    after_standalone = after_next_eol;
                }
            }
        };
        (&self[..end_of_text], after_text, after_standalone)
    }

    fn span_tag(&self, open_delimiter: &str, close_delimiter: &str) -> Option<(&str, usize)> {
        if let Some(c) = self.chars().nth(open_delimiter.len()) {
            if let Some(p) = match c {
                '{' => self.find(&String::from("}").add(close_delimiter)),
                '=' => self.find(&String::from("=").add(close_delimiter)),
                _ => self.find(close_delimiter)
            } {
                let odl = open_delimiter.len();
                let cdl = if c == '{' || c== '=' {
                    close_delimiter.len() + 1
                } else {
                    close_delimiter.len()
                };
                Some((&self[odl..p].trim(), p + cdl))
            } else {
                // close delimiter not found
                None
            }
        } else {
            // no text after open delimiter
            None
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
            && STRIPPABLE_SIGILS.contains(&self[open_delimiter.len()..].trim()[0..1])
    }

    fn trim_sigil(&self) -> &str {
        self[1..].trim_start()
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
    fn partial_tag_with_dynamic_name() {
        expect_sequence(
            "{{>*parent}}",
            vec![
                Token::Partial("parent", true, false)
            ]
        )
    }

    #[test]
    fn parent_tag_with_dynamic_name() {
        expect_sequence(
            "{{<*parent}}",
            vec![
                Token::Partial("parent", true, true)
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
