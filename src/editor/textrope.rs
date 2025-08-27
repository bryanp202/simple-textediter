mod rope;

use rope::Rope;
use crate::{editor::{cursor::Cursor, windowstate::WindowState}, vector::Vector2D};

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

    pub fn undo(mut self, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let Some(undo_action) = self.undo_stack.pop() else {
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

    pub fn insert(self, index: usize, insert_text: String, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        if insert_text.len() == 0 {
            return self;
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
            redo_stack: Vec::new(),
        }
    }
}

impl TextRope {
    fn execute_new_insert(self, index: usize, insert_text: String, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let cursor_start = cursor.pos();

        let (mut new_text_data, len) = self._insert(index, &insert_text);
        cursor.text_shift_x(len as isize, &new_text_data, window);

        let cursor_end = cursor.pos();
        new_text_data.undo_stack.push(Action::new_remove(index, cursor_end, cursor_start, len));
        new_text_data
    }

    fn execute_new_remove(self, index: usize, len: usize, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let cursor_start = cursor.pos();

        let shift_amt = len as isize;
        cursor.text_shift_x(-shift_amt, &self, window);
        let (mut new_text_data, insert_text) = self._remove(index, len);        

        let cursor_end = cursor.pos();
        new_text_data.undo_stack.push(Action::new_insert(index, cursor_end, cursor_start, insert_text));
        new_text_data
    }

    fn execute_new_replace(self, index: usize, len: usize, replace_text: String, jump_pos: Vector2D, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        cursor.jump_to(jump_pos.x, jump_pos.y, &self, window);
        let (new_text_data, removed_text) = self._remove(index, len);
        let (mut new_text_data, replace_len) = new_text_data._insert(index, &replace_text);
        cursor.text_shift_x(replace_len as isize, &new_text_data, window);

        let cursor_end = cursor.pos();
        new_text_data.undo_stack.push(Action::new_replace(index, cursor_end, jump_pos, replace_len, removed_text));
        new_text_data
    }

    fn execute_new_delete(self, index: usize, len: usize, cursor: &mut Cursor, window: &mut WindowState) -> Self {
        let cursor_pos = cursor.pos();

        let (mut new_text_data, removed_text) = self._remove(index, len);
        cursor.jump_to(cursor_pos.x, cursor_pos.y, &new_text_data, window);
        new_text_data.undo_stack.push(Action::new_append(index, cursor_pos, removed_text));
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
            redo_stack: self.redo_stack,
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
            redo_stack: self.redo_stack,
        }, removed_text)
    }
}

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