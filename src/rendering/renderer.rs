use super::*;

use macroquad::prelude::*;

use super::view::*;
use crate::problem::{
    Tour, TSP,
};

#[derive(Debug, Clone)]
pub struct Map {
    pub tsp: TSP,
    pub points: Vec<(f32, f32)>,
    // pub tour       : Tour      ,
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
            tsp, points,
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

    pub fn draw(&self) {
        let map = &self.map;

        self.draw_points(map);
        self.draw_tour(map);
        self.draw_route_points(map);
        self.draw_info(map);
    }

    fn draw_points(&self, map: &Map) {
        for p in &map.points {
            let (x, y) = self.view.to_screen(p.0, p.1);

            draw_circle_lines(x, y, 1.0, 1.0, WHITE);
        }
    }

    // TODO!()
    fn draw_tour(&self, map: &Map) {
        // if map.tour.is_empty() {
        //     return;
        // }

        // let mut full_route = Vec::with_capacity(map.tour.len() as usize + 2);

        // // TODO: change this shit!!
        // full_route.push(0);
        // full_route.extend_from_slice(&map.tour);
        // full_route.push(1);
        // // WHY REALLY COPY IT!!??

        // for pair in full_route.windows(2) {
        //     let (i, j) = (pair[0], pair[1]);
        //     let pl = &map.points[i as usize];
        //     let pr = &map.points[j as usize];

        //     let (xl, yl) = self.view.to_screen(pl.x, pl.y);
        //     let (xr, yr) = self.view.to_screen(pr.x, pr.y);

        //     draw_line(xl, yl, xr, yr, 3.0, BLUE);
        // }
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

    fn draw_info(&self, map: &Map) {
        let font_size = 24.0;

        let title_dims = measure_text(&map.tsp.name, None, font_size as u16, 1.0);
        let title_x = (screen_width() - title_dims.width) / 2.0;
        draw_text(&map.tsp.name, title_x, 30.0, font_size, WHITE);

        // let cost_text = format!("{:.2} / {:.2}", map.used_cost, map.tmax);
        // let cost_dims = measure_text(&cost_text, None, 20, 1.0);
        // let cost_x = screen_width() - cost_dims.width - 20.0;

        // draw_text(&cost_text, cost_x, screen_height() - 50.0, 20.0, cost_color);
    }
}


