use crate::editor::textrope::rope::Rope;

pub enum Action {
    Insert { index: usize, insert_text: String },
    Remove { index: usize, len: usize },
}

impl Action {
    pub fn new_insert(index: usize, insert_text: String) -> Self {
        Action::Insert { index, insert_text }
    }

    pub fn new_remove(index: usize, len: usize) -> Self {
        Action::Remove { index, len }
    }

    pub fn execute(self, text_data: Rope, start_len: usize, start_lines: usize) -> (Self, Rope, usize, usize) {
        match self {
            Action::Insert { index, insert_text } => {
                let len = insert_text.chars().count();
                let lines = Rope::get_line_count(&insert_text);
                let new_text_data = text_data.insert(index, &insert_text);
                let inverted_action = Action::Remove { index, len };
                (inverted_action, new_text_data, start_len + len, start_lines + lines)
            },
            Action::Remove { index, len } => {
                let insert_text = text_data.chars().skip(index).take(len).collect::<String>();
                let lines = Rope::get_line_count(&insert_text);
                let new_text_data = text_data.remove(index, len);
                let inverted_action = Action::Insert { index, insert_text };
                (inverted_action, new_text_data, start_len - len, start_lines - lines)
            },
        }
    }
}