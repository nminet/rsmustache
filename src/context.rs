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
/// type ContextRef<'a> = &'a dyn Context
/// ```
///
/// 
/// The Mustache template system - for the purpose of rendering - assumes the context
/// is one of
/// - a named text value
/// - a mapping of string to context
/// - an iterator over a sequence of contexts
/// - a boolean value
/// 
/// Any context can be used as a section.
/// ```text
/// {{#x}}
/// this is rendered if x is not falsy or a non-empty sequence
/// {{/x}}
/// {{^x}}
/// this is rendered if x is falsy or an empty sequence
/// {{/x}}
/// ```
/// If **x** is a sequence, the content is rendered once for each item.
/// 
/// Mustache requires null and false to be falsy. Boolean conversion of
/// other values are left for implementation to decide (the **is_falsy**
/// entry in the trait allows controlling this).
/// 
/// 
/// See Implementors section below for examples.
pub trait Context {
    /// Get a child context from a mapping, or None if the context is not a mapping.
    /// 
    /// The section parameter is [Some] value if and only if the request for the name
    /// is in section position. In this case it contains a (start,end) pair such that
    /// the [start..end] slice in the source that produced the template contains
    /// the text of the section.
    fn child(&self, name: &str, section: Option<(usize, usize)>) -> Option<ContextRef<'_>>;

    /// Get an iterator over a sequence of children contexts, or None if the context is not a sequence.
    fn children(&self) -> Option<ContextRefIterator<'_>>;

    /// Get the contents of the context.
    /// 
    /// [ContextValue::Text] is rendered as text.
    /// [ContextValue::Lambda] is compiled and rendered using the current stack
    /// and partials.
    fn value(&self) -> ContextValue;

    /// Indicate if the context is falsy.
    fn is_falsy(&self) -> bool;
}

#[derive(PartialEq, Debug)]
pub enum ContextValue {
    Text(String),
    Lambda(String),
}

pub type ContextRef<'a> = &'a dyn Context;
pub type ContextRefIterator<'a> = Box<dyn Iterator<Item = ContextRef<'a>> + 'a>;

    
struct Frame<'a> {
    current: Option<ContextRef<'a>>,
    iterator: Option<ContextRefIterator<'a>>,
}

impl<'a> Frame<'a> {
    fn new_from_single(context: ContextRef<'a>) -> Self {
        Frame {
            current: Some(context),
            iterator: None
        }
    }

    fn new_from_iterator(mut iterator: ContextRefIterator<'a>) -> Self {
        Frame {
            current: iterator.next(),
            iterator: Some(iterator)
        }
    }

    fn current(&self) -> Option<&ContextRef<'a>> {
        self.current.as_ref()
    }

    fn next(&mut self) -> bool {
        if let Some(mut iterator) = self.iterator.take() {
            self.current = iterator.next();
            self.iterator = Some(iterator);
        } else {
            self.current = None;
        }
        self.current.is_some()
    }
}


pub(crate) struct Stack<'a> {
    frames: Vec<Frame<'a>>,
}

impl<'a> Stack<'a> {
    pub(crate) fn new(root: ContextRef<'a>) -> Self {
        let frame = Frame::new_from_single(root);
        Stack {
            frames: vec![frame]
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
            let current = self.frames[idx].current().copied();
            if let Some(context) = current {
                if let Some(iterator) = context.children() {
                    let frame = Frame::new_from_iterator(iterator);
                    self.frames.push(frame);
                }
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
            let frame = if let Some(iterator) = context.children() {
                Frame::new_from_iterator(iterator)
            } else {
                Frame::new_from_single(context)
            };
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



    pub(crate) fn in_sequence(&self) -> bool {
        self.frames[self.frames.len() - 1].iterator.is_some()
    }

    pub(crate) fn current(&self) -> Option<&ContextRef<'a>> {
        self.frames[self.frames.len() - 1].current()
    }

    pub(crate) fn is_falsy(&self) -> bool {
        self.current().is_none_or(|context| context.is_falsy())
    }

    pub(crate) fn next(&mut self) -> bool {
        if let Some(mut frame) = self.frames.pop() {
            let more = frame.next();
            self.frames.push(frame);
            more
        } else {
            false
        }
    }

    pub(crate) fn get(&mut self, name: &str) -> Option<ContextValue> {
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

    pub fn value(&self) -> ContextValue {
        match self.current() {
            Some(context) => context.value(),
            _ => ContextValue::Text("".to_owned())
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

        assert_eq!(stack.get("name"), sct("John Doe"));
        assert!(!stack.push("xxx", None));
        assert!(stack.push("phones", None));
        assert_eq!(stack.get("prefix"), sct("+44"));
        assert_eq!(stack.get("extension"), sct("1234567"));
        assert_eq!(stack.get("aaa"), None);
        assert!(stack.next());
        assert_eq!(stack.get("prefix"), sct("+44"));
        assert_eq!(stack.get("extension"), sct("2345678"));
        assert!(!stack.next());
        assert_eq!(stack.get("age"), sct("43"));
    }

    #[test]
    fn normal_backtrack() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones", None);
        assert!(stack.push("stuff", None));
        assert_eq!(stack.value(), ct("item1"));
        assert!(stack.next());
        assert_eq!(stack.value(), ct("item2"));
        assert!(!stack.next());
        assert_eq!(stack.get("extension"), sct("1234567"));
    }

    #[test]
    fn dotted_from_top() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert!(stack.push("obj.part2", None));
        assert_eq!(stack.value(), ct("yyy"));
    }

    #[test]
    fn dotted_after_backtrack() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones", None);
        assert!(stack.push("obj.part2", None));
        assert_eq!(stack.value(), ct("yyy"));
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
        assert_eq!(stack.value(), ct("John Doe"));
    }

    #[test]
    fn edge_case_safety() {
        // Test edge cases that could previously cause panics
        
        // Test empty stack next() - should not panic
        let root = json1();
        let mut stack = Stack::new(&root);
        // Clear all frames to test empty stack
        stack.truncate(0);
        // This should return false, not panic
        assert!(!stack.next());
        
        // Test zero numbers in JSON are falsy
        let zero_int = serde_json::json!(0);
        assert!(zero_int.is_falsy());
        
        let zero_float = serde_json::json!(0.0);
        assert!(zero_float.is_falsy());
        
        // Test negative zero
        let neg_zero = serde_json::json!(-0.0);
        assert!(neg_zero.is_falsy());
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

    fn ct(text: &str) -> ContextValue {
        ContextValue::Text(text.to_owned())
    }

    fn sct(text: &str) -> Option<ContextValue> {
        Some(ct(text))
    }
}
