#![feature(test)]
extern crate rand;
extern crate test;

// http://graphics.stanford.edu/~seander/bithacks.html#InterleaveBMN
#[inline]
pub fn interleave_morton(x: u32, y: u32) -> u32 {
    // x and y should be smaller then 16 bits otherwise it will overflow 32 bits when interleaved
    debug_assert!(x < 65536 && y < 65536, "overflow catched: x:{} < 65536, y:{} < 65536", x, y);
    let x = (x | (x << 8)) & 0x00FF00FF;
    let x = (x | (x << 4)) & 0x0F0F0F0F;
    let x = (x | (x << 2)) & 0x33333333;
    let x = (x | (x << 1)) & 0x55555555;

    let y = (y | (y << 8)) & 0x00FF00FF;
    let y = (y | (y << 4)) & 0x0F0F0F0F;
    let y = (y | (y << 2)) & 0x33333333;
    let y = (y | (y << 1)) & 0x55555555;

    let z = x | (y << 1);
    z
}

// http://stackoverflow.com/questions/4909263/how-to-efficiently-de-interleave-bits-inverse-morton
#[inline]
pub fn deinterleave_morton(z: u32) -> (u32, u32) {
      let x = z & 0x55555555;
      let x = (x | (x >> 1)) & 0x33333333;
      let x = (x | (x >> 2)) & 0x0F0F0F0F;
      let x = (x | (x >> 4)) & 0x00FF00FF;
      let x = (x | (x >> 8)) & 0x0000FFFF;

      let y = (z >> 1) & 0x55555555;
      let y = (y | (y >> 1)) & 0x33333333;
      let y = (y | (y >> 2)) & 0x0F0F0F0F;
      let y = (y | (y >> 4)) & 0x00FF00FF;
      let y = (y | (y >> 8)) & 0x0000FFFF;

      (x,y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand;
    use rand::Rng;
    use test::Bencher;

    fn idx_tile(x: usize, y: usize, stride: usize) -> usize { stride * y + x }
    fn idx_tile_tuple(xy: (u32,u32), stride: u32) -> u32 { let (x,y) = xy; stride * y + x }

    #[test]
    fn test_interleave() {
        let mut rng_xor = rand::weak_rng();

        let mut tile_morton = [0;32*32]; // 1024 locations
        let mut tile_normal = [0;32*32]; // 1024 locations
        // fill tiles with same random numbers
        for x in 0..32 {
            for y in 0..32 {
                let random = rng_xor.next_u32();
                tile_morton[interleave_morton(x as u32, y as u32) as usize] = random;
                tile_normal[idx_tile(x, y, 32)] = random;
            }
        }

        // check that the same random numbers are stored there
        // (morton curve did not override it's own elements)
        for x in 0..32 {
            for y in 0..32 {
                let morton = tile_morton[interleave_morton(x as u32, y as u32) as usize];
                let normal = tile_normal[idx_tile(x, y, 32)];
                assert!(morton == normal);
            }
        }
    }
    #[test]
    fn test_deinterleave() {
        let mut rng_xor = rand::weak_rng();

        let mut tile_morton = [0;32*32]; // 1024 locations
        let mut tile_normal = [0;32*32]; // 1024 locations

        // fill tiles with same random numbers
        for x in 0..32 {
            for y in 0..32 {
                let random = rng_xor.next_u32();
                tile_morton[interleave_morton(x as u32, y as u32) as usize] = random;
                tile_normal[idx_tile(x, y, 32)] = random;
            }
        }

        // check that the same random numbers are stored there
        // (morton curve did not override it's own elements)
        for z in 0..1024 {
            let morton = tile_morton[z];
            let normal = tile_normal[idx_tile_tuple(deinterleave_morton(z as u32), 32) as usize];
            assert!(morton == normal);
        }
    }
    #[test]
    fn deinterleave_interleave() {
        for z in 0..65536 {
            let (x,y) = deinterleave_morton(z);
            let morton = interleave_morton(x,y);
            assert!(morton == z);
        }
    }
    #[test]
    fn interleave_deinterleave() {
        for x in 0..1024 {
            for y in 0..1024 {
                let morton = interleave_morton(x,y);
                let (d_x,d_y) = deinterleave_morton(morton);
                assert!(d_x == x && d_y == y);
            }
        }
    }

    // tests with random input
    #[test]
    fn rand_interleave_deinterleave_1000() {
        let mut rng_xor = rand::weak_rng();
        for _ in 0..1024 {
            let x = rng_xor.gen_range(0,65536);
            let y = rng_xor.gen_range(0,65536);
            let morton = interleave_morton(x,y);
            let (d_x,d_y) = deinterleave_morton(morton);
            assert!(d_x == x && d_y == y);
        }
    }
    #[test]
    fn rand_deinterleave_interleave_1000() {
        let mut rng_xor = rand::weak_rng();
        for _ in 0..1024 {
            let z = rng_xor.next_u32();
            let (x,y) = deinterleave_morton(z);
            let morton = interleave_morton(x,y);
            assert!(morton == z);
        }
    }

    // benchmarks
    #[bench]
    fn bench_interleave_1000(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let x = rng_xor.gen_range(0,65536);
        let y = rng_xor.gen_range(0,65536);
        let n = test::black_box(1000);
        b.iter(|| for _ in 0..n { test::black_box(interleave_morton(x, y)); });
    }
    #[bench]
    fn bench_deinterleave_1000(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let random = rng_xor.next_u32();
        let n = test::black_box(1000);
        b.iter(|| for _ in 0..n { test::black_box(deinterleave_morton(random)); });
    }
    #[bench]
    fn bench_interleave_deinterleave_1000(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let x = rng_xor.gen_range(0,65536);
        let y = rng_xor.gen_range(0,65536);
        let n = test::black_box(1000);
        b.iter(|| for _ in 0..n { test::black_box(deinterleave_morton(interleave_morton(x, y))); });
    }
    #[bench]
    fn bench_deinterleave_interleave_1000(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let random = rng_xor.next_u32();
        let n = test::black_box(1000);
        b.iter(|| for _ in 0..n {
            let (x,y) = deinterleave_morton(random);
            test::black_box(interleave_morton(x,y));
        });
    }

    #[bench]
    fn bench_horizontal_access_normal(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let mut tile_normal = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with same random numbers
        for y in 0..2048 {
            for x in 0..2048 {
                let random = rng_xor.next_u32();
                tile_normal[idx_tile(x, y, 2048)] = random;
            }
        }
        // bench horizontal access (x direction)
        b.iter(|| {
            for y in 0..2048 {
                for x in 0..2048 {
                    test::black_box(tile_normal[idx_tile(x, y, 2048)]);
                }
            }
        });
    }
    #[bench]
    fn bench_vertical_access_normal(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let mut tile_normal = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with same random numbers
        for x in 0..2048 {
            for y in 0..2048 {
                let random = rng_xor.next_u32();
                tile_normal[idx_tile(x, y, 2048)] = random;
            }
        }
        // bench vertical access (y direction)
        b.iter(|| {
            for x in 0..2048 {
                for y in 0..2048 {
                    test::black_box(tile_normal[idx_tile(x, y, 2048) as usize]);
                }
            }
        });
    }
    #[bench]
    fn bench_morton_access_normal(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with same random numbers
        for z in 0..2048*2048 {
            let random = rng_xor.next_u32();
            tile_morton[idx_tile_tuple(deinterleave_morton(z), 2048) as usize] = random;
        }
        // bench horizontal access (x direction)
        b.iter(|| {
            for z in 0..2048*2048 {
                test::black_box(tile_morton[idx_tile_tuple(deinterleave_morton(z), 2048) as usize]);
            }
        });
    }
    #[bench]
    fn bench_horizontal_access_morton(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with same random numbers
        for y in 0..2048 {
            for x in 0..2048 {
                let random = rng_xor.next_u32();
                tile_morton[interleave_morton(x, y) as usize] = random;
            }
        }
        // bench horizontal access (x direction)
        b.iter(|| {
            for y in 0..2048 {
                for x in 0..2048 {
                    test::black_box(tile_morton[interleave_morton(x,y) as usize]);
                }
            }
        });
    }
    #[bench]
    fn bench_vertical_access_morton(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with same random numbers
        for x in 0..2048 {
            for y in 0..2048 {
                let random = rng_xor.next_u32();
                tile_morton[interleave_morton(x, y) as usize] = random;
            }
        }
        // bench vertical access (y direction)
        b.iter(|| {
            for x in 0..2048 {
                for y in 0..2048 {
                    test::black_box(tile_morton[interleave_morton(x,y) as usize]);
                }
            }
        });
    }
    #[bench]
    fn bench_morton_access_morton(b: &mut Bencher) {
        let mut rng_xor = rand::weak_rng();
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with same random numbers
        for z in 0..2048*2048 {
            let random = rng_xor.next_u32();
            tile_morton[z] = random;
        }
        // bench horizontal access (x direction)
        b.iter(|| {
            for z in 0..2048*2048 {
                test::black_box(tile_morton[z]);
            }
        });
    }
}
