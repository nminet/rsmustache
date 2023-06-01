use std::cmp::{min, max};

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
        let starts_new_line = self.pos == 0 || &self.input[self.pos - 1.. self.pos] == "\n";
        let (text, after_text, after_standalone) = tail.span_text(&self.open_delimiter, &self.close_delimiter);
        self.after_standalone = self.pos + after_standalone;
        self.pos += after_text;
        Token::text(&text, starts_new_line)
    }

    fn read_tag(&mut self, tail: &'a str) -> Token<'a> {
        if let Some((text, after_tag)) = tail.span_tag(&self.open_delimiter, &self.close_delimiter) {
            let start_of_line = if let Some(p) = self.input[..self.pos].rfind('\n') {
                p + 1
            } else {
                0
            };
            let before_tag = if self.pos < self.after_standalone {
                if let Some(p) = self.input[..self.pos].rfind(self.close_delimiter) {
                    max(start_of_line, p + self.close_delimiter.len())
                } else {
                    self.pos
                }
            } else {
                self.pos
            };
            let starts_new_line = start_of_line == self.pos;
            let indent = if self.input.is_indent(start_of_line, self.pos) {
                &self.input[start_of_line..self.pos]
            } else {
                ""
            };
            self.pos += after_tag;
            if self.pos < self.after_standalone {
                self.pos = match self.input[self.pos..self.after_standalone].find(self.open_delimiter) {
                    Some(p) if self.pos + p < self.after_standalone => self.pos + p,
                    _ => self.after_standalone
                }
            }
            Token::tag(text, indent, starts_new_line, before_tag, self.pos)
        } else {
            self.pos = self.input.len();
            Token::error("missing close delimiter")
        }
    }

    pub(crate) fn set_delimiters<'s: 'a>(&mut self, od: &'s str, cd: &'s str) {
        self.open_delimiter = od;
        self.close_delimiter = cd;
        self.after_standalone = self.pos + self.input[self.pos..].span_standalone(od, cd);
    }
}



 #[derive(PartialEq, Debug)]
pub(crate) enum Token<'a> {
    Text(&'a str, bool),
    Value(&'a str, bool, bool),
    Section(&'a str, usize),
    InvertedSection(&'a str),
    Block(&'a str),
    EndSection(&'a str, usize),
    Partial(&'a str, bool, &'a str),
    Parent(&'a str, bool, &'a str),
    Comment(&'a str),
    Delimiters(&'a str, &'a str),
    Error(String)
}

impl<'a> Token<'a> {
    fn text(text: &str, starts_new_line: bool) -> Token {
        Token::Text(text, starts_new_line)
    }
    
    fn tag(
        text: &'a str, indent: &'a str, starts_new_line: bool, before_tag: usize, after_tag: usize
    ) -> Token<'a> {
        if let Some(s) = text.chars().nth(0) {
            match s {
                '#' => Token::section(text.trim_sigil(), after_tag),
                '^' => Token::inverted_section(text.trim_sigil()),
                '$' => Token::block(text.trim_sigil()),
                '<' => Token::parent(text.trim_sigil(), indent),
                '/' => Token::end_section(text.trim_sigil(), before_tag),
                '>' => Token::partial(text.trim_sigil(), indent),
                '=' => Token::delimiters(text.trim_sigil()),
                '!' => Token::Comment(text.trim_sigil()),
                '&' | '{' => Token::value(text.trim_sigil(), false, starts_new_line),
                _ => Token::value(text, true, starts_new_line)
            }
        } else {
            Token::error("empty open-close pair")
        }
    }

    fn section(text: &str, after_tag: usize) -> Token {
        let tag = match maybe_tag(text) {
            Ok(tag) => tag,
            Err(token) => return token
        };
        Token::Section(tag, after_tag)
    }

    fn inverted_section(text: &str) -> Token {
        let tag = match maybe_tag(text) {
            Ok(tag) => tag,
            Err(token) => return token
        };
        Token::InvertedSection(tag)
    }

    fn block(text: &str) -> Token {
        let tag = match maybe_tag(text) {
            Ok(tag) => tag,
            Err(token) => return token
        };
        Token::Block(tag)
    }

    fn end_section(text: &str, before_tag: usize) -> Token {
        let tag = match maybe_tag(text) {
            Ok(tag) => tag,
            Err(token) => return token
        };
        Token::EndSection(tag, before_tag)
    }

    fn parent(text: &'a str, indent: &'a str) -> Token<'a> {
        let (tag, is_dynamic) = match maybe_dynamic_tag(text) {
            Ok(result) => result,
            Err(token) => return token
        };
        Token::Parent(tag, is_dynamic, indent)
    }

    fn partial(text: &'a str, indent: &'a str) -> Token<'a> {
        let (tag, is_dynamic) = match maybe_dynamic_tag(text) {
            Ok(result) => result,
            Err(token) => return token
        };
        Token::Partial(tag, is_dynamic, indent)
    }

    fn delimiters(text: &str) -> Token {
        let (od, cd) = match maybe_delimiters(text) {
            Ok(result) => result,
            Err(token) => return token
        };
        Token::Delimiters(od, cd)
    }

    fn value(text: &str, escaped: bool, starts_new_line: bool) -> Token {
        let tag = match maybe_tag(text) {
            Ok(tag) => tag,
            Err(token) => return token
        };
        Token::Value(tag, escaped, starts_new_line)
    }

    fn error(text: &str) -> Token {
        Token::Error(text.to_string())
    }
}

fn maybe_tag(text: &str) -> Result<&str, Token> {
    if text == "." {
        Ok(text)
    } else if text.len() == 0 {
        Err(Token::error("missing tag"))
    } else if text.starts_with('.') || text.ends_with('.')|| text.contains(' ') || text.contains("..") {
        Err(Token::error("invalid tag"))
    } else {
        Ok(text)
    }
}

fn maybe_dynamic_tag(text: &str) -> Result<(&str, bool), Token> {
    let is_dynamic = text.starts_with("*");
    let text = if is_dynamic {
        text[1..].trim_start()
    } else {
        text
    };
    let tag = maybe_tag(text)?;
    Ok((tag, is_dynamic))
}

fn maybe_delimiters(text: &str) -> Result<(&str, &str), Token> {
    let words = text.split_ascii_whitespace().collect::<Vec<_>>();
    if text.find("=").is_some() || words.len() != 2 {
        Err(Token::error("invalid delimiters tag"))
    } else {
        Ok((words[0], words[1]))
    }
}

trait ReaderStringOps {
    fn span_text(&self, open_delimiter: &str, close_delimiter: &str) -> (&str, usize, usize);
    fn span_tag(&self, open_delimiter: &str, close_delimiter: &str) -> Option<(&str, usize)>;
    fn span_standalone(&self, open_delimiter: &str, close_delimiter: &str) -> usize;
    fn is_standalone_open(&self, open_delimiter: &str) -> bool;
    fn trim_sigil(&self) -> &str;
    fn is_space(&self, start: usize, len: usize) -> bool;
    fn is_indent(&self, start: usize, len: usize) -> bool;
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
            if !self.is_space(pos, od) {
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
                    if !self.is_space(cd, cd + min(od, eol)) {
                        break
                    };
                    if eol < od {
                        after = cd + eol + 1;
                        if !self.is_space(after, cd + od) {
                            break
                        };
                        pos = after
                    };
                    cd + od
                },
                (Some(od), None) => {
                    if !self.is_space(cd, cd + od) {
                        break
                    };
                    cd + od
                }
                (None, Some(eol)) => {
                    if !self.is_space(cd, cd + eol) {
                        break
                    };
                    after = cd + eol + 1;
                    break
                }
                _ => {
                    if self.is_space(cd, self.len()) {
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

    fn is_space(&self, start: usize, after: usize) -> bool {
        self[start..after].trim().is_empty() && self[start..after].find('\n').is_none()
    }

    fn is_indent(&self, start: usize, after: usize) -> bool {
        after == start || self[start..after].chars().all(
            |c| c == ' ' || c == '\t'
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
                Token::text(" 123456 ", true)
            ]
        );
    }

    #[test]
    fn standalone_single_is_trimed() {
        expect_sequence(
            "x\n   {{/a}}  \ny",
            vec![
                Token::Text("x\n", true),
                Token::EndSection("a", 5),
                Token::Text("y", true)
            ]
        )
    }
 
    #[test]
    fn standalone_multi_is_trimed() {
        expect_sequence(
            "x\n   {{ # a }}{{^x}}{{/x}}{{ / a }}  \ny",
            vec![
                Token::Text("x\n", true),
                Token::Section("a", 14),
                Token::InvertedSection("x"),
                Token::EndSection("x", 20),
                Token::EndSection("a", 26),
                Token::Text("y", true)
            ]
        )
    }

    #[test]
    fn not_standalone_multi_is_not_trimed() {
        expect_sequence(
            "x\n   {{ #a }}{{^b }}{{{x}}}{{ /b}}{{/a}}  \ny",
            vec![
                Token::Text("x\n   ", true),
                Token::Section("a", 13),
                Token::InvertedSection("b"),
                Token::Value("x", false, false),
                Token::EndSection("b", 27),
                Token::EndSection("a", 34),
                Token::Text("  \ny", false)
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
                Token::error("missing tag")
            ]
        )
    }
       
    #[test]
    fn value_with_escape() {
        expect_sequence(
            "{{ v }}",
            vec![
                Token::Value("v", true, true)
            ]
        )
    }
 
    #[test]
    fn value_without_escape() {
        expect_sequence(
            "{{{ v }}}",
            vec![
                Token::Value("v", false, true)
            ]
        )
    }

    #[test]
    fn inner_section() {
        expect_sequence(
            "{{#a}}\n{{#b}}\n{{#c}}\n\n",
            vec![
                Token::Section("a", 7),
                Token::Section("b", 14),
                Token::Section("c", 21),
                Token::Text("\n", true)
            ]
        )
    }

    #[test]
    fn repeated_newline() {
        expect_sequence(
            "{{#a}} \n \n {{#b}}",
            vec![
                Token::Section("a", 8),
                Token::Text(" \n", true),
                Token::Section("b", 17),
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
