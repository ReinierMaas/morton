#[macro_use]
extern crate bencher;

use bencher::{black_box, Bencher};
use rand::{thread_rng, Rng};

use morton::utils::{idx_tile, idx_tile_tuple};
use morton::{deinterleave_morton, interleave_morton};

fn interleave_1000(b: &mut Bencher) {
    let x = thread_rng().gen::<u16>();
    let y = thread_rng().gen::<u16>();
    b.iter(|| {
        for _ in 0..1000 {
            black_box(interleave_morton(x, y));
        }
    });
}

fn deinterleave_1000(b: &mut Bencher) {
    let morton = thread_rng().gen::<u32>();
    b.iter(|| {
        for _ in 0..1000 {
            black_box(deinterleave_morton(morton));
        }
    });
}

fn interleave_deinterleave_1000(b: &mut Bencher) {
    let x = thread_rng().gen::<u16>();
    let y = thread_rng().gen::<u16>();
    b.iter(|| {
        for _ in 0..1000 {
            black_box(deinterleave_morton(interleave_morton(x, y)));
        }
    });
}

fn deinterleave_interleave_1000(b: &mut Bencher) {
    let morton = thread_rng().gen::<u32>();
    b.iter(|| {
        for _ in 0..1000 {
            let (x, y) = deinterleave_morton(morton);
            black_box(interleave_morton(x, y));
        }
    });
}

fn horizontal_access_normal(b: &mut Bencher) {
    let mut tile_normal = vec![0; 2048 * 2048]; // 16MB allocate more then largest cache
                                                // fill tiles with some random numbers
    for y in 0..2048 {
        for x in 0..2048 {
            let random = thread_rng().gen::<u32>();
            tile_normal[idx_tile(x, y, 2048)] = random;
        }
    }
    // bench horizontal access (x direction)
    b.iter(|| {
        for y in 0..2048 {
            for x in 0..2048 {
                black_box(tile_normal[idx_tile(x, y, 2048)]);
            }
        }
    });
}

fn vertical_access_normal(b: &mut Bencher) {
    let mut tile_normal = vec![0; 2048 * 2048]; // 16MB allocate more then largest cache
                                                // fill tiles with some random numbers
    for x in 0..2048 {
        for y in 0..2048 {
            let random = thread_rng().gen::<u32>();
            tile_normal[idx_tile(x, y, 2048)] = random;
        }
    }
    // bench vertical access (y direction)
    b.iter(|| {
        for x in 0..2048 {
            for y in 0..2048 {
                black_box(tile_normal[idx_tile(x, y, 2048)]);
            }
        }
    });
}

fn morton_access_normal(b: &mut Bencher) {
    let mut tile_morton = vec![0; 2048 * 2048]; // 16MB allocate more then largest cache
                                                // fill tiles with some random numbers
    for z in 0..2048 * 2048 {
        let random = thread_rng().gen::<u32>();
        tile_morton[idx_tile_tuple(deinterleave_morton(z), 2048)] = random;
    }
    // bench horizontal access (x direction)
    b.iter(|| {
        for z in 0..2048 * 2048 {
            black_box(tile_morton[idx_tile_tuple(deinterleave_morton(z), 2048)]);
        }
    });
}

fn horizontal_access_morton(b: &mut Bencher) {
    let mut tile_morton = vec![0; 2048 * 2048]; // 16MB allocate more then largest cache
                                                // fill tiles with some random numbers
    for y in 0..2048 {
        for x in 0..2048 {
            let random = thread_rng().gen::<u32>();
            tile_morton[interleave_morton(x, y) as usize] = random;
        }
    }
    // bench horizontal access (x direction)
    b.iter(|| {
        for y in 0..2048 {
            for x in 0..2048 {
                black_box(tile_morton[interleave_morton(x, y) as usize]);
            }
        }
    });
}

fn vertical_access_morton(b: &mut Bencher) {
    let mut tile_morton = vec![0; 2048 * 2048]; // 16MB allocate more then largest cache
                                                // fill tiles with some random numbers
    for x in 0..2048 {
        for y in 0..2048 {
            let random = thread_rng().gen::<u32>();
            tile_morton[interleave_morton(x, y) as usize] = random;
        }
    }
    // bench vertical access (y direction)
    b.iter(|| {
        for x in 0..2048 {
            for y in 0..2048 {
                black_box(tile_morton[interleave_morton(x, y) as usize]);
            }
        }
    });
}

#[allow(clippy::needless_range_loop, reason = "access direction clarity")]
fn morton_access_morton(b: &mut Bencher) {
    let mut tile_morton = vec![0; 2048 * 2048]; // 16MB allocate more then largest cache
                                                // fill tiles with some random numbers
    for z in 0..2048 * 2048 {
        let random = thread_rng().gen::<u32>();
        tile_morton[z] = random;
    }
    // bench horizontal access (x direction)
    b.iter(|| {
        for z in 0..2048 * 2048 {
            black_box(tile_morton[z]);
        }
    });
}

benchmark_group!(
    benches,
    interleave_1000,
    deinterleave_1000,
    interleave_deinterleave_1000,
    deinterleave_interleave_1000,
    horizontal_access_normal,
    vertical_access_normal,
    morton_access_normal,
    horizontal_access_morton,
    vertical_access_morton,
    morton_access_morton
);
benchmark_main!(benches);
