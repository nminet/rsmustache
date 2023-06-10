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
/// If **x** is a list, the literal text is emitted once for each item.
/// 
/// Mustache requires null and false to be falsy. Boolean conversion of
/// other values are left for implementation to decide (the **is_falsy**
/// entry in the trait allows controlling this).
/// 
/// 
/// See Implementors section below for examples.

pub trait Context<'a> {
    /// Get a child context from a mapping, or None if the context is not a mapping.
    /// 
    /// When the name is in section position, the section text for the current render
    /// is the [start..end] slice in the source that produced the template.
    fn child<'b>(&'a self, name: &str, section: Option<(usize, usize)>)
        -> Option<ContextRef<'b>>
        where 'a: 'b;

    /// Get children contexts from a list, or None if the context is not a list.
    fn children<'b>(&'a self) -> Option<Vec<ContextRef<'b>>>
    where 'a: 'b;

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
    is_sequence: bool
}

impl<'a> Frame<'a> {
    fn new(contexts: Vec<ContextRef<'a>>, is_sequence: bool) -> Self {
        let contexts = VecDeque::from(contexts);
        Frame {
            contexts,
            is_sequence
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
    frames: Vec<Frame<'a>>
}

impl<'a> Stack<'a> {
    pub(crate) fn new(root: ContextRef<'a>) -> Self {
        let frame = Frame::new(vec![root], false);
        let frames = vec![frame];
        Stack { 
            frames
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.frames.len()
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        self.frames.truncate(len);
    }

    pub(crate) fn push(&mut self, name: &str, location: Option<(usize, usize)>) -> bool {
        self.push_internal(name, self.len() - 1, false, location)
    }

    fn push_internal(
        &mut self, name: &str, mut idx: usize, is_dotted: bool, location: Option<(usize, usize)>
    ) -> bool {
        if name == "." {
            if let Some(children) = self.children(idx) {
                let frame = Frame::new(children, true);
                self.frames.push(frame);
            };
            true

        } else if let Some(pos) = name.find(".") {
            let (head, tail) = name.split_at(pos);
            let len = self.len();
            if self.push_internal(head, idx, is_dotted, location) && self.push_internal(&tail[1..], idx, true, location) {
                true
            } else {
                self.truncate(len);
                false
            }

        } else if let Some(context) = self.child(idx, name, is_dotted, location) {
            let (contexts, is_sequence) = if let Some(children) = context.children() {
                (children, true)
            } else {
                (vec![context], false)
            };
            let frame = Frame::new(contexts, is_sequence);
            if is_dotted {
                self.truncate(self.len() - 1);
            }
            self.frames.push(frame);
            true
        
        } else if is_dotted {
            // no backtracking while processing dotted name
            false

        } else {
            let mut resolved = false;
            loop {
                if resolved || idx == 0 {
                    break;
                }
                idx -= 1;
                resolved = self.push_internal(name, idx, false, location);
            };
            resolved
        }
    }

    fn child(
        &self, mut idx: usize, name: &str, is_dotted: bool, location: Option<(usize, usize)>
    ) -> Option<ContextRef<'a>> {
        if is_dotted {
            idx += 1;
        }
        self.frames[idx].current()?.child(name, location)
    }

    fn children(&self, idx: usize)  -> Option<Vec<ContextRef<'a>>> {
        self.frames[idx].current()?.children()
    }

    pub(crate) fn in_sequence(&self) -> bool {
        self.frames[self.frames.len() - 1].is_sequence
    }

    pub(crate) fn current(&self) -> Option<&ContextRef<'a>> {
        self.frames[self.frames.len() - 1].current()
    }

    pub(crate) fn is_falsy(&self) -> bool {
        self.current().map_or(
            true,
             |context| context.is_falsy()
        )
    }

    pub(crate) fn next(&mut self) -> bool {
        let mut frame = self.frames.pop().unwrap();
        let more = frame.next();
        self.frames.push(frame);
        more
    }

    pub(crate) fn get(&mut self, name: &str) -> Option<String> {
        if name == "." {
            Some(self.value())
        } else {
            let len = self.len();
            if self.push(name, None) {
                let result = self.value();
                self.truncate(len);
                Some(result)
            } else {
                None
            }
        }
    }

    fn value(&self) -> String {
        match self.current() {
            Some(context) => context.value(),
            _ => "".to_owned()
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
        assert!(!stack.push("xxx", None));
        assert!(stack.push("phones", None));
        assert_eq!(stack.get("prefix").unwrap(), "+44");
        assert_eq!(stack.get("extension").unwrap(), "1234567");
        assert!(stack.get("aaa").is_none());
        assert!(stack.next());
        assert_eq!(stack.get("prefix").unwrap(), "+44");
        assert_eq!(stack.get("extension").unwrap(), "2345678");
        assert!(!stack.next());
        assert_eq!(stack.get("age").unwrap(), "43");
    }

    #[test]
    fn normal_backtrack() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones", None);
        assert!(stack.push("stuff", None));
        assert_eq!(stack.value(), "item1");
        assert!(stack.next());
        assert_eq!(stack.value(), "item2");
        assert!(!stack.next());
        assert_eq!(stack.get("extension").unwrap(), "1234567");
    }

    #[test]
    fn dotted_from_top() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert!(stack.push("obj.part2", None));
        assert_eq!(stack.value(), "yyy");
    }

    #[test]
    fn dotted_after_backtrack() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones", None);
        assert!(stack.push("obj.part2", None));
        assert_eq!(stack.value(), "yyy");
    }

    #[test]
    fn backtrack_after_dotted() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones", None);
        assert!(stack.push("obj.part2", None));
        assert!(stack.push("age", None));
    }

    #[test]
    fn broken_chain() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert!(!stack.push("obj.part1.part2", None));
    }

    #[test]
    fn failed_dotted_resolution_leaves_stack_unchanged() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("name", None);
        assert!(!stack.push("obj.part1.part3", None));
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
