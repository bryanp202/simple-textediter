use std::str::{FromStr, Split};

use crate::editor::command::Command;

pub fn parse(cmd_str: String) -> Command {
    if let Some(':') = cmd_str.chars().nth(0) {
        parse_editor_cmd(&cmd_str[1..])
    } else {
        Command::ERROR
    }
}

pub fn parse_editor_cmd(cmd_str: &str) -> Command {
    let mut words = cmd_str.split(' ');

    match words.by_ref().next() {
        Some("q") => Command::QUIT,
        Some("j") => parse_jump_cmd(words),
        Some("w") => parse_write_cmd(words),
        Some("o") => parse_open_cmd(words),
        Some("r") => parse_run_cmd(words),
        _ => Command::ERROR,
    }
}

fn parse_jump_cmd(mut words: Split<char>) -> Command {
    let Ok(Some(line_num)) = parse_num_arg::<u32>(&mut words) else {
        return Command::ERROR;
    };
    let Some(line_nume_min_one) = line_num.checked_sub(1) else {
        return Command::ERROR;
    };

    let cmd = match parse_num_arg::<u32>(&mut words) {
        Ok(Some(column_num)) => {
            if let Some(col_num_min_one) = column_num.checked_sub(1) {
                Command::JUMP(col_num_min_one, line_nume_min_one)
            } else {
                Command::ERROR
            }
        },
        Ok(None) => Command::JUMP(0, line_nume_min_one),
        Err(_) => Command::ERROR,
    };

    check_rem(words, cmd)
}

fn parse_write_cmd(mut words: Split<char>) -> Command {
    let Some(file_path) = words.next() else {
        return Command::ERROR;
    };

    let path_buf = file_path.into();
    let cmd = Command::WRITE(path_buf);

    check_rem(words, cmd)
}

fn parse_open_cmd(mut words: Split<char>) -> Command {
    let Some(file_path) = words.next() else {
        return Command::ERROR;
    };

    let path_buf = file_path.into();
    let cmd = Command::OPEN(path_buf);

    check_rem(words, cmd)
}

fn parse_run_cmd(mut words: Split<char>) -> Command {
    let Some(program) = words.next() else {
        return Command::ERROR;
    };

    let args = words.map(|word| word.to_string()).collect::<Vec<String>>();

    Command::RUN(program.to_string(), args)
}

/// Helpers
fn parse_num_arg<T>(words: &mut Split<char>) -> Result<Option<T>, ()>
where T: FromStr {
    let Some(num_str) = words.next() else {
        return Ok(None);
    };
    num_str.parse::<T>().map_or(Err(()), |num|  Ok(Some(num)))
}

fn check_rem(mut words: Split<char>, cmd: Command) -> Command {
    if let Some(_) = words.next() {
        Command::ERROR
    } else {
        cmd
    }
}