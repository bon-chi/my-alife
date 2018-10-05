extern crate my_alife;

use my_alife::algorithm::gray_scott::{initial_matrix, laplacian};
use my_alife::visualizer::matrix_visualizer::MatrixVisualizer;
use std::fmt::Debug;

// model parameter
const F: f32 = 0.025;
const K: f32 = 0.05;

fn main() -> Result<(), impl Debug> {
    let state = initial_matrix();
    let matrix = MatrixVisualizer::new(
        "Gray Scott",
        "res/shaders/matrix_visualizer_vertex.glsl",
        "res/shaders/matrix_visualizer_fragment.glsl",
    );
    matrix?.draw_loop(state, F, K, laplacian)
}
