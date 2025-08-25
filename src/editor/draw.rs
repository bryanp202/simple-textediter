use std::cmp::Ordering;
use std::error::Error;

use sdl3::pixels::Color;
use sdl3::render::FRect;
use sdl3::{rect::Rect, render::Canvas, video::Window};
use crate::editor::{windowstate::WindowState, TextAlignment};
use crate::editor::cursor::Cursor;
use crate::vector::Vector2D;

pub fn text_target_aligned(alignment: &TextAlignment, padding: u32, start_y: u32, text_w: u32, text_h: u32, screen_w: u32) -> Rect {
    match alignment {
        TextAlignment::LEFT => text_target_left(padding, start_y, text_w, text_h),
        TextAlignment::CENTER => text_target_center(start_y, text_w, text_h, screen_w),
        TextAlignment::RIGHT => text_target_right(padding, start_y, text_w, text_h, screen_w),
    }
}

fn text_target_left(padding: u32, start_y: u32, text_w: u32, text_h: u32) -> Rect {
    Rect::new(padding as i32, start_y as i32, text_w as u32, text_h as u32)
}

fn text_target_center(start_y: u32, text_w: u32, text_h: u32, screen_w: u32) -> Rect {
    let x = (screen_w as isize - text_w as isize) / 2;
    Rect::new(x as i32, start_y as i32, text_w as u32, text_h as u32)
}

fn text_target_right(padding: u32, start_y: u32, text_w: u32, text_h: u32, screen_w: u32) -> Rect {
    let x = screen_w as isize - text_w as isize - padding as isize;
    let text_w = text_w;
    Rect::new(x as i32, start_y as i32, text_w as u32, text_h as u32)
}

pub fn selection_box(
    canvas: &mut Canvas<Window>,
    cursor: &Cursor,
    window: &WindowState,
    line_num: usize,
    line_len: usize,
    select_color: Color,
) -> Result<(), Box<dyn Error>> {
    let Some(Vector2D { x: select_char, y: select_line}) = cursor.select_start_pos() else {
        return Ok(());
    };
    let Vector2D { x: cursor_char, y: cursor_line } = cursor.pos();

    let (start_char, start_line, end_char, end_line) = match cursor_line.cmp(&select_line) {
        Ordering::Less => (cursor_char, cursor_line, select_char, select_line),
        Ordering::Equal=> if cursor_char < select_char {
            (cursor_char, cursor_line, select_char, select_line)
        } else {
            (select_char, select_line, cursor_char, cursor_line)
        },
        Ordering::Greater => (select_char, select_line, cursor_char, cursor_line),
    };

    if line_num < start_line as usize || line_num > end_line as usize {
        return Ok(());
    }

    let (current_line_start_char, current_line_end_char) = if line_num == start_line as usize {
        if start_line != end_line {
            (start_char, line_len as u32)
        } else {
            (start_char, end_char)
        }
    } else if line_num == end_line as usize {
        (0, end_char)
    } else {
        (0, line_len as u32)
    };

    let (text_pad, line_pad) = window.get_padding();
    let (char_width, char_height) = window.get_text_dim();
    let char_width = char_width as u32;
    let line_height = char_height as u32 + line_pad;
    let adjusted_char = current_line_start_char.saturating_sub(window.get_first_char() as u32);
    let adjusted_line = line_num.saturating_sub(window.get_first_line()) as u32;
    let x = adjusted_char * char_width + text_pad;
    let y   = adjusted_line * line_height + text_pad;

    let chars = (current_line_end_char - current_line_start_char).min(line_len as u32);
    let width = (chars * char_width).max(4);

    let frect = FRect::new(x as f32, y as f32, width as f32, char_height);
    canvas.set_draw_color(select_color);
    canvas.fill_rect(frect)?;

    Ok(())
}