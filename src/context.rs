use std::collections::VecDeque;

/// Adapter to render an external type into a Mustache template.
/// 
/// The trait is used by the rendering engine to obtain context data and navigate
/// in the implied tree as directed by the rendered template.
/// 
/// To avoid unnecessary memory copies and support dynamic generation of data,
/// the trait assumes the implementation manages the lifecycle of the underlying
/// data, providing a view on internal data structures. To support this, functions
/// in the trait return context as
/// ```text
/// type ContextRef<'a> = &'a dyn Context<'a>
/// ```
///
/// 
/// The Mustache template system - for the purpose of rendering - assume the context
/// is one of
/// - a named text value
/// - a mapping of string to context
/// - a list of contexts
/// - a boolean value
/// 
/// Any context can be used as a section.
/// ```text
/// {{#x}}
/// this is rendered if x is not falsy or a non-empty list
/// {{/x}}
/// {{^x}}
/// this is rendered if x is falsy or an empty list
/// {{/x}}
/// ```
/// If **x** is a list, literal text in the section is emitted for each item.
/// 
/// Mustache requires null and false to be falsy. Boolean conversion of
/// other values are left for implementation to decide (the **is_falsy**
/// entry in the trait allows controlling this).
/// 
/// 
/// See Implementation section below for examples.

pub trait Context<'a> {
    /// Get a child context from a mapping, or None if the context is not a mapping.
    fn child(&'a self, name: &str) -> Option<ContextRef<'a>>;

    /// Get children contexts from a list, or None if the context is not a list.
    fn children(&'a self) -> Option<Vec<ContextRef<'a>>>;

    /// Get the rendered text for the context.
    fn value(&self) -> String;

    /// Indicate if the context is falsy.
    fn is_falsy(&self) -> bool;
}

pub type ContextRef<'a> = &'a dyn Context<'a>;


#[derive(Clone)]
struct Frame<'a> {
    // VecDeque to avoid quadratic complexity when removing from start.
    contexts: VecDeque<ContextRef<'a>>,
    is_sequence: bool,
    is_dotted: bool
}

impl<'a> Frame<'a> {
    fn new(contexts: Vec<ContextRef<'a>>, is_sequence: bool, is_dotted: bool) -> Self {
        let contexts = VecDeque::from(contexts);
        Frame {
            contexts,
            is_sequence,
            is_dotted
        }
    }

    fn current(&self) -> Option<&ContextRef<'a>> {
        self.contexts.front()
    }

    fn next(&mut self) -> bool {
        self.contexts.pop_front();
        !self.contexts.is_empty()
    }
}


#[derive(Clone)]
pub(crate) struct Stack<'a> {
    frames: Vec<Frame<'a>>,
    backtrack_depth: usize
}

impl<'a> Stack<'a> {
    pub(crate) fn new(root: ContextRef<'a>) -> Self {
        let frame = Frame::new(vec![root], false, false);
        let frames = vec![frame];
        Stack { 
            frames,
            backtrack_depth: 0
        }
    }

    fn backtracking(&self) -> Self {
        let len = self.frames.len() - 1;
        let frames = Vec::from(&self.frames[..len]);
        Stack {
            frames,
            backtrack_depth: 1
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.frames.len()
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        self.frames.truncate(len);
    }

    fn merge(&mut self, other: Stack<'a>) {
        let unchanged = self.frames.len() - other.backtrack_depth;
        self.frames.extend_from_slice(&other.frames[unchanged..]);
    }

    fn push_dotted(&mut self, name: &str, is_dotted: bool) -> bool {
        if name == "." {
            if let Some(children) = self.children() {
                let frame = Frame::new(children, true, is_dotted);
                self.frames.push(frame);
            };
            true

        } else if let Some(idx) = name.find(".") {
            let (head, tail) = name.split_at(idx);
            let len = self.len();
            if self.push_dotted(head, true) && self.push_dotted(&tail[1..], true) {
                true
            } else {
                self.truncate(len);
                false
            }

        } else if let Some(context) = self.child(name) {
            let (contexts, is_sequence) = if let Some(children) = context.children() {
                (children, true)
            } else {
                (vec![context], false)
            };
            let frame = Frame::new(contexts, is_sequence, is_dotted);
            self.frames.push(frame);
            true
        
        } else if is_dotted && self.top().is_dotted {
            // no backtracking while processing dotted name
            false

        } else if self.len() == 1 {
            // nowhere to backtrack
            false

        } else {
            let mut resolved: bool;
            let mut ts = self.backtracking();
            loop {
                resolved = ts.push(name);
                if resolved || !ts.down() {
                    break;
                }
            }
            if resolved {
                self.merge(ts);
            }
            resolved
        }
    }

    pub(crate) fn push(&mut self, name: &str) -> bool {
        self.push_dotted(name, false)
    }

    pub(crate) fn next(&mut self) -> bool {
        let mut frame = self.frames.pop().unwrap();
        let more = frame.next();
        self.frames.push(frame);
        more
    }

    fn down(&mut self) -> bool {
        let len = self.frames.len();
        if len > 1 {
            let mut next_len = len - 1;
            while self.frames[next_len - 1].is_dotted {
                next_len -= 1;
            }
            self.frames.truncate(next_len);
            if self.backtrack_depth > 0 {
                self.backtrack_depth += len - next_len;
            }
            true
        } else {
            false
        }
    }

    fn top(&self) -> &Frame<'a> {
        self.frames.last().unwrap()
    }

    pub(crate) fn top_is_sequence(&self) -> bool {
        self.top().is_sequence
    }

    pub(crate) fn current(&self) -> Option<&ContextRef<'a>> {
        self.top().current()
    }

    fn child(&self, name: &str)  -> Option<ContextRef<'a>> {
        self.current()?.child(name)
    }

    fn children(&self)  -> Option<Vec<ContextRef<'a>>> {
        self.current()?.children()
    }

    fn value(&self) -> String {
        match self.current() {
            Some(context) => context.value(),
            _ => "".to_owned()
        }
    }

    pub(crate) fn is_falsy(&self) -> bool {
        self.current().map_or(
            true,
             |context| context.is_falsy()
        )
    }

    pub(crate) fn get(&mut self, name: &str) -> Option<String> {
        if name == "." {
            Some(self.value())
        } else {
            let len = self.len();
            if self.push(name) {
                let result = self.value();
                self.truncate(len);
                Some(result)
            } else {
                None
            }
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::JsonValue;

    #[test]
    fn basic_access() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert_eq!(stack.get("name").unwrap(), "John Doe");
        assert!(!stack.push("xxx"));
        assert!(stack.push("phones"));
        assert_eq!(stack.get("prefix").unwrap(), "+44");
        assert_eq!(stack.get("extension").unwrap(), "1234567");
        assert!(stack.get("aaa").is_none());
        assert!(stack.next());
        assert_eq!(stack.get("prefix").unwrap(), "+44");
        assert_eq!(stack.get("extension").unwrap(), "2345678");
        assert!(!stack.next());
        assert!(stack.down());
        assert_eq!(stack.get("age").unwrap(), "43");
        assert!(!stack.down());
    }

    #[test]
    fn normal_backtrack() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones");
        assert!(stack.push("stuff"));
        assert_eq!(stack.value(), "item1");
        assert!(stack.next());
        assert_eq!(stack.value(), "item2");
        assert!(!stack.next());
        assert!(stack.down());
        assert_eq!(stack.get("extension").unwrap(), "1234567");
    }

    #[test]
    fn dotted_from_top() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert!(stack.push("obj.part2"));
        assert_eq!(stack.value(), "yyy");
    }

    #[test]
    fn dotted_after_backtrack() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones");
        assert!(stack.push("obj.part2"));
        assert_eq!(stack.value(), "yyy");
    }

    #[test]
    fn backtrack_after_dotted() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones");
        assert!(stack.push("obj.part2"));
        assert!(stack.push("age"));
    }

    #[test]
    fn broken_chain() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert!(!stack.push("obj.part1.part2"));
    }

    #[test]
    fn failed_dotted_resolution_leaves_stack_unchanged() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("name");
        assert!(!stack.push("obj.part1.part3"));
        assert_eq!(stack.value(), "John Doe");
    }

    fn json1() -> JsonValue {
        let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                {
                    "prefix": "+44",
                    "extension": "1234567"
                },
                {
                    "prefix": "+44",
                    "extension": "2345678"
                }
            ],
            "stuff": [
                "item1",
                "item2"
            ],
            "obj": {
                "part1": "xxx",
                "part2": "yyy"
            }
        }"#;
        serde_json::from_str::<JsonValue>(data).unwrap()
    }
}
