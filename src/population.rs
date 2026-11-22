use ::rand::{RngExt, rngs::SmallRng, seq::index::sample};
use fixedbitset::FixedBitSet;
use macroquad::{
    prelude::*,
    ui::{hash, root_ui, widgets::Editbox},
};

use crate::{
    draw_button,
    genetic::{crossing, mutation, new_tour},
    local_search::LocalSearch,
    problem::Tour,
    rendering::renderer::Renderer,
};

#[derive(Clone)]
struct PopulationTour {
    tour: Tour,
    cost: u64,
    selected: bool,
}

#[derive(Clone, Copy)]
struct Iteration {
    remaining: usize,
    tournaments: usize,
    tournament_size: usize,
    selecting: bool,
    crossover: CrossoverOptions,
}

#[derive(Clone, Copy, Default)]
struct CrossoverOptions {
    mutation_probability: f64,
    local_search: LocalSearch,
}

pub struct InstanceScreen {
    pub renderer: Renderer,
    population: Vec<PopulationTour>,
    chairs: Vec<PopulationTour>,
    parents: Vec<PopulationTour>,
    show_chairs: bool,
    mutation_input: String,
    local_search: LocalSearch,
    crossover: CrossoverOptions,
    tournament_input: String,
    rounds_input: String,
    iterations_input: String,
    iteration: Option<Iteration>,
    best_seen: Option<PopulationTour>,
    worst_seen: u64,
    tournament_size: usize,
    pending_rounds: usize,
    min_cost: u64,
    max_cost: u64,
    color_by_cost: bool,
    sorted_by_cost: bool,
    target_size: usize,
    size_input: String,
    error: Option<&'static str>,
    scroll: f32,
}

impl InstanceScreen {
    pub fn new(renderer: Renderer) -> Self {
        Self {
            renderer,
            population: Vec::new(),
            chairs: Vec::new(),
            parents: Vec::new(),
            show_chairs: false,
            mutation_input: "15".into(),
            local_search: LocalSearch::default(),
            crossover: CrossoverOptions::default(),
            tournament_input: "2".into(),
            rounds_input: "4".into(),
            iterations_input: "1".into(),
            iteration: None,
            best_seen: None,
            worst_seen: 0,
            tournament_size: 0,
            pending_rounds: 0,
            min_cost: u64::MAX,
            max_cost: 0,
            color_by_cost: false,
            sorted_by_cost: false,
            target_size: 0,
            size_input: String::new(),
            error: None,
            scroll: 0.0,
        }
    }

    pub fn draw(&mut self, rng: &mut SmallRng) {
        if self.target_size == 0 {
            self.draw_setup(rng);
            return;
        }

        self.generate_tours(rng);
        self.run_tournaments(rng);
        self.advance_iteration();

        let thermometer = Rect::new(108.0, 8.0, 36.0, 36.0);
        let thermometer_hovered = thermometer.contains(mouse_position().into());
        if thermometer_hovered && is_mouse_button_pressed(MouseButton::Left) {
            self.color_by_cost = !self.color_by_cost;
        }

        let thermometer_top = 180.0;
        let header_height = thermometer_top + 58.0;
        self.draw_population(header_height);
        draw_rectangle(0.0, 0.0, screen_width(), header_height, BLACK);

        let fill = if self.color_by_cost || thermometer_hovered {
            DARKBLUE
        } else {
            DARKGRAY
        };
        draw_rectangle(
            thermometer.x,
            thermometer.y,
            thermometer.w,
            thermometer.h,
            fill,
        );
        let center = thermometer.center();
        draw_rectangle_lines(center.x - 3.5, center.y - 11.0, 7.0, 16.0, 1.5, WHITE);
        draw_circle(center.x, center.y + 6.0, 5.5, WHITE);
        draw_circle(center.x, center.y + 6.0, 3.5, fill);
        draw_line(
            center.x,
            center.y - 5.0,
            center.x,
            center.y + 6.0,
            2.0,
            if self.color_by_cost { SKYBLUE } else { WHITE },
        );

        if draw_button(Rect::new(152.0, 8.0, 64.0, 36.0), "Sort") {
            self.sorted_by_cost = true;
            self.population.sort_by_key(|entry| entry.cost);
            self.scroll = 0.0;
        }

        self.draw_global_thermometer(thermometer_top);
        self.tournament_controls();

        if self.population.len() < self.target_size && self.error.is_none() {
            let progress = format!("{}/{} tours", self.population.len(), self.target_size);
            draw_text(&progress, 228.0, 32.0, 20.0, WHITE);
        }
    }

    fn generate_tours(&mut self, rng: &mut SmallRng) {
        let cities = self.renderer.map.tsp.len;
        let remaining = self.target_size - self.population.len();
        let search = self.crossover.local_search;
        let tours_per_frame = if search.lin_kernighan {
            2
        } else if search.two_opt || search.relocate {
            16
        } else {
            128
        };
        let batch = remaining.min(tours_per_frame);
        let mut crossover_buffer = FixedBitSet::with_capacity(cities);
        let start = self.population.len();
        for child in start..start + batch {
            let tour = if self.parents.is_empty() {
                new_tour(rng, cities)
            } else {
                let partners = self.parents.len() - 1;
                let first = child / partners;
                let second = child % partners;
                let second = second + usize::from(second >= first);
                let mut tour = vec![0; cities].into_boxed_slice();
                crossover_buffer.clear();
                let _ = crossing(
                    &self.parents[first].tour,
                    &self.parents[second].tour,
                    &mut tour,
                    rng,
                    cities,
                    &mut crossover_buffer,
                );
                if rng.random_bool(self.crossover.mutation_probability) {
                    mutation(&mut tour, rng, cities);
                }
                self.crossover
                    .local_search
                    .improve(&self.renderer.map.tsp, &mut tour);
                tour
            };
            let cost = self.renderer.map.tsp.tour_cost(&tour);
            self.min_cost = self.min_cost.min(cost);
            self.max_cost = self.max_cost.max(cost);
            self.worst_seen = self.worst_seen.max(cost);
            let entry = PopulationTour {
                tour,
                cost,
                selected: false,
            };
            if self.best_seen.as_ref().is_none_or(|best| cost < best.cost) {
                self.best_seen = Some(entry.clone());
            }
            self.population.push(entry);
        }
        if self.population.len() == self.target_size && !self.parents.is_empty() {
            self.preserve_best();
            self.parents.clear();
        }
        if self.sorted_by_cost && batch > 0 {
            self.population.sort_by_key(|entry| entry.cost);
        }
    }

    fn preserve_best(&mut self) {
        let Some(best) = &self.best_seen else {
            return;
        };
        if self.population.iter().any(|entry| entry.tour == best.tour) {
            return;
        }

        if let Some(worst) = self.population.iter_mut().max_by_key(|entry| entry.cost) {
            *worst = best.clone();
            self.min_cost = best.cost;
            self.max_cost = self
                .population
                .iter()
                .map(|entry| entry.cost)
                .max()
                .unwrap();
        }
    }

    fn tournament_controls(&mut self) {
        let manual = self.iteration.is_none();
        if draw_button(Rect::new(12.0, 52.0, 92.0, 36.0), "Torneio") && manual {
            self.start_tournaments(Some(1));
        }
        if draw_button(Rect::new(112.0, 52.0, 158.0, 36.0), "Rodar K torneios") && manual {
            self.start_tournaments(self.rounds_input.parse().ok());
        }
        if draw_button(Rect::new(278.0, 52.0, 78.0, 36.0), "Cruzar") && manual {
            self.start_crossing();
        }
        if draw_button(Rect::new(364.0, 52.0, 136.0, 36.0), "Iterar M vezes") && manual {
            self.start_iteration();
        }

        let fields = [
            (
                "N por torneio",
                hash!("tournament-size"),
                &mut self.tournament_input,
            ),
            (
                "K torneios",
                hash!("tournament-rounds"),
                &mut self.rounds_input,
            ),
            (
                "M ciclos",
                hash!("iteration-count"),
                &mut self.iterations_input,
            ),
            (
                "Mutação (%)",
                hash!("mutation-chance"),
                &mut self.mutation_input,
            ),
        ];
        let mut x = 12.0;
        for (label, id, value) in fields {
            draw_text(label, x, 119.0, 20.0, WHITE);
            x += measure_text(label, None, 20, 1.0).width + 8.0;
            Editbox::new(id, vec2(56.0, 32.0))
                .position(vec2(x, 96.0))
                .multiline(false)
                .filter(&|c| c.is_ascii_digit())
                .ui(&mut root_ui(), value);
            x += 72.0;
        }

        draw_text("Busca local", 12.0, 155.0, 20.0, WHITE);
        let two_opt = Rect::new(119.0, 133.0, 88.0, 34.0);
        let relocate = Rect::new(215.0, 133.0, 112.0, 34.0);
        let lin_kernighan = Rect::new(335.0, 133.0, 172.0, 34.0);
        if draw_button(two_opt, "2-opt") {
            self.local_search.two_opt = !self.local_search.two_opt;
        }
        if draw_button(relocate, "Relocação") {
            self.local_search.relocate = !self.local_search.relocate;
        }
        if draw_button(lin_kernighan, "Lin–Kernighan") {
            self.local_search.lin_kernighan = !self.local_search.lin_kernighan;
        }
        for (bounds, active) in [
            (two_opt, self.local_search.two_opt),
            (relocate, self.local_search.relocate),
            (lin_kernighan, self.local_search.lin_kernighan),
        ] {
            if active {
                draw_rectangle_lines(bounds.x, bounds.y, bounds.w, bounds.h, 2.0, SKYBLUE);
            }
        }

        if let Some(error) = self.error {
            draw_text(error, 228.0, 32.0, 20.0, ORANGE);
        } else if let Some(iteration) = self.iteration {
            let remaining = iteration.remaining + usize::from(!self.parents.is_empty());
            let progress = format!("Ciclos restantes: {remaining}");
            draw_text(&progress, 420.0, 32.0, 20.0, WHITE);
        } else if self.pending_rounds > 0 {
            let pending = format!("Torneios restantes: {}", self.pending_rounds);
            draw_text(&pending, 228.0, 32.0, 20.0, WHITE);
        }
    }

    fn draw_global_thermometer(&self, top: f32) {
        let optimum = match self.renderer.map.tsp.name.as_str() {
            // Proven optima from TSPLIB95's symmetric TSP table.
            "eil51" => 426,
            "berlin52" => 7_542,
            "st70" => 675,
            "eil76" => 538,
            "pr76" => 108_159,
            "rat99" => 1_211,
            _ => return,
        };
        let Some(best) = self.best_seen.as_ref().map(|entry| entry.cost) else {
            return;
        };
        let worst = self.worst_seen;

        let gap = 100.0 * best.saturating_sub(optimum) as f64 / optimum as f64;
        let caption =
            format!("Global   Ótimo {optimum}   Melhor {best} (+{gap:.1}%)   Pior {worst}");
        draw_text(&caption, 12.0, top + 18.0, 20.0, WHITE);

        let bar = Rect::new(
            12.0,
            top + 28.0,
            (screen_width() - 24.0).clamp(1.0, 540.0),
            10.0,
        );
        for step in 0..60 {
            let start = bar.x + bar.w * step as f32 / 60.0;
            let end = bar.x + bar.w * (step + 1) as f32 / 60.0;
            draw_rectangle(
                start,
                bar.y,
                end - start,
                bar.h,
                temperature(step as f32 / 59.0),
            );
        }
        let ratio = if worst > optimum {
            (best.saturating_sub(optimum) as f64 / (worst - optimum) as f64).min(1.0) as f32
        } else {
            0.0
        };
        let marker = bar.x + ratio * bar.w;
        draw_line(marker, bar.y - 4.0, marker, bar.bottom() + 4.0, 3.0, WHITE);
    }

    fn configured_crossover(&mut self) -> Option<CrossoverOptions> {
        let percent = self
            .mutation_input
            .parse::<u8>()
            .ok()
            .filter(|&value| value <= 100);
        if percent.is_none() {
            self.error = Some("Informe uma chance de mutação de 0 a 100%.");
        }
        percent.map(|value| CrossoverOptions {
            mutation_probability: f64::from(value) / 100.0,
            local_search: self.local_search,
        })
    }

    fn start_iteration(&mut self) {
        if self.pending_rounds > 0 || self.population.len() != self.target_size {
            self.error = Some("Aguarde o processamento atual.");
            return;
        }

        let remaining = self.iterations_input.parse::<usize>().unwrap_or(0);
        let tournaments = self.rounds_input.parse::<usize>().unwrap_or(0);
        let tournament_size = self.tournament_input.parse::<usize>().unwrap_or(0);
        if remaining == 0 || tournaments < 2 {
            self.error = Some("Informe M positivo e K de pelo menos 2.");
            return;
        }
        let Some(children) = tournaments.checked_mul(tournaments - 1) else {
            self.error = Some("Quantidade de descendentes grande demais.");
            return;
        };
        if !(1..=self.population.len()).contains(&tournament_size)
            || (remaining > 1 && tournament_size > children)
        {
            self.error = Some("N deve caber em todas as populações do ciclo.");
            return;
        }

        let Some(crossover) = self.configured_crossover() else {
            return;
        };
        self.chairs.clear();
        self.iteration = Some(Iteration {
            remaining,
            tournaments,
            tournament_size,
            selecting: false,
            crossover,
        });
        self.advance_iteration();
    }

    fn advance_iteration(&mut self) {
        let Some(mut iteration) = self.iteration else {
            return;
        };
        if self.pending_rounds > 0 || self.population.len() != self.target_size {
            return;
        }
        if iteration.remaining == 0 {
            self.iteration = None;
            return;
        }

        if iteration.selecting {
            self.start_crossing_with_options(iteration.crossover);
            iteration.selecting = false;
            iteration.remaining -= 1;
        } else {
            self.start_tournament_batch(iteration.tournament_size, iteration.tournaments);
            iteration.selecting = true;
        }
        self.iteration = self.error.is_none().then_some(iteration);
    }

    fn start_tournaments(&mut self, rounds: Option<usize>) {
        let size = self.tournament_input.parse::<usize>().unwrap_or(0);
        self.start_tournament_batch(size, rounds.unwrap_or(0));
    }

    fn start_tournament_batch(&mut self, size: usize, rounds: usize) {
        self.error = if self.pending_rounds > 0 || self.population.len() != self.target_size {
            Some("Aguarde o processamento atual.")
        } else if !(1..=self.population.len()).contains(&size) {
            Some("N deve estar entre 1 e o tamanho de Pop.")
        } else if rounds == 0 {
            Some("K deve ser um inteiro positivo.")
        } else if self.chairs.try_reserve(rounds).is_err() {
            Some("Não há memória para todas as cadeiras.")
        } else {
            None
        };
        if self.error.is_some() {
            return;
        }
        for entry in &mut self.population {
            entry.selected = false;
        }
        self.tournament_size = size;
        self.pending_rounds = rounds;
        self.show_chairs = true;
        self.scroll = 0.0;
    }

    fn run_tournaments(&mut self, rng: &mut SmallRng) {
        let batch = self.pending_rounds.min(64);
        for _ in 0..batch {
            let mut contestants =
                sample(rng, self.population.len(), self.tournament_size).into_iter();
            let mut winner = contestants.next().expect("empty tournament");
            self.population[winner].selected = true;
            for index in contestants {
                self.population[index].selected = true;
                if self.population[index].cost < self.population[winner].cost {
                    winner = index;
                }
            }
            let mut chair = self.population[winner].clone();
            chair.selected = false;
            self.chairs.push(chair);
        }
        self.pending_rounds -= batch;
    }

    fn start_crossing(&mut self) {
        if let Some(options) = self.configured_crossover() {
            self.start_crossing_with_options(options);
        }
    }

    fn start_crossing_with_options(&mut self, options: CrossoverOptions) {
        if self.pending_rounds > 0 || self.population.len() != self.target_size {
            self.error = Some("Aguarde o processamento atual.");
            return;
        }
        let count = self.chairs.len();
        if count < 2 {
            self.error = Some("São necessárias pelo menos duas chairs.");
            return;
        }
        let Some(children) = count.checked_mul(count - 1) else {
            self.error = Some("Quantidade de descendentes grande demais.");
            return;
        };
        self.parents = std::mem::take(&mut self.chairs);
        self.crossover = options;
        self.population.clear();
        self.target_size = children;
        self.min_cost = u64::MAX;
        self.max_cost = 0;
        self.scroll = 0.0;
        self.error = None;
    }

    fn draw_setup(&mut self, rng: &mut SmallRng) {
        if is_key_pressed(KeyCode::Space) {
            self.renderer.map.tour = new_tour(rng, self.renderer.map.tsp.len);
        }

        draw_text("Tours", 112.0, 32.0, 22.0, WHITE);
        Editbox::new(hash!("population-size"), vec2(74.0, 36.0))
            .position(vec2(165.0, 8.0))
            .multiline(false)
            .filter(&|c| c.is_ascii_digit())
            .ui(&mut root_ui(), &mut self.size_input);

        let generate = draw_button(Rect::new(247.0, 8.0, 76.0, 36.0), "Gerar");
        if generate || is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter) {
            match self.size_input.parse::<usize>() {
                Ok(count) if count > 0 => {
                    self.target_size = count;
                    self.error = None;
                    return;
                }
                _ => self.error = Some("Digite um número inteiro positivo válido."),
            }
        }

        if let Some(error) = self.error {
            draw_text(error, 12.0, 63.0, 20.0, ORANGE);
        }

        self.renderer.draw(Rect::new(
            16.0,
            72.0,
            (screen_width() - 32.0).max(1.0),
            (screen_height() - 88.0).max(1.0),
        ));
    }

    fn draw_population(&mut self, top: f32) {
        const GAP: f32 = 12.0;
        const WIDTH: f32 = 176.0;
        const HEIGHT: f32 = 128.0;
        const ROW_STRIDE: f32 = HEIGHT + GAP;

        let columns = (((screen_width() - GAP) / (WIDTH + GAP)) as usize).max(1);
        let width = WIDTH.min((screen_width() - 2.0 * GAP).max(1.0));
        let grid_width = columns as f32 * (width + GAP) - GAP;
        let left = (screen_width() - grid_width) / 2.0;
        const TITLE_HEIGHT: f32 = 30.0;
        let visible_height = (screen_height() - top).max(0.0);
        let chairs_height = if self.show_chairs {
            TITLE_HEIGHT + self.chairs.len().div_ceil(columns) as f32 * ROW_STRIDE + GAP
        } else {
            0.0
        };
        let pop_height = TITLE_HEIGHT + self.population.len().div_ceil(columns) as f32 * ROW_STRIDE;
        let max_scroll = (chairs_height + pop_height + GAP - visible_height).max(0.0);
        self.scroll = (self.scroll - mouse_wheel().1 * 48.0).clamp(0.0, max_scroll);

        let mouse: Vec2 = mouse_position().into();
        let mut hovered = None;
        let origin = top - self.scroll;
        let sections = [
            ("Chairs", &self.chairs, origin, self.show_chairs),
            ("Pop", &self.population, origin + chairs_height, true),
        ];
        for (title, entries, y, visible) in sections {
            if !visible {
                continue;
            }
            draw_text(title, left, y + 23.0, 24.0, WHITE);
            let grid_top = y + TITLE_HEIGHT;
            let first_row = ((top - grid_top) / ROW_STRIDE).floor().max(0.0) as usize;
            let last_row = ((screen_height() - grid_top) / ROW_STRIDE).ceil().max(0.0) as usize;
            for row in first_row..entries.len().div_ceil(columns).min(last_row) {
                for (column, entry) in entries[row * columns..].iter().take(columns).enumerate() {
                    let bounds = Rect::new(
                        left + column as f32 * (width + GAP),
                        grid_top + row as f32 * ROW_STRIDE,
                        width,
                        HEIGHT,
                    );
                    if mouse.y >= top && bounds.contains(mouse) {
                        hovered = Some((bounds, entry));
                    }
                    self.draw_card(bounds, entry, false);
                }
            }
        }

        if let Some((bounds, tour)) = hovered {
            let scale = 2.0_f32
                .min(screen_width() / bounds.w)
                .min(visible_height / bounds.h);
            if scale <= 1.0 {
                return;
            }
            let width = bounds.w * scale;
            let height = bounds.h * scale;
            let x =
                (bounds.x + (bounds.w - width) / 2.0).clamp(0.0, (screen_width() - width).max(0.0));
            let y = (bounds.y + (bounds.h - height) / 2.0)
                .clamp(top, (screen_height() - height).max(top));
            self.draw_card(Rect::new(x, y, width, height), tour, true);
        }
    }

    fn draw_card(&self, bounds: Rect, entry: &PopulationTour, enlarged: bool) {
        let fill = if self.color_by_cost {
            cost_color(entry.cost, self.min_cost, self.max_cost)
        } else {
            Color::from_rgba(25, 28, 34, 255)
        };
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, fill);
        if entry.selected || enlarged {
            let border = if entry.selected { VIOLET } else { WHITE };
            draw_rectangle_lines(bounds.x, bounds.y, bounds.w, bounds.h, 3.0, border);
        }

        let inset = (bounds.w * 0.15).min(10.0);
        let font_size = if enlarged { 24 } else { 18 };
        self.renderer.draw_tour_preview(
            Rect::new(
                bounds.x + inset,
                bounds.y + inset,
                bounds.w - 2.0 * inset,
                (bounds.h - f32::from(font_size) - 2.0 * inset).max(1.0),
            ),
            &entry.tour,
            if enlarged { 2.5 } else { 1.5 },
        );
        let cost = entry.cost.to_string();
        let text = measure_text(&cost, None, font_size, 1.0);
        draw_text(
            &cost,
            bounds.right() - inset - text.width,
            bounds.bottom() - 8.0,
            f32::from(font_size),
            WHITE,
        );
    }
}

fn cost_color(cost: u64, min: u64, max: u64) -> Color {
    let ratio = if min == max {
        0.5
    } else {
        (cost - min) as f32 / (max - min) as f32
    };
    temperature(ratio)
}

fn temperature(ratio: f32) -> Color {
    let blue = Color::from_rgba(30, 71, 138, 255);
    let amber = Color::from_rgba(119, 86, 30, 255);
    let red = Color::from_rgba(139, 39, 51, 255);
    let (start, end, amount) = if ratio <= 0.5 {
        (blue, amber, ratio * 2.0)
    } else {
        (amber, red, (ratio - 0.5) * 2.0)
    };
    Color::new(
        start.r + (end.r - start.r) * amount,
        start.g + (end.g - start.g) * amount,
        start.b + (end.b - start.b) * amount,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{problem::TSP, rendering::renderer::Map};

    #[test]
    fn crossing_applies_local_search_with_mutation_disabled() {
        let tsp = TSP::parser(
            "NAME: square\nTYPE: TSP\nDIMENSION: 4\nEDGE_WEIGHT_TYPE: EUC_2D\n\
             NODE_COORD_SECTION\n1 0 0\n2 10 0\n3 10 10\n4 0 10\nEOF\n",
        )
        .unwrap();
        let tour = Box::from([0, 2, 1, 3]);
        let entry = PopulationTour {
            cost: tsp.tour_cost(&tour),
            tour,
            selected: false,
        };
        let mut screen = InstanceScreen::new(Renderer::new(Map::from_tsp(tsp)));
        screen.target_size = 1;
        screen.population.push(entry.clone());
        screen.best_seen = Some(entry.clone());
        screen.chairs = vec![entry.clone(), entry];
        screen.mutation_input = "0".into();
        screen.local_search.lin_kernighan = true;
        screen.start_crossing();

        screen.generate_tours(&mut SmallRng::seed_from_u64(23));

        assert_eq!(screen.population.len(), 2);
        assert!(screen.population.iter().all(|entry| entry.cost == 40));
        assert_eq!(screen.best_seen.unwrap().cost, 40);
    }
    use ::rand::SeedableRng;

    #[test]
    fn mutation_changes_identical_parents_and_preserves_the_best_tour() {
        let tsp = TSP::parser(include_str!("../data/eil51.tsp")).unwrap();
        let mut screen = InstanceScreen::new(Renderer::new(Map::from_tsp(tsp)));
        let mut rng = SmallRng::seed_from_u64(42);
        screen.target_size = 1;
        screen.generate_tours(&mut rng);
        let original = screen.population[0].clone();

        for percent in ["0", "100"] {
            screen.chairs = vec![original.clone(), original.clone()];
            screen.mutation_input = percent.into();
            screen.start_crossing();
            screen.generate_tours(&mut rng);

            assert_eq!(screen.population.len(), 2);
            let unchanged = screen
                .population
                .iter()
                .filter(|entry| entry.tour == original.tour)
                .count();
            if percent == "0" {
                assert_eq!(unchanged, 2);
            } else {
                assert!(unchanged < 2);
            }
            let best = screen.best_seen.as_ref().unwrap();
            assert!(best.cost <= original.cost);
            assert!(
                screen
                    .population
                    .iter()
                    .any(|entry| entry.tour == best.tour)
            );
            assert_eq!(
                screen.min_cost,
                screen
                    .population
                    .iter()
                    .map(|entry| entry.cost)
                    .min()
                    .unwrap()
            );
            assert_eq!(
                screen.max_cost,
                screen
                    .population
                    .iter()
                    .map(|entry| entry.cost)
                    .max()
                    .unwrap()
            );
        }
    }

    #[test]
    fn iteration_runs_requested_cycles_and_keeps_historical_costs() {
        let tsp = TSP::parser(include_str!("../data/eil51.tsp")).unwrap();
        let mut screen = InstanceScreen::new(Renderer::new(Map::from_tsp(tsp)));
        let mut rng = SmallRng::seed_from_u64(21);
        screen.target_size = 5;
        screen.generate_tours(&mut rng);
        let initial_best = screen.best_seen.as_ref().unwrap().cost;
        let initial_worst = screen.worst_seen;
        screen.rounds_input = "3".into();
        screen.iterations_input = "3".into();
        screen.start_iteration();

        let mut crossed = 0;
        for _ in 0..10 {
            screen.generate_tours(&mut rng);
            screen.run_tournaments(&mut rng);
            screen.advance_iteration();
            crossed += usize::from(!screen.parents.is_empty());
            if screen.iteration.is_none() {
                break;
            }
        }

        assert!(screen.iteration.is_none());
        assert_eq!(crossed, 3);
        assert_eq!(screen.population.len(), 6);
        assert!(screen.best_seen.as_ref().unwrap().cost <= initial_best);
        assert!(screen.worst_seen >= initial_worst);
    }
    #[test]
    fn tournaments_fill_chairs_and_cross_ordered_pairs() {
        let tsp = TSP::parser(include_str!("../data/eil51.tsp")).unwrap();
        let cities = tsp.len;
        let mut screen = InstanceScreen::new(Renderer::new(Map::from_tsp(tsp)));
        let mut rng = SmallRng::seed_from_u64(42);
        screen.target_size = 5;
        screen.generate_tours(&mut rng);
        let cheapest = screen
            .population
            .iter()
            .map(|entry| entry.cost)
            .min()
            .unwrap();

        screen.tournament_input = "5".into();
        screen.start_tournaments(Some(1));
        screen.run_tournaments(&mut rng);
        assert_eq!(screen.chairs[0].cost, cheapest);
        assert!(screen.population.iter().all(|entry| entry.selected));

        screen.tournament_input = "3".into();
        screen.start_tournaments(Some(1));
        screen.run_tournaments(&mut rng);
        assert_eq!(
            screen
                .population
                .iter()
                .filter(|entry| entry.selected)
                .count(),
            3
        );

        screen.start_tournaments(Some(2));
        screen.run_tournaments(&mut rng);
        assert_eq!(screen.chairs.len(), 4);

        screen.start_crossing();
        screen.generate_tours(&mut rng);
        assert_eq!(screen.population.len(), 12);
        assert!(screen.chairs.is_empty());

        for entry in &screen.population {
            let mut tour = entry.tour.to_vec();
            tour.sort_unstable();
            assert!(tour.into_iter().eq(0..cities as u32));
            assert_eq!(entry.cost, screen.renderer.map.tsp.tour_cost(&entry.tour));
        }
    }
}
