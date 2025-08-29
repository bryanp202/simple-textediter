use std::path::PathBuf;

mod parser;

pub enum Command {
    QUIT,
    WRITE(PathBuf),
    OPEN(PathBuf),
    GOTO(usize),
}