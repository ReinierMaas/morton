use rand::{thread_rng, Rng};

use morton::utils::{idx_tile, idx_tile_tuple};
use morton::{deinterleave_morton, interleave_morton};

#[test]
fn interleave() {
    let mut tile_morton = [0; 32 * 32]; // 1024 locations
    let mut tile_normal = [0; 32 * 32]; // 1024 locations
                                        // fill tiles with same random numbers
    for x in 0..32 {
        for y in 0..32 {
            let random = thread_rng().gen::<u32>();
            tile_morton[interleave_morton(x as u16, y as u16) as usize] = random;
            tile_normal[idx_tile(x, y, 32)] = random;
        }
    }
    // check that the same random numbers are stored there
    // (morton curve did not override it's own elements)
    for x in 0..32 {
        for y in 0..32 {
            let morton = tile_morton[interleave_morton(x as u16, y as u16) as usize];
            let normal = tile_normal[idx_tile(x, y, 32)];
            assert!(morton == normal);
        }
    }
}

#[test]
fn deinterleave() {
    let mut tile_morton = [0; 32 * 32]; // 1024 locations
    let mut tile_normal = [0; 32 * 32]; // 1024 locations
                                        // fill tiles with same random numbers
    for x in 0..32 {
        for y in 0..32 {
            let random = thread_rng().gen::<u32>();
            tile_morton[interleave_morton(x as u16, y as u16) as usize] = random;
            tile_normal[idx_tile(x, y, 32)] = random;
        }
    }
    // check that the same random numbers are stored there
    // (morton curve did not override it's own elements)
    for z in 0..1024 {
        let morton = tile_morton[z];
        let normal = tile_normal[idx_tile_tuple(deinterleave_morton(z as u32), 32)];
        assert!(morton == normal);
    }
}

#[test]
fn deinterleave_interleave() {
    for z in 0..65536 {
        let (x, y) = deinterleave_morton(z);
        let morton = interleave_morton(x, y);
        assert!(morton == z);
    }
}

#[test]
fn interleave_deinterleave() {
    for x in 0..1024 {
        for y in 0..1024 {
            let morton = interleave_morton(x, y);
            let (d_x, d_y) = deinterleave_morton(morton);
            assert!(d_x == x && d_y == y);
        }
    }
}

// tests with random input
#[test]
fn rand_interleave_deinterleave_1000() {
    for _ in 0..1024 {
        let x = thread_rng().gen::<u16>();
        let y = thread_rng().gen::<u16>();
        let morton = interleave_morton(x, y);
        let (d_x, d_y) = deinterleave_morton(morton);
        assert!(d_x == x && d_y == y);
    }
}

#[test]
fn rand_deinterleave_interleave_1000() {
    for _ in 0..1024 {
        let z = thread_rng().gen::<u32>();
        let (x, y) = deinterleave_morton(z);
        let morton = interleave_morton(x, y);
        assert!(morton == z);
    }
}
