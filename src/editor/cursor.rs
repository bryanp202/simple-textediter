use sdl3::{pixels::Color, render::{Canvas, FPoint}, video::Window};

use crate::{editor::{inputstate::InputState, textrope::TextRope, windowstate::WindowState}, vector::Vector2D};
use std::{error::Error, time::{Duration, Instant}, u32, usize};

const DEFAULT_BLINK_PERIOD: Duration = Duration::from_millis(500);
const DEFAULT_CURSOR_COLOR: Color = crate::editor::textbox::DEFAULT_FONT_COLOR;

pub struct Cursor {
    pos: Vector2D,
    select_start_pos: Option<Vector2D>,
    snap_x: u32,
    blink_period: Duration,
    blink_timer: Instant,
    color: Color,
    blink_on: bool,
    tampered_flag: bool,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pos(&self) -> Vector2D {
        self.pos
    }

    pub fn take_tampered_flag(&mut self) -> bool {
        let flag = self.tampered_flag;
        self.tampered_flag = false;
        flag
    }

    pub fn select_start_pos(&self) -> Option<Vector2D> {
        if let Some(select_start) = self.select_start_pos {
            if select_start.x != self.pos.x || select_start.y != self.pos.y {
                Some(select_start)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn jump_to(&mut self, x: u32, y: u32, input: &InputState, text_data: &TextRope, window: &mut WindowState) {
        self.snap_x = x;
        self.reset_select_pos(input);
        self.move_to(x, y, window, text_data)
    }

    pub fn text_jump_to(&mut self, x: u32, y: u32, text_data: &TextRope, window: &mut WindowState) {
        self.snap_x = x;
        self.select_start_pos = None;
        self.move_to_no_tamper_flag(x, y, window, text_data)
    }

    pub fn focus_on(&mut self, text_data: &TextRope, window: &mut WindowState) {
        let (x, y) = self.pos.into();
        window.adjust_focus(x as usize, y as usize, text_data);
        self.reset_blink();
    }

    pub fn text_shift_x(&mut self, amt: isize, text_data: &TextRope, window: &mut WindowState) {
        let (new_x, new_y) = self.align_x(amt, text_data);
        self.select_start_pos = None;
        self.move_to_no_tamper_flag(new_x, new_y, window, text_data)
    }

    pub fn shift_x(&mut self, amt: isize, input: &InputState, text_data: &TextRope, window: &mut WindowState) {
        let (new_x, new_y) = match (input.keyboard.ctrl_down(), input.keyboard.shift_down(), self.select_start_pos()) {
            (true, ..) => self.align_word_x(amt, text_data),
            (_, false, Some(select_start_pos)) => {
                if amt >= 0 {
                    select_start_pos.max(self.pos).into()
                } else {
                    select_start_pos.min(self.pos).into()
                }
            }
            _ => self.align_x(amt, text_data),
        };
        self.reset_select_pos(input);
        self.move_to(new_x, new_y, window, text_data)
    }

    pub fn shift_y(&mut self, amt: isize, input: &InputState, text_data: &TextRope, window: &mut WindowState) {
        let (new_x, new_y) = self.align_y(amt, text_data);
        self.reset_select_pos(input);
        self.move_to(new_x, new_y, window, text_data)
    }

    pub fn mouse_move(&mut self, click_x: f32, click_y: f32, input: &InputState, text_data: &TextRope, window: &mut WindowState) {
        if input.mouse.left_down() {
            self.jump_to_mouse(click_x, click_y, text_data, window);
        }
    }

    pub fn select_all(&mut self, text_data: &TextRope, window: &mut WindowState) {
        self.select_start_pos = Some(Vector2D::new(0, 0));
        let last_line_index = text_data.line_count() - 1;
        let last_line = text_data.lines().nth(last_line_index).unwrap();
        let last_line_len = last_line.chars().count();
        self.move_to_no_adjust(last_line_len as u32, last_line_index as u32, window);
    }

    pub fn draw(&mut self, active: bool, canvas: &mut Canvas<Window>, window: &WindowState) -> Result<(), Box<dyn Error>> {
        if !active || !self.blink_on {
            return Ok(());
        }

        let Some(Vector2D {x: shifted_x, y: shifted_y}) = window.in_screen_bound(self.pos.x, self.pos.y) else {
            return Ok(());
        };

        canvas.set_draw_color(self.color);
        let (text_pad, line_pad) = window.get_padding();
        let pos = window.pos();
        let text_pad = text_pad as f32;
        let line_pad = line_pad as f32;
        let (width, height) = window.get_text_dim();

        let x = shifted_x as f32 * width + text_pad + pos.x as f32;
        let y = shifted_y as f32 * (height + line_pad) + text_pad + pos.y as f32;

        let start = FPoint::new(x, y);
        let end = FPoint::new(x, y + height);
        canvas.draw_line(start, end)?;

        Ok(())
    }

    pub fn update(&mut self, window: &mut WindowState) {
        if self.blink_timer.elapsed() > self.blink_period {
            self.blink_on = !self.blink_on;
            self.blink_timer = Instant::now();
            window.set_render_flag()
        }
    }

    pub fn home(&mut self, input: &InputState, text_data: &TextRope, window: &mut WindowState) {
        let y = if input.keyboard.ctrl_down() {
            0
        } else {
            self.pos().y
        };
        self.jump_to(0, y, input, text_data, window);
    }
}

impl Cursor {
    pub fn left_click_press(&mut self, click_x: f32, click_y: f32, clicks: u8, text_data: &TextRope, window: &mut WindowState) {
        match clicks {
            1 => {
                self.jump_to_mouse(click_x, click_y, text_data, window);
                self.select_start_pos = Some(self.pos);
            },
            2 => {
                let (char_num, line_num) = snap_click_pos(click_x, click_y, window, text_data);
                self.select_word_or_chunk(line_num as u32, char_num as u32, text_data, window);
            },
            3 => {
                let (_, line_num) = snap_click_pos(click_x, click_y, window, text_data);
                let line_len = text_data.lines().nth(line_num).unwrap().chars().count();
                self.select_start_pos = Some(Vector2D::new(0, line_num as u32));
                self.move_to(line_len as u32, line_num as u32, window, text_data);
            },
            _ => self.select_all(text_data, window),
        }
    }

    pub fn select_around_cursor(&mut self, text_data: &TextRope, window: &mut WindowState) {
        let Vector2D{x: char_num, y: line_num} = self.pos;
        self.select_word_or_chunk(line_num, char_num, text_data, window);
    }
}

impl Cursor {
    fn reset_blink(&mut self) {
        self.blink_timer = Instant::now();
        self.blink_on = true;
    }

    fn jump_to_mouse(&mut self, mouse_x: f32, mouse_y: f32, text_data: &TextRope, window: &mut WindowState) {
        let (new_x, new_y) = snap_click_pos(mouse_x, mouse_y, window, text_data);
        self.snap_x = new_x as u32;

        self.move_to(new_x as u32, new_y as u32, window, text_data)
    }

    fn move_to(&mut self, x: u32, y: u32, window: &mut WindowState, text_data: &TextRope) {
        self.tampered_flag = true;
        window.adjust_focus(x as usize, y as usize, text_data);
        self.pos.x = x;
        self.pos.y = y;
        self.reset_blink()
    }

    fn move_to_no_adjust(&mut self, x: u32, y: u32, window: &mut WindowState) {
        self.tampered_flag = true;
        self.pos.x = x;
        self.pos.y = y;
        window.set_render_flag();
        self.reset_blink()
    }
    fn move_to_no_tamper_flag(&mut self, x: u32, y: u32, window: &mut WindowState, text_data: &TextRope) {
        window.adjust_focus(x as usize, y as usize, text_data);
        self.pos.x = x;
        self.pos.y = y;
        self.reset_blink()
    }

    fn reset_select_pos(&mut self, input: &InputState) {
        if !input.keyboard.shift_down() {
            self.select_start_pos = None;
        } else {
            if let None = self.select_start_pos {
                self.select_start_pos = Some(self.pos);
            }
        }
    }

    /// Returns (new_x, new_y)
    fn align_x(&mut self, amt: isize, text_data: &TextRope) -> (u32, u32) {
        let shifted_x = (self.pos.x as isize).saturating_add(amt);
        let mut new_x = shifted_x as u32;
        let mut new_y = self.pos.y;
        if shifted_x < 0 {
            if let Some(shifted_y) = self.pos.y.checked_sub(1) {
                new_y = shifted_y;
                new_x = text_data.lines()
                    .nth(new_y as usize)
                    .map(|line_str| line_str.chars().count())
                    .unwrap_or(0) as u32;
            } else {
                new_x = 0;
            }
        } else {
            let mut line_iter = text_data.lines().skip(new_y as usize);
            let mut line_len = line_iter.next().map(|line_str| line_str.chars().count() as u32).unwrap();
            while new_x > line_len {
                let Some(next_line_len) = line_iter.next().map(|line_str| line_str.chars().count() as u32) else {
                    new_x = line_len;
                    break;
                };
                new_y += 1;
                new_x -= line_len + 1;
                line_len = next_line_len;
            }
        };
        self.snap_x = new_x;
        (new_x, new_y)
    }

    fn align_word_x(&mut self, amt: isize, text_data: &TextRope) -> (u32, u32) {
        let (start_x, start_y) = self.pos.into();
        let (new_x, new_y) = if amt >= 0{
            match find_end_of_chunk(start_y, start_x, text_data) {
                Ok(new_x) => (new_x, start_y),
                Err(last_x) => {
                    if start_y + 1 < text_data.line_count() as u32 {
                        (find_end_of_chunk(start_y + 1, 0, text_data).unwrap_or(0), start_y + 1)
                    } else {
                        (last_x, start_y)
                    }
                }
            }
        } else {
            match find_start_of_chunk(start_y, start_x, text_data) {
                Ok(new_x) => (new_x, start_y),
                Err(last_x) => {
                    if start_y >= 1 {
                        (find_start_of_chunk(start_y - 1, u32::MAX, text_data).unwrap_or(0), start_y - 1)
                    } else {
                        (last_x, start_y)
                    }
                }
            }
        };
        self.snap_x = new_x;
        (new_x, new_y)
    }

    /// Returns (new_x, new_y)
    fn align_y(&self, amt: isize, text_data: &TextRope) -> (u32, u32) {
        let new_y = (self.pos.y as isize).saturating_add(amt).clamp(0, text_data.line_count() as isize  - 1) as u32;
        let line_len = text_data.lines().nth(new_y as usize).unwrap().chars().count() as u32;
        let new_x = self.pos.x.max(self.snap_x).min(line_len);
        (new_x, new_y)
    }

    fn select_word_or_chunk(&mut self, line_num: u32, char_num: u32, text_data: &TextRope, window: &mut WindowState) {
        let line_text = text_data.lines().nth(line_num as usize).unwrap();
        
        let mut first_left_space: Option<usize> = None;
        let mut first_alpha_space: Option<usize> = None;
        let mut first_left_symbol: Option<usize> = None;
        let mut char_iter = line_text.chars().enumerate();
        for (i, c) in char_iter.by_ref().take(char_num as usize) {
            if c == ' ' {
                first_left_space = first_left_space.or(Some(i));
                first_alpha_space = None;
                first_left_symbol = None;
            } else if is_identifier(c) {
                first_alpha_space = first_alpha_space.or(Some(i));
                first_left_space = None;
                first_left_symbol = None;
            } else {
                first_left_symbol = first_left_symbol.or(Some(i));
                first_left_space = None;
                first_alpha_space = None;
            }
        }
        let (target_offset, target_char) = char_iter.by_ref().next().map_or((0, ' '), |(_, c)| (1, c));
        let (start_x, end_x) = if target_char == ' ' {
            let last_space_index = char_iter.by_ref()
                .take_while(|&(_, c)| c == ' ')
                .last()
                .map_or(
                    (char_num + target_offset) as usize,
                     |(i, _)| i + 1
                );
            (first_left_space.unwrap_or(char_num as usize), last_space_index)
        } else if target_char.is_alphanumeric() || target_char == '_' {
            let last_alpha_index = char_iter
                .take_while(|&(_, c)| is_identifier(c))
                .last()
                .map_or(char_num as usize, |(i, _)| i);
            (first_alpha_space.map_or(char_num as usize, |x| x), last_alpha_index + 1)
        } else {
            let last_symbol_index = char_iter
                .take_while(|&(_, c)| is_symbol(c))
                .last()
                .map_or(char_num as usize, |(i, _)| i);
            (first_left_symbol.map_or(char_num as usize, |x| x), last_symbol_index + 1)
        };

        self.select_start_pos = Some(Vector2D::new(start_x as u32, line_num));
        self.move_to(end_x as u32, line_num, window, text_data)
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            pos: Vector2D {
                x: 0,
                y: 0,
            },
            select_start_pos: None,
            snap_x: 0,
            blink_period: DEFAULT_BLINK_PERIOD,
            color: DEFAULT_CURSOR_COLOR,
            blink_timer: Instant::now(),
            blink_on: true,
            tampered_flag: false,
        }
    }
}

/// Returns (char, line)
fn snap_click_pos(mouse_x: f32, mouse_y: f32, window: &WindowState, text_data: &TextRope) -> (usize, usize) {
    let pos = window.pos();

    let mouse_x = mouse_x - pos.x as f32;
    let mouse_y = mouse_y - pos.y as f32;

    let (text_pad, line_pad) = window.get_padding();
    let text_pad = text_pad as f32;
    let line_pad = line_pad as f32;
    let (width, height) = window.get_text_dim();

    let new_x = (mouse_x - text_pad + width / 2.0) / width;
    let new_x = new_x.max(0.0) as usize + window.get_first_char();
    let new_y = (mouse_y - text_pad) / (height + line_pad);
    let new_y = new_y.max(0.0) as usize + window.get_first_line();

    let new_y = new_y.min(text_data.line_count() - 1);
    let new_x = new_x.min(text_data.lines().nth(new_y as usize).unwrap().chars().count());
    (new_x, new_y)
}

fn find_start_of_chunk(line_num: u32, start_char: u32, text_data: &TextRope) -> Result<u32, u32> {
    if start_char == 0 {
        return Err(0);
    }
    let curr_line = text_data.lines().nth(line_num as usize).unwrap();
    let (first_alpha, first_symbol, _) = curr_line
        .chars()
        .enumerate()
        .take(start_char as usize)
        .fold((None, None, None), |(first_alpha, first_symbol, last_space), (i, c)| {
            match c {
                ' ' => (first_alpha, first_symbol, Some(i)),
                c if is_identifier(c) => (first_alpha.filter(|_| last_space.is_none()).or(Some(i)), None, None),
                _ => (None, first_symbol.filter(|_| last_space.is_none()).or(Some(i)), None),
            }
        });
    Ok(first_symbol.or(first_alpha).unwrap_or(0) as u32)
}

fn find_end_of_chunk(line_num: u32, start_char: u32, text_data: &TextRope) -> Result<u32, u32> {
    let curr_line = text_data.lines().nth(line_num as usize).unwrap();
    let mut char_iter = curr_line.chars().enumerate().skip(start_char as usize);
    let Some((target_char_index, target_char)) = char_iter.by_ref().next() else {
        return Err(start_char);
    };

    let end_index = match target_char {
        ' ' => {
            let after_space_skip = char_iter
                .by_ref()
                .filter(|&(_, c)| c != ' ')
                .next()
                .map_or((target_char_index, None), |(i, c)| (i, Some(c)));
            match after_space_skip {
                (last_alpha, Some(c)) if is_identifier(c) => char_iter.take_while(|&(_, c)| is_identifier(c))
                    .last()
                    .map_or(last_alpha, |(i, _)| i),
                (last_symbol, Some(_)) => char_iter.take_while(|&(_, c)| is_symbol(c))
                    .last()
                    .map_or(last_symbol, |(i, _)| i),
                (last_space_plus_one, None) => last_space_plus_one - 1,
            }
        },
        c if is_identifier(c) => char_iter
                    .take_while(|&(_, c)| is_identifier(c))
                    .last()
                    .map_or(target_char_index, |(i, _)| i),
        _ => char_iter
                    .take_while(|&(_, c)| is_symbol(c))
                    .last()
                    .map_or(target_char_index, |(i, _)| i),
    };
    Ok(end_index as u32 + 1)
}

fn is_identifier(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_symbol(c: char) -> bool {
    !is_identifier(c) && c != ' '
}