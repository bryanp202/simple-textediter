use sdl3::rect::Rect;
use crate::editor::TextAlignment;

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