use std::path::PathBuf;

use crate::editor::textbox::TextBox;

mod parse;

#[derive(Clone)]
pub enum Command {
    ERROR,
    QUIT,
    WRITE(PathBuf),
    OPEN(PathBuf),
    JUMP(u32, u32),
    RUN(String, Vec<String>),
    FIND(Option<String>),
    PREVIOUS,
}

impl Command {
    pub fn new(cmd_str: String) -> Self {
        parse::parse(cmd_str)
    }
}

pub struct CommandState {
    find_cmd: Option<(String, usize)>,
    replace_cmd: Option<(String, String, usize)>,
    previous_cmd: Command,
}

impl CommandState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_prev(&mut self, cmd: Command) {
        match cmd {
            Command::PREVIOUS => {},
            _ => self.previous_cmd = cmd,
        }
    }

    pub fn execute_cmd(&mut self, textbox: &mut TextBox, cmd: Command) {
        match cmd {
            Command::FIND(pattern) => self.execute_find(textbox, &pattern),
            Command::PREVIOUS => self.execute_cmd(textbox, self.previous_cmd.clone()),
            _ => {},
        }
    }
}

impl CommandState {
    fn execute_find(&mut self, textbox: &mut TextBox, maybe_pattern: &Option<String>) {
        if let Some(pattern) = maybe_pattern {
            let cursor_index = textbox.cursor_index();
            self.find_cmd = Some((pattern.clone(), cursor_index));
        }

        let Some((pattern, start_index)) = self.find_cmd.take() else {
            return;
        };

        if let Some(index) = textbox.find(&pattern, start_index) {
            self.find_cmd = Some((pattern, index));
        }
    }
}

impl Default for CommandState {
    fn default() -> Self {
        Self {
            find_cmd: None,
            replace_cmd: None,
            previous_cmd: Command::ERROR,
        }
    }
}