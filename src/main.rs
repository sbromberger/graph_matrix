use graph_matrix::GraphMatrix;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let edges = vec![(0u32, 1u32), (1u32, 2u32), (2u32, 3u32), (1u32, 3u32)];
    let g = GraphMatrix::from_edges(edges);
    println!("g = {}", g);
    let h: GraphMatrix<u32> = GraphMatrix::from_edge_file(Path::new(filename));
    println!("h = {}", h);
    let v = h.row(0);
    println!("row(0) of h = {:?}", v);
}
