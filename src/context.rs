use std::fmt::Debug;
use std::collections::VecDeque;


pub trait Context<'a>: Debug {
    fn child(&'a self, name: &str) -> Option<ContextRef<'a>>;
    fn children(&'a self) -> Option<Vec<ContextRef<'a>>>;
    fn value(&self) -> Option<String>;
    fn is_truthy(&self) -> bool;
}

pub type ContextRef<'a> = &'a dyn Context<'a>;


#[derive(Debug, Clone)]
struct Frame<'a> {
    // VecDeque to avoid quadratic complexity when removing from start.
    contexts: VecDeque<ContextRef<'a>>,
    resolve_down: bool
}

impl<'a> Frame<'a> {
    fn new(contexts: Vec<ContextRef<'a>>, resolve_down: bool) -> Self {
        let contexts = VecDeque::from(contexts);
        Frame {
            contexts,
            resolve_down
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


#[derive(Debug, Clone)]
pub(crate) struct Stack<'a> {
    frames: Vec<Frame<'a>>,
    backtrack_depth: usize
}

impl<'a> Stack<'a> {
    pub(crate) fn new(root: ContextRef<'a>) -> Self {
        let frame = Frame::new(vec![root], false);
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

    fn push_dotted(&mut self, name: &str, dotted: bool) -> bool {
        if name == "." {
            if let Some(children) = self.children() {
                self.frames.push(
                    Frame::new(children, !dotted)
                )
            };
            true
        } else if let Some(idx) = name.find(".") {
            let (head, tail) = name.split_at(idx);
            self.push_dotted(head, true) && self.push(&tail[1..])

        } else if let Some(context) = self.child(name) {
            let contexts = if let Some(children) = context.children() {
                children
            } else {
                vec![context]
            };
            let frame = Frame::new(contexts, !dotted);
            self.frames.push(frame);
            true
        
        } else {
            let mut resolved = false;
            if self.top().resolve_down {
                let mut ts = self.backtracking();
                loop {
                    resolved = ts.push(name);
                    if resolved || !ts.top().resolve_down {
                        break;
                    }
                    ts.down();
                }
                if resolved {
                    self.merge(ts);
                }
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
            self.frames.truncate(len - 1);
            if self.backtrack_depth > 0 {
                self.backtrack_depth += 1;
            }
            true
        } else {
            false
        }
    }

    fn top(&self) -> &Frame<'a> {
        self.frames.last().unwrap()
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

    fn value(&self) -> Option<String> {
        self.current()?.value()
    }

    pub(crate) fn get(&mut self, name: &str) -> Option<String> {
        if name == "." {
            self.value()
        } else {
            let len = self.len();
            self.push(name);
            let result = self.value();
            self.truncate(len);
            result
        }
    }

    pub(crate) fn is_truthy(&self) -> bool {
        self.current().map_or(
            false,
             |context| context.is_truthy()
        )
    }

    fn merge(&mut self, other: Stack<'a>) {
        let unchanged = self.frames.len() - other.backtrack_depth;
        self.frames.extend_from_slice(&other.frames[unchanged..]);
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
        assert_eq!(stack.value().unwrap(), "item1");
        assert!(stack.next());
        assert_eq!(stack.value().unwrap(), "item2");
        assert!(!stack.next());
        assert!(stack.down());
        assert_eq!(stack.get("extension").unwrap(), "1234567");
   }

    #[test]
   fn dotted_from_top() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert!(stack.push("obj.part2"));
        assert_eq!(stack.value().unwrap(), "yyy");
   }

    #[test]
   fn dotted_after_backtrack() {
        let root = json1();
        let mut stack = Stack::new(&root);

        stack.push("phones");
        assert!(stack.push("obj.part2"));
        assert_eq!(stack.value().unwrap(), "yyy");
   }

    #[test]
    fn broken_chain() {
        let root = json1();
        let mut stack = Stack::new(&root);

        assert!(!stack.push("obj.part1.part2"));
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
