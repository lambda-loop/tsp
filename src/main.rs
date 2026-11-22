

mod problem  ;
mod genetic  ;
mod evolution;
mod rendering;

use rendering::{
    renderer::*,
};

#[macroquad::main(window_conf)]
async fn main() {
    // let input = std::fs::read_to_string("data/usa13509.tsp").unwrap();
    let input
        = std::fs::read_to_string("data/rat783.tsp").unwrap();
    let tsp = problem::TSP::parser(&input).unwrap();
    let map = Map::from_tsp(tsp);
    let renderer = Renderer::new(map);
    loop {
        if macroquad::prelude::is_key_down(
            macroquad::prelude::KeyCode::Q 
        ) { break }
        macroquad::prelude::clear_background(
            macroquad::prelude::BLACK,
        );
        
        renderer.draw();
        
        macroquad::prelude::next_frame().await;
    }
}

use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "TSP".into(),
        window_width: 1000,
        window_height: 800,
        ..Default::default()
    }
}
