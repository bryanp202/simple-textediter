use std::path::PathBuf;

mod parse;

pub enum Command {
    ERROR,
    QUIT,
    WRITE(PathBuf),
    OPEN(PathBuf),
    JUMP(u32, u32),
}

impl Command {
    pub fn new(cmd_str: String) -> Self {
        parse::parse(cmd_str)
    }
}