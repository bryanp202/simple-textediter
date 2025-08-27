mod action;
mod rope;

use action::Action;
use rope::Rope;

pub struct TextRope {
    root: Rope,
    len: usize,
    line_count: usize,
    undo_stack: Vec<Action>,
    redo_stack: Vec<Action>,
}

impl TextRope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn undo(mut self) -> (Self, isize) {
        let Some(undo_action) = self.undo_stack.pop() else {
            return (self, 0);
        };
        let start_len = self.len;
        let (redo_action, root, len, line_count) = undo_action.execute(self.root, start_len, self.line_count);
        (Self {
            root,
            len,
            line_count,
            undo_stack: self.undo_stack,
            redo_stack: push_action_stack(self.redo_stack, redo_action),
        }, start_len as isize - len as isize)
    }

    pub fn redo(mut self) -> (Self, isize) {
        let Some(redo_action) = self.redo_stack.pop() else {
            return (self, 0);
        };
        let start_len = self.len;
        let (undo_action, root, len, line_count) = redo_action.execute(self.root, start_len, self.line_count);
        (Self {
            root,
            len,
            line_count,
            undo_stack: push_action_stack(self.undo_stack, undo_action),
            redo_stack: self.redo_stack,
        }, start_len as isize - len as isize)
    }

    pub fn insert(self, index: usize, insert_text: String) -> Self {
        if insert_text.len() == 0 {
            return self;
        }
        self.execute_action(Action::new_insert(index, insert_text))
    }

    pub fn append(self, insert_text: String) -> Self {
        if insert_text.len() == 0 {
            return self;
        }
        let index = self.len;
        self.execute_action(Action::new_insert(index, insert_text))
    }

    pub fn remove(self, index: usize, len: usize) -> Self {
        if len == 0 {
            return self;
        }
        self.execute_action(Action::new_remove(index, len))
    }

     #[allow(dead_code)]
    pub fn pop(self, len: usize) -> Self {
        if len == 0 {
            return self;
        }
        let index = self.len - len;
        self.execute_action(Action::new_remove(index, len))
    }

     #[allow(dead_code)]
    pub fn get(&self, index: usize) -> Option<char> {
        self.root.get(index)
    }

    pub fn get_line_index(&self, target_line: usize) -> usize {
        self.root.line_start_index(target_line)
    }

    pub fn chars(&self) -> rope::RopeIterator {
        self.root.chars()
    }

    pub fn lines(&self) -> rope::RopeLineIterator {
        self.root.lines()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn line_count(&self) -> usize {
        self.line_count + 1
    }
}

impl Default for TextRope {
    fn default() -> Self {
        Self {
            root: Rope::new(),
            len: 0,
            line_count: 0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl TextRope {
    fn execute_action(mut self, action: Action) -> Self {
        self.redo_stack.clear();
        let (undo_action, root, len, line_count) = action.execute(self.root, self.len, self.line_count);
        Self {
            root,
            len,
            line_count,
            undo_stack: push_action_stack(self.undo_stack, undo_action),
            redo_stack: self.redo_stack
        }
    }
}

fn push_action_stack(mut stack: Vec<Action>, action: Action) -> Vec<Action> {
    stack.push(action);
    stack
}