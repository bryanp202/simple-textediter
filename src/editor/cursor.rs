use sdl3::{pixels::Color, render::{Canvas, FPoint}, video::Window};

use crate::{editor::rope::TextRope, vector::Vector2D};
use std::{error::Error, time::{Duration, Instant}};

const DEFAULT_BLINK_PERIOD: Duration = Duration::from_millis(500);
const DEFAULT_CUSROR_COLOR: Color = super::DEFAULT_FONT_COLOR;

pub struct Cursor {
    pos: Vector2D,
    snap_x: u32,
    height: f32,
    width: f32,
    blink_on: bool,
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

    pub fn draw_context(&self) -> (f32, Vector2D) {
        (self.height, self.pos)
    }

    pub fn shift_x(&mut self, amt: isize, text_data: &TextRope) {
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
        self.move_to(new_x, new_y)
    }

    pub fn shift_y(&mut self, amt: isize, text_data: &TextRope) {
        let new_y = (self.pos.y as isize).saturating_add(amt).clamp(0, text_data.line_count() as isize  - 1) as u32;
        let line_len = text_data.lines().nth(new_y as usize).unwrap().chars().count() as u32;
        let new_x = self.pos.x.max(self.snap_x).min(line_len);
        self.move_to(new_x, new_y)
    }

    pub fn ret(&mut self) {
        self.snap_x = 0;
        self.move_to(0, self.pos.y + 1)
    }

    pub fn move_to(&mut self, x: u32, y: u32) {
        self.pos.x = x;
        self.pos.y = y;
        self.reset_blink();
    }

    pub fn reset_blink(&mut self) {
        self.blink_timer = Instant::now();
        self.blink_on = true;
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, text_pad: u32, line_pad: u32) -> Result<(), Box<dyn Error>> {
        if self.blink_on {
            canvas.set_draw_color(self.color);
            let text_pad = text_pad as f32;
            let line_pad = line_pad as f32;

            let x =  self.pos.x as f32 * self.width + text_pad;
            let y = self.pos.y as f32 * (self.height + line_pad) + text_pad;

            let start = FPoint::new(x, y);
            let end = FPoint::new(x, y + self.height);
            canvas.draw_line(start, end)?;
        }

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

impl Default for Cursor {
    fn default() -> Self {
        Self {
            height: 0.0,
            width: 0.0,
            pos: Vector2D {
                x: 0,
                y: 0,
            },
            snap_x: 0,
            blink_on: false,
            blink_period: DEFAULT_BLINK_PERIOD,
            color: DEFAULT_CUSROR_COLOR,
            blink_timer: Instant::now(),
        }
    }
}