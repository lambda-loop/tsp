mod evolution;
mod genetic;
mod local_search;
mod population;
mod problem;
mod rendering;
mod showcase;

use ::rand::{make_rng, rngs::SmallRng};
use macroquad::prelude::*;
use population::InstanceScreen;
use problem::TSP;
use rendering::renderer::{Map, Renderer};
use showcase::Showcase;

const INSTANCES: [(&str, &str, usize); 8] = [
    ("wi29", include_str!("../data/wi29.tsp"), 29),
    ("dj38", include_str!("../data/dj38.tsp"), 38),
    ("eil51", include_str!("../data/eil51.tsp"), 51),
    ("berlin52", include_str!("../data/berlin52.tsp"), 52),
    ("st70", include_str!("../data/st70.tsp"), 70),
    ("eil76", include_str!("../data/eil76.tsp"), 76),
    ("pr76", include_str!("../data/pr76.tsp"), 76),
    ("rat99", include_str!("../data/rat99.tsp"), 99),
];

enum Screen {
    Instance(InstanceScreen),
    Showcase(Showcase),
}

#[macroquad::main(window_conf)]
async fn main() {
    let previews = INSTANCES.map(|(name, source, _)| {
        let tsp = TSP::parser(source).unwrap_or_else(|| panic!("invalid bundled instance: {name}"));
        Renderer::new(Map::from_tsp(tsp))
    });
    let mut screen = None;
    let mut menu_scroll = 0.0;
    let mut rng: SmallRng = make_rng();

    loop {
        if is_key_pressed(KeyCode::Q) {
            break;
        }

        clear_background(BLACK);

        if let Some(active) = &mut screen {
            match active {
                Screen::Instance(instance) => instance.draw(&mut rng),
                Screen::Showcase(showcase) => showcase.draw(),
            }
            let back = draw_button(Rect::new(12.0, 8.0, 88.0, 36.0), "Menu");
            if back || is_key_pressed(KeyCode::Escape) {
                screen = None;
            }
        } else {
            let selected = draw_menu(&previews, &mut menu_scroll);
            draw_rectangle(0.0, 0.0, screen_width(), 52.0, BLACK);
            if draw_button(Rect::new(12.0, 8.0, 116.0, 36.0), "Showcase") {
                screen = Some(Screen::Showcase(Showcase::new()));
            } else if let Some(preview) = selected {
                screen = Some(Screen::Instance(start_instance(preview.map.tsp.clone())));
            }
        }

        next_frame().await;
    }
}

fn start_instance(tsp: TSP) -> InstanceScreen {
    InstanceScreen::new(Renderer::new(Map::from_tsp(tsp)))
}

fn draw_menu<'a>(previews: &'a [Renderer], scroll: &mut f32) -> Option<&'a Renderer> {
    const GAP: f32 = 16.0;
    const CARD_WIDTH: f32 = 240.0;
    const CARD_HEIGHT: f32 = 180.0;
    const MIN_CARD_WIDTH: f32 = 180.0;
    const HEADER_HEIGHT: f32 = 52.0;

    let width = screen_width();
    let height = screen_height();
    let columns = (((width - GAP) / (MIN_CARD_WIDTH + GAP)) as usize).clamp(1, 3);
    let card_width = ((width - GAP * (columns + 1) as f32) / columns as f32).clamp(1.0, CARD_WIDTH);
    let grid_width = columns as f32 * (card_width + GAP) - GAP;
    let grid_height = previews.len().div_ceil(columns) as f32 * (CARD_HEIGHT + GAP) - GAP;
    let left = (width - grid_width) / 2.0;
    let visible_height = (height - HEADER_HEIGHT).max(0.0);
    let top = HEADER_HEIGHT + ((visible_height - grid_height) / 2.0).max(GAP);
    let max_scroll = (grid_height + 2.0 * GAP - visible_height).max(0.0);
    *scroll = (*scroll - mouse_wheel().1 * 48.0).clamp(0.0, max_scroll);

    let mouse = Vec2::from(mouse_position());
    let clicked = is_mouse_button_pressed(MouseButton::Left);
    let mut selected = None;

    for (index, preview) in previews.iter().enumerate() {
        let row = index / columns;
        let column = index % columns;
        let x = left + column as f32 * (card_width + GAP);
        let y = top + row as f32 * (CARD_HEIGHT + GAP) - *scroll;
        let bounds = Rect::new(x, y, card_width, CARD_HEIGHT);
        if bounds.bottom() <= HEADER_HEIGHT || bounds.y >= height {
            continue;
        }

        let hovered = mouse.y >= HEADER_HEIGHT && bounds.contains(mouse);
        let fill = if hovered {
            DARKBLUE
        } else {
            Color::from_rgba(25, 28, 34, 255)
        };
        draw_rectangle(x, y, card_width, CARD_HEIGHT, fill);
        if hovered {
            draw_rectangle_lines(x, y, card_width, CARD_HEIGHT, 2.0, SKYBLUE);
            if clicked {
                selected = Some(preview);
            }
        }

        let inset = (card_width * 0.15).min(18.0);
        preview.draw_preview(Rect::new(
            x + inset,
            y + inset,
            card_width - 2.0 * inset,
            CARD_HEIGHT - 2.0 * inset,
        ));
    }

    selected
}

fn draw_button(bounds: Rect, label: &str) -> bool {
    let hovered = bounds.contains(mouse_position().into());
    let color = if hovered { DARKBLUE } else { DARKGRAY };
    draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, color);

    let font_size = bounds.h.min(22.0) as u16;
    let text = measure_text(label, None, font_size, 1.0);
    let x = bounds.x + (bounds.w - text.width) / 2.0;
    let y = bounds.y + (bounds.h - text.height) / 2.0 + text.offset_y;
    draw_text(label, x, y, font_size as f32, WHITE);

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn window_conf() -> Conf {
    Conf {
        window_title: "TSP".into(),
        window_width: 1000,
        window_height: 800,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_instances_match_menu() {
        for (name, source, cities) in INSTANCES {
            let tsp = TSP::parser(source).unwrap_or_else(|| panic!("invalid instance: {name}"));
            assert_eq!(tsp.name, name);
            assert_eq!(tsp.len, cities);
            assert!(tsp.len < 100);
            for i in 0..tsp.len {
                for j in i + 1..tsp.len {
                    assert_eq!(tsp.distance(i, j), tsp.distance(j, i));
                }
            }
            assert_eq!(start_instance(tsp).renderer.map.points.len(), cities);
        }
    }

    #[test]
    fn tour_cost_includes_return_to_start() {
        let tsp = TSP::parser(
            "NAME: triangle\nTYPE: TSP\nDIMENSION: 3\nEDGE_WEIGHT_TYPE: EUC_2D\n\
             NODE_COORD_SECTION\n1 0 0\n2 3 0\n3 3 4\nEOF\n",
        )
        .unwrap();

        assert_eq!(tsp.tour_cost(&[0, 1, 2]), 12);
    }
}
