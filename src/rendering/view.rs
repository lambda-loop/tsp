//

use macroquad::prelude::{Rect, screen_height, screen_width};

const MARGIN: f32 = 50.0;

#[derive(Debug, Clone)]
pub struct View {
    min_x : f32,
    min_y : f32,
    width : f32,
    height: f32,
}

impl View {
    pub fn new(ps: &[(f32, f32)]) -> Self {
        // TODO: make them act in the same time?
        let min_x = ps.iter().map(|p| p.0).fold(f32::INFINITY, f32::min);
        let max_x = ps.iter().map(|p| p.0).fold(f32::NEG_INFINITY, f32::max);
        let min_y = ps.iter().map(|p| p.1).fold(f32::INFINITY, f32::min);
        let max_y = ps.iter().map(|p| p.1).fold(f32::NEG_INFINITY, f32::max);

        Self {
            min_x,
            min_y,
            width: (max_x - min_x).max(1.0),
            height: (max_y - min_y).max(1.0),
        }
    }

    pub fn screen_bounds() -> Rect {
        Rect::new(
            MARGIN,
            MARGIN,
            (screen_width() - MARGIN * 2.0).max(1.0),
            (screen_height() - MARGIN * 2.0).max(1.0),
        )
    }

    pub fn to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        self.project(x, y, Self::screen_bounds())
    }

    pub fn project(&self, x: f32, y: f32, bounds: Rect) -> (f32, f32) {
        let scale = (bounds.w / self.width).min(bounds.h / self.height);
        let offset_x = bounds.x + (bounds.w - self.width * scale) / 2.0;
        let offset_y = bounds.y + (bounds.h - self.height * scale) / 2.0;

        let sx = (x - self.min_x) * scale + offset_x;
        let sy = (y - self.min_y) * scale + offset_y;

        (sx, sy)
    }
}
