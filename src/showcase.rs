use ::rand::{SeedableRng, rngs::SmallRng};
use fixedbitset::FixedBitSet;
use macroquad::prelude::*;

use crate::{
    draw_button,
    genetic::{crossing, mutation, new_tour},
    local_search::LocalSearch,
    problem::{TSP, Tour, tour_edges},
    rendering::renderer::{Map, Renderer},
};

const REMOVED: Color = Color::new(1.0, 0.38, 0.40, 1.0);
const INHERITED: Color = Color::new(0.79, 0.61, 1.0, 1.0);
const ADDED: Color = Color::new(0.38, 0.94, 0.66, 1.0);
const DURATION: f64 = 5.0;

#[derive(Clone, Copy)]
enum Operation {
    Mutation,
    Crossover,
    TwoOpt,
    Relocate,
    LinKernighan,
}

impl Operation {
    const ALL: [Self; 5] = [
        Self::Mutation,
        Self::Crossover,
        Self::TwoOpt,
        Self::Relocate,
        Self::LinKernighan,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Mutation => "Mutação",
            Self::Crossover => "Cruzamento",
            Self::TwoOpt => "2-opt",
            Self::Relocate => "Relocação",
            Self::LinKernighan => "Lin–Kernighan",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Mutation => "Inverte a ordem das cidades em um trecho da tour.",
            Self::Crossover => "Copia um trecho do Pai A e completa o filho na ordem do Pai B.",
            Self::TwoOpt => "Troca duas arestas para encurtar a tour.",
            Self::Relocate => "Move uma cidade para uma posição que reduz o custo.",
            Self::LinKernighan => "Encadeia trocas enquanto o ganho acumulado justificar a busca.",
        }
    }
}

type Edge = (usize, usize);

struct Demo {
    before: Tour,
    second_parent: Option<Tour>,
    after: Tour,
    unchanged: Vec<Edge>,
    removed: Vec<Edge>,
    added: Vec<Edge>,
    inherited: Vec<Edge>,
}

impl Demo {
    fn new(tsp: &TSP, operation: Operation) -> Self {
        let mut rng = SmallRng::seed_from_u64(187);
        let mut buffer = FixedBitSet::with_capacity(tsp.len);
        for attempt in 0..32 {
            let before = new_tour(&mut rng, tsp.len);
            let mut after = before.clone();
            let mut inherited = Vec::new();
            let second_parent = match operation {
                Operation::Mutation => {
                    mutation(&mut after, &mut rng, tsp.len);
                    None
                }
                Operation::Crossover => {
                    let other = new_tour(&mut rng, tsp.len);
                    buffer.clear();
                    if let Some((start, end)) =
                        crossing(&before, &other, &mut after, &mut rng, tsp.len, &mut buffer)
                    {
                        inherited.extend(
                            before[start..=end]
                                .windows(2)
                                .map(|pair| (pair[0] as usize, pair[1] as usize)),
                        );
                    }
                    Some(other)
                }
                _ => {
                    LocalSearch {
                        two_opt: matches!(operation, Operation::TwoOpt),
                        relocate: matches!(operation, Operation::Relocate),
                        lin_kernighan: matches!(operation, Operation::LinKernighan),
                    }
                    .improve(tsp, &mut after);
                    None
                }
            };

            let old_edges = edges(&before);
            let new_edges = edges(&after);
            let removed = old_edges
                .iter()
                .copied()
                .filter(|edge| !new_edges.contains(edge))
                .collect::<Vec<_>>();
            let added = new_edges
                .iter()
                .copied()
                .filter(|edge| !old_edges.contains(edge))
                .collect::<Vec<_>>();
            let unchanged = old_edges
                .iter()
                .copied()
                .filter(|edge| new_edges.contains(edge))
                .collect();
            let shows_crossover = second_parent.as_ref().is_none_or(|other| {
                let other_edges = edges(other);
                !inherited.is_empty() && new_edges.iter().any(|edge| !other_edges.contains(edge))
            });
            if (!removed.is_empty() && shows_crossover) || attempt == 31 {
                return Self {
                    before,
                    second_parent,
                    after,
                    unchanged,
                    removed,
                    added,
                    inherited,
                };
            }
        }
        unreachable!("the last attempt always returns a demonstration")
    }
}

fn edges(tour: &Tour) -> Vec<Edge> {
    tour_edges(tour)
        .map(|(a, b)| (a.min(b), a.max(b)))
        .collect()
}

pub struct Showcase {
    renderer: Renderer,
    active: usize,
    demo: Demo,
    elapsed: f64,
}

impl Showcase {
    pub fn new() -> Self {
        let tsp = TSP::parser(
            "NAME: showcase\nTYPE: TSP\nDIMENSION: 10\nEDGE_WEIGHT_TYPE: EUC_2D\n\
             NODE_COORD_SECTION\n1 8 49\n2 19 16\n3 49 7\n4 80 17\n5 94 41\n\
             6 83 73\n7 56 91\n8 29 81\n9 40 50\n10 67 50\nEOF\n",
        )
        .expect("valid showcase coordinates");
        let demo = Demo::new(&tsp, Operation::Mutation);
        Self {
            renderer: Renderer::new(Map::from_tsp(tsp)),
            active: 0,
            demo,
            elapsed: 0.0,
        }
    }

    pub fn draw(&mut self) {
        self.elapsed = (self.elapsed + f64::from(get_frame_time())) % DURATION;
        let tab_width = ((screen_width() - 32.0) / 5.0).clamp(1.0, 152.0);
        for (index, operation) in Operation::ALL.into_iter().enumerate() {
            let bounds = Rect::new(
                12.0 + index as f32 * (tab_width + 2.0),
                56.0,
                tab_width,
                36.0,
            );
            if draw_button(bounds, operation.label()) {
                if index != self.active {
                    self.active = index;
                    self.demo = Demo::new(&self.renderer.map.tsp, operation);
                }
                self.elapsed = 0.0;
            }
            if index == self.active {
                draw_rectangle_lines(bounds.x, bounds.y, bounds.w, bounds.h, 2.0, SKYBLUE);
            }
        }
        if draw_button(Rect::new(108.0, 8.0, 110.0, 36.0), "Repetir") {
            self.elapsed = 0.0;
        }

        draw_text(
            Operation::ALL[self.active].description(),
            12.0,
            123.0,
            20.0,
            WHITE,
        );
        self.draw_routes();
    }

    fn draw_routes(&self) {
        let parents = self.demo.second_parent.is_some();
        let columns = if parents { 3.0 } else { 2.0 };
        let width = ((screen_width() - 12.0 * (columns + 1.0)) / columns).max(1.0);
        let height = (screen_height() - 202.0).clamp(54.0, 480.0);
        let card = |column: f32| Rect::new(12.0 + column * (width + 12.0), 142.0, width, height);
        let before = self.card(
            card(0.0),
            if parents { "Pai A" } else { "Antes" },
            Some(&self.demo.before),
        );
        self.renderer
            .draw_tour_preview(before, &self.demo.before, 3.0);
        for &edge in &self.demo.inherited {
            self.renderer.draw_edge(before, edge, 3.0, INHERITED, 1.0);
        }
        if let Some(other) = &self.demo.second_parent {
            let bounds = self.card(card(1.0), "Pai B", Some(other));
            self.renderer.draw_tour_preview(bounds, other, 3.0);
        }

        let time = self.elapsed as f32;
        let result = (time >= 3.0).then_some(&self.demo.after);
        let stage = if time < 1.0 {
            "Preparação"
        } else if time < 2.0 {
            "Removendo arestas"
        } else if time < 3.0 {
            "Adicionando arestas"
        } else if parents {
            "Filho"
        } else {
            "Depois"
        };
        let output = self.card(card(columns - 1.0), stage, result);
        for &edge in &self.demo.unchanged {
            self.renderer.draw_edge(output, edge, 2.0, SKYBLUE, 1.0);
        }
        if time < 2.0 {
            let alpha = 1.0 - (time - 1.0).clamp(0.0, 1.0);
            let color = if time < 1.0 {
                SKYBLUE
            } else {
                Color {
                    a: alpha,
                    ..REMOVED
                }
            };
            for &edge in &self.demo.removed {
                self.renderer.draw_edge(output, edge, 2.5, color, 1.0);
            }
        } else {
            let progress = (time - 2.0).min(1.0);
            for &edge in &self.demo.added {
                self.renderer.draw_edge(output, edge, 3.0, ADDED, progress);
            }
        }
        for &edge in &self.demo.inherited {
            self.renderer.draw_edge(output, edge, 3.0, INHERITED, 1.0);
        }
        self.renderer.draw_preview(output);

        let legend_y = card(0.0).bottom() + 26.0;
        draw_text("Removidas", 12.0, legend_y, 19.0, REMOVED);
        draw_text("Adicionadas", 116.0, legend_y, 19.0, ADDED);
        if parents {
            draw_text("Trecho do Pai A", 235.0, legend_y, 19.0, INHERITED);
        }
    }

    fn card(&self, bounds: Rect, title: &str, tour: Option<&Tour>) -> Rect {
        draw_rectangle(
            bounds.x,
            bounds.y,
            bounds.w,
            bounds.h,
            Color::from_rgba(25, 28, 34, 255),
        );
        draw_text(title, bounds.x + 12.0, bounds.y + 24.0, 20.0, WHITE);
        if let Some(tour) = tour {
            let cost = format!("Custo {}", self.renderer.map.tsp.tour_cost(tour));
            draw_text(&cost, bounds.x + 12.0, bounds.bottom() - 10.0, 18.0, WHITE);
        }
        let inset = (bounds.w * 0.10).min(26.0);
        Rect::new(
            bounds.x + inset,
            bounds.y + 30.0,
            bounds.w - 2.0 * inset,
            (bounds.h - 60.0).max(1.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operator_produces_a_visible_and_valid_example() {
        let showcase = Showcase::new();
        let tsp = &showcase.renderer.map.tsp;
        for operation in Operation::ALL {
            let demo = Demo::new(tsp, operation);
            assert!(!demo.removed.is_empty(), "{}", operation.label());
            assert_eq!(demo.removed.len(), demo.added.len());
            assert_eq!(
                demo.second_parent.is_some(),
                matches!(operation, Operation::Crossover)
            );
            if let Some(other) = &demo.second_parent {
                assert!(!demo.inherited.is_empty());
                assert!(
                    edges(&demo.after)
                        .iter()
                        .any(|edge| !edges(other).contains(edge))
                );
                for &(a, b) in &demo.inherited {
                    assert!(edges(&demo.after).contains(&(a.min(b), a.max(b))));
                }
            }
            let mut sorted = demo.after.to_vec();
            sorted.sort_unstable();
            assert!(sorted.into_iter().eq(0..tsp.len as u32));
            if matches!(
                operation,
                Operation::TwoOpt | Operation::Relocate | Operation::LinKernighan
            ) {
                assert!(tsp.tour_cost(&demo.after) < tsp.tour_cost(&demo.before));
            }
        }
    }
}
