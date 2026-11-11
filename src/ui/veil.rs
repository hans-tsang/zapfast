//! The privacy screen (Ctrl+Shift+P): names, pictures, chat previews,
//! messages and the composer's text are drawn as soft bars until the pointer
//! rests on them, so someone glancing at the screen reads nothing. It lasts
//! until toggled off or ZapFast quits, and is never saved.
//!
//! Content is laid out as usual and painted at zero opacity, or replaced by
//! bars where it is painted by hand, so turning the screen on moves nothing
//! and every click target, selection and screen-reader label stays where it
//! was.

use egui::{Color32, Rect, Ui, pos2, vec2};

use crate::theme::{self, Icon, Palette};

/// How much of the text's colour a bar takes.
const INK: f32 = 0.2;

/// The height of a placeholder line, as a share of the line it hides.
const BAR_SHARE: f32 = 0.6;

/// The spacing of bars standing for lines of a message.
const LINE_PITCH: f32 = 20.0;

/// Taller than this, a message is media rather than lines of text, and shows
/// a closed eye instead of bars.
const TEXT_HEIGHT: f32 = 140.0;

/// Whether content in `rect` is hidden this frame: the screen is on and the
/// pointer is not over it.
pub fn hides(ui: &Ui, active: bool, rect: Rect) -> bool {
    active && !(rect.is_positive() && ui.rect_contains_pointer(rect))
}

/// A soft bar where a line of text of `size` would be, starting at `pos`.
pub fn line(ui: &Ui, pos: egui::Pos2, size: egui::Vec2, color: Color32) {
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    let height = (size.y * BAR_SHARE).max(4.0);
    let rect = Rect::from_min_size(
        pos2(pos.x, pos.y + (size.y - height) / 2.0),
        vec2(size.x, height),
    );
    ui.painter()
        .rect_filled(rect, height / 2.0, color.gamma_multiply(INK));
}

/// A plain circle where a profile picture would be.
pub fn avatar(ui: &Ui, palette: &Palette, rect: Rect) {
    ui.painter().circle_filled(
        rect.center(),
        rect.width() / 2.0,
        palette.dim.gamma_multiply(0.35),
    );
}

/// Bars, or a closed eye for media, standing for a message's hidden contents
/// within `rect`. `seed` varies the bars' lengths so messages differ.
pub fn message(ui: &Ui, palette: &Palette, rect: Rect, seed: &str) {
    if rect.width() < 8.0 || rect.height() < 8.0 {
        return;
    }
    if rect.height() > TEXT_HEIGHT {
        let size = (rect.width().min(rect.height()) * 0.25).clamp(18.0, 40.0);
        theme::paint_icon(
            ui,
            Icon::EyeOff,
            Rect::from_center_size(rect.center(), egui::Vec2::splat(size)),
            size,
            palette.text.gamma_multiply(0.5),
        );
        return;
    }
    let lines = ((rect.height() / LINE_PITCH).floor() as usize).max(1);
    let pitch = rect.height() / lines as f32;
    let mut hash = seed.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    });
    for index in 0..lines {
        // The last line runs short, as a paragraph's does.
        let share = if index + 1 == lines && lines > 1 {
            0.35 + (hash % 30) as f32 / 100.0
        } else {
            0.75 + (hash % 25) as f32 / 100.0
        };
        hash = hash.wrapping_mul(1_103_515_245).wrapping_add(12_345) >> 3;
        line(
            ui,
            pos2(rect.left(), rect.top() + pitch * index as f32),
            vec2(rect.width() * share.min(1.0), pitch),
            palette.text,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pointer_at(pos: Option<egui::Pos2>) -> egui::RawInput {
        let mut input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(400.0, 300.0))),
            ..Default::default()
        };
        input.events.push(match pos {
            Some(pos) => egui::Event::PointerMoved(pos),
            None => egui::Event::PointerGone,
        });
        input
    }

    fn hidden(active: bool, pointer: Option<egui::Pos2>) -> bool {
        let ctx = egui::Context::default();
        let rect = Rect::from_min_size(pos2(10.0, 10.0), vec2(100.0, 40.0));
        let mut result = false;
        for _ in 0..2 {
            let _ = ctx.run_ui(pointer_at(pointer), |ui| {
                result = hides(ui, active, rect);
            });
        }
        result
    }

    #[test]
    fn hides_only_while_on_and_away_from_the_pointer() {
        assert!(!hidden(false, None));
        assert!(!hidden(false, Some(pos2(500.0, 500.0))));
        assert!(hidden(true, None));
        assert!(hidden(true, Some(pos2(200.0, 200.0))));
        assert!(!hidden(true, Some(pos2(50.0, 30.0))));
    }
}
