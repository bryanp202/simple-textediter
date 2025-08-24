use sdl3::{pixels::Color, render::{Canvas, FPoint}, video::Window};

use crate::{editor::{rope::TextRope, windowstate::WindowState}, vector::Vector2D};
use std::{error::Error, time::{Duration, Instant}};

const DEFAULT_BLINK_PERIOD: Duration = Duration::from_millis(500);
const DEFAULT_CUSROR_COLOR: Color = super::DEFAULT_FONT_COLOR;

pub struct Cursor {
    pos: Vector2D,
    select_start_pos: Option<Vector2D>,
    snap_x: u32,
    height: f32,
    width: f32,
    blink_on: bool,
    left_down: bool,
    blink_period: Duration,
    blink_timer: Instant,
    color: Color,
}

impl Cursor {
    pub fn new(width: u32, height: u32) -> Self {
        Self{
            width: width as f32,
            height: height as f32,
            ..Default::default()
        }
    }

    pub fn pos(&self) -> Vector2D {
        self.pos
    }

    pub fn get_dim(&self) -> (f32, f32) {
        (self.width, self.height)
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

    pub fn jump_to(&mut self, x: u32, y: u32, window: &mut WindowState, text_data: &TextRope) {
        self.snap_x = x;
        self.select_start_pos = None;
        self.move_to(x, y, window, text_data)
    }

    pub fn shift_x(&mut self, amt: isize, text_data: &TextRope, window: &mut WindowState) {
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
        self.select_start_pos = None;
        self.move_to(new_x, new_y, window, text_data)
    }

    pub fn shift_y(&mut self, amt: isize, text_data: &TextRope, window: &mut WindowState) {
        let new_y = (self.pos.y as isize).saturating_add(amt).clamp(0, text_data.line_count() as isize  - 1) as u32;
        let line_len = text_data.lines().nth(new_y as usize).unwrap().chars().count() as u32;
        let new_x = self.pos.x.max(self.snap_x).min(line_len);
        self.select_start_pos = None;
        self.move_to(new_x, new_y, window, text_data)
    }

    pub fn left_click_press(&mut self, click_x: f32, click_y: f32, text_data: &TextRope, text_pad: u32, line_pad: u32, window: &mut WindowState) {
        self.left_down = true;
        self.jump_to_mouse(click_x, click_y, text_data, text_pad, line_pad, window);
        self.select_start_pos = Some(self.pos);
    }

    pub fn mouse_move(&mut self, click_x: f32, click_y: f32, text_data: &TextRope, text_pad: u32, line_pad: u32, window: &mut WindowState) -> bool {
        if self.left_down {
            self.jump_to_mouse(click_x, click_y, text_data, text_pad, line_pad, window);
        }

        self.left_down
    }

    pub fn left_click_release(&mut self) {
        self.left_down = false;
    }

    pub fn ret(&mut self, window: &mut WindowState, text_data: &TextRope) {
        self.snap_x = 0;
        self.select_start_pos = None;
        self.move_to(0, self.pos.y + 1, window, text_data)
    }

    pub fn reset_blink(&mut self) {
        self.blink_timer = Instant::now();
        self.blink_on = true;
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, text_pad: u32, line_pad: u32, window: &WindowState) -> Result<(), Box<dyn Error>> {
        if !self.blink_on {
            return Ok(());
        }

        let Some(Vector2D {x: shifted_x, y: shifted_y}) = window.in_screen_bound(self.pos.x, self.pos.y) else {
            return Ok(());
        };

        canvas.set_draw_color(self.color);
        let text_pad = text_pad as f32;
        let line_pad = line_pad as f32;

        let x = shifted_x as f32 * self.width + text_pad;
        let y = shifted_y as f32 * (self.height + line_pad) + text_pad;

        let start = FPoint::new(x, y);
        let end = FPoint::new(x, y + self.height);
        canvas.draw_line(start, end)?;

        Ok(())
    }

    pub fn update(&mut self) -> bool {
        if self.blink_timer.elapsed() > self.blink_period {
            self.blink_on = !self.blink_on;
            self.blink_timer = Instant::now();
            true
        } else {
            false
        }
    }
}

impl Cursor {
    fn jump_to_mouse(&mut self, mouse_x: f32, mouse_y: f32, text_data: &TextRope, text_pad: u32, line_pad: u32, window: &mut WindowState) {
        let text_pad = text_pad as f32;
        let line_pad = line_pad as f32;
    
        let new_x = (mouse_x - text_pad + self.width / 2.0) / self.width;
        let new_x = new_x.max(0.0) as usize + window.get_first_char();
        let new_y = (mouse_y - text_pad) / (self.height + line_pad);
        let new_y = new_y.max(0.0) as usize + window.get_first_line();

        let new_y = new_y.min(text_data.line_count() - 1);
        let new_x = new_x.min(text_data.lines().nth(new_y as usize).unwrap().chars().count());
        self.snap_x = new_x as u32;

        self.move_to(new_x as u32, new_y as u32, window, text_data)
    }

    fn move_to(&mut self, x: u32, y: u32, window: &mut WindowState, text_data: &TextRope) {
        window.adjust_focus(x as usize, y as usize, text_data);
        self.pos.x = x;
        self.pos.y = y;
        self.reset_blink()
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            height: 0.0,
            width: 0.0,
            pos: Vector2D {
                x: 0,
                y: 0,
            },
            select_start_pos: None,
            snap_x: 0,
            blink_on: true,
            left_down: false,
            blink_period: DEFAULT_BLINK_PERIOD,
            color: DEFAULT_CUSROR_COLOR,
            blink_timer: Instant::now(),
        }
    }
}