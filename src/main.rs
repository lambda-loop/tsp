

mod problem  ;
mod genetic  ;
mod evolution;
mod rendering;

fn main() {
    // let input = std::fs::read_to_string("data/usa13509.tsp").unwrap();
    let input
        = std::fs::read_to_string("data/rat783.tsp").unwrap();
    let tsp = problem::TSP::parser(&input);
    println!("{tsp:?}");
}
