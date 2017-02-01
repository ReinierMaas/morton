extern crate morton;
use morton::Morton;

fn main() {
    let mut backing_data = Vec::new();
    let morton32 = Morton::new(4, 8, vec![0; 32], &mut backing_data);
}