mod rope;

use rope::Rope;
use crate::{editor::{cursor::Cursor, windowstate::WindowState}, vector::Vector2D};

pub struct TextRope {
    root: Rope,
    len: usize,
    line_count: usize,
    undo_stack: Vec<Action>,
    current_action: Option<Action>,
    redo_stack: Vec<Action>,
    space_flag: bool,
}

impl TextRope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn undo(mut self, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let Some(undo_action) = self.current_action.take().or_else(|| self.undo_stack.pop()) else {
            return self;
        };
        let (mut new_rope, inverted_action) = undo_action.execute(self, cursor, window);
        new_rope.redo_stack.push(inverted_action);
        new_rope
    }

    pub fn redo(mut self, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let Some(redo_action) = self.redo_stack.pop() else {
            return self;
        };
        let (mut new_rope, inverted_action) = redo_action.execute(self, cursor, window);
        new_rope.undo_stack.push(inverted_action);
        new_rope
    }

    pub fn insert(mut self, index: usize, insert_text: String, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        if insert_text.len() == 0 {
            return self;
        }
        if insert_text.len() == 1 && insert_text.as_bytes()[0] == b' ' {
            if !self.space_flag {
                self.push_current_action();
                self.space_flag = true;
            }
        } else if self.space_flag {
            self.space_flag = false;
            self.push_current_action();
        }
        self.execute_new_insert(index, insert_text, cursor, window)
    }

    // pub fn append(self, insert_text: String, cursor: &mut Cursor, window: &mut WindowState) -> Self {
    //     if insert_text.len() == 0 {
    //         return self;
    //     }
    //     let index = self.len;
    //     self.execute_new_insert(index, insert_text, cursor, window)
    // }

    pub fn remove(self, index: usize, len: usize, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        if len == 0 {
            return self;
        }
        self.execute_new_remove(index, len, cursor, window)
    }

    pub fn delete(self, index: usize, len: usize, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        if len == 0 {
            return self;
        }
        self.execute_new_delete(index, len, cursor, window)
    }

    pub fn replace(self, index: usize, len: usize, replace_text: String, jump_pos: Vector2D, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        self.execute_new_replace(index, len, replace_text, jump_pos, cursor, window)
    }

    pub fn push_and_insert(mut self, index: usize, insert_text: String, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        self.push_current_action();
        let mut new_self = self.execute_new_insert(index, insert_text, cursor, window);
        new_self.push_current_action();
        new_self
    }

     #[allow(dead_code)]
    pub fn pop(self, len: usize, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        if len == 0 {
            return self;
        }
        let index = self.len - len;
        self.execute_new_remove(index, len, cursor, window)
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
            current_action: None,
            redo_stack: Vec::new(),
            space_flag: false,
        }
    }
}

impl TextRope {
    fn execute_new_insert(self, index: usize, insert_text: String, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let cursor_start = cursor.pos();

        let (mut new_text_data, len) = self._insert(index, &insert_text);
        cursor.text_shift_x(len as isize, &new_text_data, window);

        let cursor_end = cursor.pos();
        new_text_data.push_undo(
            Action::new_remove(index, cursor_end, cursor_start, len),
            cursor.take_tampered_flag(),
        );
        new_text_data
    }

    fn execute_new_remove(self, index: usize, len: usize, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let cursor_start = cursor.pos();

        let shift_amt = len as isize;
        cursor.text_shift_x(-shift_amt, &self, window);
        let (mut new_text_data, insert_text) = self._remove(index, len);        

        let cursor_end = cursor.pos();
        new_text_data.push_undo(
            Action::new_insert(index, cursor_end, cursor_start, insert_text),
            cursor.take_tampered_flag(),
        );
        new_text_data
    }

    fn execute_new_replace(self, index: usize, len: usize, replace_text: String, jump_pos: Vector2D, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        cursor.jump_to(jump_pos.x, jump_pos.y, &self, window);
        let (new_text_data, removed_text) = self._remove(index, len);
        let (mut new_text_data, replace_len) = new_text_data._insert(index, &replace_text);
        cursor.text_shift_x(replace_len as isize, &new_text_data, window);

        let cursor_end = cursor.pos();
        new_text_data.push_undo(
            Action::new_replace(index, cursor_end, jump_pos, replace_len, removed_text),
            cursor.take_tampered_flag(),
        );
        new_text_data
    }

    fn execute_new_delete(self, index: usize, len: usize, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let cursor_pos = cursor.pos();

        let (mut new_text_data, removed_text) = self._remove(index, len);
        cursor.focus_on(&new_text_data, window);
        new_text_data.push_undo(
            Action::new_append(index, cursor_pos, removed_text),
            cursor.take_tampered_flag(),
        );
        new_text_data
    }

    fn _insert(self, index: usize, insert_text: &str) -> (Self, usize) {
        let len = insert_text.chars().count();
        let line_count = Rope::get_line_count(insert_text);
        let new_root = self.root.insert(index, insert_text);

        (Self {
            root: new_root,
            len: self.len + len,
            line_count: self.line_count + line_count,
            undo_stack: self.undo_stack,
            current_action: self.current_action,
            redo_stack: self.redo_stack,
            space_flag: self.space_flag
        }, len)
    }

    fn _remove(self, index: usize, len: usize) -> (Self, String) {
        if len == 0 {
            return (self, String::from(""));
        }
        let removed_text = self.chars().skip(index).take(len).collect::<String>();
        let new_root = self.root.remove(index, len);
        
        (Self {
            len: self.len - len,
            line_count: new_root.line_count() - 1,
            root: new_root,
            undo_stack: self.undo_stack,
            current_action: self.current_action,
            redo_stack: self.redo_stack,
            space_flag: self.space_flag
        }, removed_text)
    }

    /// Tamper flag must be set true if the cursor moved from the last index arrived from previous actions
    fn push_undo(&mut self, new_undo_action: Action, tamper_flag: bool) {
        self.redo_stack.clear();

        let Some(current_action) = self.current_action.take() else {
            self.current_action = Some(new_undo_action);
            return;
        };

        if tamper_flag {
            self.undo_stack.push(current_action);
            self.current_action = Some(new_undo_action);
            return;
        }

        match current_action {
            Action::Append { index, cursor_pos, mut append_text } => {
                match new_undo_action {
                    Action::Append { cursor_pos, append_text: new_append_text, .. } => {
                        append_text.push_str(&new_append_text);
                        self.current_action = Some(Action::new_append(index, cursor_pos, append_text));
                    },
                    _ => {
                        self.undo_stack.push(Action::new_append(index, cursor_pos, append_text));
                        self.current_action = Some(new_undo_action);
                    },
                }
            },
            Action::Delete { index, cursor_pos, len } => {
                match new_undo_action {
                    Action::Delete { len: new_len, .. } => self.current_action = Some(Action::new_delete(index, cursor_pos, len + new_len)),
                    _ => {
                        self.undo_stack.push(current_action);
                        self.current_action = Some(new_undo_action);
                    },
                }
            },
            Action::Replace { index, cursor_start, cursor_end, len, replace_text } => {
                match new_undo_action {
                    Action::Remove { cursor_start: new_cursor_start, len: new_len, .. } => {
                        self.current_action = Some(Action::new_replace(index, new_cursor_start, cursor_end, len + new_len, replace_text));
                    },
                    _ => {
                        self.undo_stack.push(Action::new_replace(index, cursor_start, cursor_end, len, replace_text));
                        self.current_action = Some(new_undo_action);
                    },
                }
            },
            Action::Insert { index, cursor_start, cursor_end, insert_text }=> {
                match new_undo_action {
                    Action::Insert { index: new_index, cursor_start: new_cursor_start, insert_text: mut new_insert_text, .. } => {
                        new_insert_text.push_str(&insert_text);
                        self.current_action = Some(Action::new_insert(new_index, new_cursor_start, cursor_end, new_insert_text));
                    },
                    _ => {
                        self.undo_stack.push(Action::new_insert(index, cursor_start, cursor_end, insert_text));
                        self.current_action = Some(new_undo_action);
                    },
                }
            },
            Action::Remove { index, cursor_end, len, .. } => {
                match new_undo_action {
                    Action::Remove { cursor_start, len: new_len, .. } => {
                        self.current_action = Some(Action::new_remove(index, cursor_start, cursor_end, len + new_len));
                    },
                    _ => {
                        self.undo_stack.push(current_action);
                        self.current_action = Some(new_undo_action);
                    },
                }
            },
        }
    }

    fn push_current_action(&mut self) {
        if let Some(current_action) = self.current_action.take() {
            self.undo_stack.push(current_action);
        }
    }
}

#[derive(Debug)]
pub enum Action {
    Insert { index: usize, cursor_start: Vector2D, cursor_end: Vector2D, insert_text: String }, // Opposite of remove
    Remove { index: usize, cursor_start: Vector2D, cursor_end: Vector2D, len: usize }, // Oppositve of insert
    Replace { index: usize, cursor_start: Vector2D, cursor_end: Vector2D, len: usize, replace_text: String}, // Opposite of self
    Delete { index: usize, cursor_pos: Vector2D, len: usize }, // Opposite of append
    Append { index: usize, cursor_pos: Vector2D, append_text: String }, // Opposite of append
}

impl Action {
    fn new_insert(index: usize, cursor_start: Vector2D, cursor_end: Vector2D, insert_text: String) -> Self {
        Self::Insert { index, cursor_start, cursor_end, insert_text }
    }

    fn new_remove(index: usize, cursor_start: Vector2D, cursor_end: Vector2D, len: usize) -> Self {
        Self::Remove { index, cursor_start, cursor_end, len }
    }

    fn new_replace(index: usize, cursor_start: Vector2D, cursor_end: Vector2D, len: usize, replace_text: String) -> Self {
        Self::Replace { index, cursor_start, cursor_end, len, replace_text }
    }

    fn new_delete(index: usize, cursor_pos: Vector2D, len: usize) -> Self {
        Self::Delete { index, cursor_pos, len }
    }

    fn new_append(index: usize, cursor_pos: Vector2D, append_text: String) -> Self {
        Self::Append { index, cursor_pos, append_text }
    }

    fn execute(self, text_data: TextRope, cursor: &mut Cursor, window: &mut WindowState) -> (TextRope, Action) {
        match self {
            Action::Insert { index, cursor_start, cursor_end, insert_text } => {
                let (new_text_data, len) = text_data._insert(index, &insert_text);
                let inverted_action = Action::new_remove(index, cursor_end, cursor_start, len);
                cursor.jump_to(cursor_end.x, cursor_end.y, &new_text_data, window);
                (new_text_data, inverted_action)
            },
            Action::Remove { index, cursor_start, cursor_end, len } => {
                let (new_text_data, insert_text) = text_data._remove(index, len);
                let inverted_action = Action::new_insert(index, cursor_end, cursor_start, insert_text);
                cursor.jump_to(cursor_end.x, cursor_end.y, &new_text_data, window);
                (new_text_data, inverted_action)
            },
            Action::Replace { index, cursor_start, cursor_end, len, replace_text } => {
                let (new_text_data, removed_text) = text_data._remove(index, len);
                let (new_text_data, len) = new_text_data._insert(index, &replace_text);
                let inverted_action = Action::new_replace(index, cursor_end, cursor_start, len, removed_text);
                cursor.jump_to(cursor_end.x, cursor_end.y, &new_text_data, window);
                (new_text_data, inverted_action)
            },
            Action::Delete { index, cursor_pos, len } => {
                let (new_text_data, insert_text) = text_data._remove(index, len);
                let inverted_action = Action::new_append(index, cursor_pos, insert_text);
                cursor.jump_to(cursor_pos.x, cursor_pos.y, &new_text_data, window);
                (new_text_data, inverted_action)
            },
            Action::Append { index, cursor_pos, append_text } => {
                let (new_text_data, len) = text_data._insert(index, &append_text);
                let inverted_action = Action::new_delete(index, cursor_pos, len);
                cursor.jump_to(cursor_pos.x, cursor_pos.y, &new_text_data, window);
                (new_text_data, inverted_action)
            },
        }
    }
}