use super::*;

use macroquad::prelude::*;

use super::view::*;
use crate::problem::{
    Tour, TSP, tour_edges,
};

#[derive(Debug, Clone)]
pub struct Map {
    pub tsp: TSP,
    pub points: Vec<(f32, f32)>,
    pub tour: Tour,
    // pub points     : Vec<Point>,
    // pub tmax       : f64       ,
    // pub used_cost  : f64       ,
    // pub title      : String    ,
    // pub score      : u64       ,
    // additional info
}

impl Map {
    pub fn from_tsp (
        tsp: TSP,

    ) -> Self {
        let points: Vec<(f32, f32)> = tsp.ps
            .iter()
            .map(|&(x, y)| (x as f32, y as f32))
            .collect();
        Map {
            tsp, points, tour: Box::default(),
        }
    }
}

pub struct Renderer {
    pub map: Map,
    view: View,
}

impl Renderer {
    pub fn new(map: Map) -> Self {
        let view = View::new(&map.points);


        Self { map, view }
    }

    pub fn draw(&self, bounds: Rect) {
        self.draw_tour_preview(bounds, &self.map.tour, 3.0);
        self.draw_route_points(&self.map);
    }

    pub fn draw_preview(&self, bounds: Rect) {
        self.draw_points(bounds, 2.0);
    }

    pub fn draw_tour_preview(&self, bounds: Rect, tour: &Tour, radius: f32) {
        self.draw_tour(bounds, tour);
        self.draw_points(bounds, radius);
    }

    fn draw_points(&self, bounds: Rect, radius: f32) {
        for &(x, y) in &self.map.points {
            let (x, y) = self.view.project(x, y, bounds);
            draw_circle(x, y, radius, WHITE);
        }
    }

    fn draw_tour(&self, bounds: Rect, tour: &Tour) {
        for (from, to) in tour_edges(tour) {
            self.draw_edge(bounds, (from, to), 1.5, SKYBLUE, 1.0);
        }
    }

    pub fn draw_edge(
        &self,
        bounds: Rect,
        (from, to): (usize, usize),
        thickness: f32,
        color: Color,
        progress: f32,
    ) {
        let (x1, y1) = self.map.points[from];
        let (x2, y2) = self.map.points[to];
        let (x1, y1) = self.view.project(x1, y1, bounds);
        let (x2, y2) = self.view.project(x2, y2, bounds);
        draw_line(
            x1,
            y1,
            x1 + (x2 - x1) * progress,
            y1 + (y2 - y1) * progress,
            thickness,
            color,
        );
    }

    fn draw_route_points(&self, map: &Map) {
        // for &idx in map.tour.as_slice() {
        //     let p = &map.points[idx as usize];
        //     let (x, y) = self.view.to_screen(p.x, p.y);

        //     draw_circle(x, y, 10.0, BLUE);
        //     draw_circle_lines(x, y, 12.0, 2.0, WHITE);
        // }

        // self.draw_special_point(map, 0);
        // self.draw_special_point(map, 1);
    }

    // TODO: do i really care about reusing this ?
    fn draw_special_point(&self, map: &Map, idx: u8) {
        let p = &map.points[idx as usize];
        let (x, y) = self.view.to_screen(p.0, p.1);

        draw_circle(x, y, 14.0, RED);
        draw_circle_lines(x, y, 16.0, 2.0, WHITE);
    }
}
