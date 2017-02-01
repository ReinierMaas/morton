#![allow(unused_features)]
#![feature(test)]
#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;

pub struct MortonChunk<'m, T: 'm> {
    morton_chunk: &'m mut [T],
    side_length: usize,
}

impl<'m, T> MortonChunk<'m, T> {
    pub fn new(morton_chunk: &mut [T], side_length: usize) -> MortonChunk<T> {
        assert!(morton_chunk.len() == side_length * side_length);
        MortonChunk {
            morton_chunk: morton_chunk,
            side_length: side_length,
        }
    }
}
pub struct Morton<'m, T: 'm> {
    morton_chunks: Vec<MortonChunk<'m, T>>,
    width: usize,
    height: usize,
    side_length: usize,
}

impl<'m, T> Morton<'m, T> {
    pub fn new(width: usize, height: usize, data: Vec<T>, backing_data: &'m mut Vec<T>) -> Morton<'m, T> {
        assert!(data.len() == width * height);
        // greatest common single digit diviser
        let mut side_length = 1;
        {
            let mut width = width;
            let mut height = height;
            while {
                width >>= 1;
                height >>= 1;
                width > 0 && height > 0
            } { side_length <<= 1; }
            // side_length is minimum of both most significant bits
            // side_length is now an upper bound of the binary gcd
        }
        while (width / side_length) * side_length < width || (height / side_length) * side_length < height  {
            side_length >>= 1;
        }
        // side_length divides width and height in side_length equal parts

        // convert data from linear to morton chunks
        // need to create the vector with nones explicitly because T is not copyable
        let mut backing_data_opt: Vec<Option<_>> = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            backing_data_opt.push(None);
        }
        for (idx, element) in data.into_iter().enumerate() {
            // calculate x and y coördinate of element
            let x = idx % width;
            let y = idx / width;
            // which location should be assigned?
            let start_index = (y / side_length) * side_length * width + (x / side_length) * side_length;
            let morton_idx = interleave_morton((x % side_length) as u32, (y % side_length) as u32) as usize;
            println!("x: {}, y: {}, start_index: {}, morton_idx: {}", x, y, start_index, morton_idx);
            backing_data_opt[start_index + morton_idx] = Some(element);
        }
        // make backing data of type T instead of Option<T>
        backing_data.extend(backing_data_opt.into_iter().map(|element| element.unwrap()));
        // split morton chunks for easy iteration
        let mut morton_chunks: Vec<MortonChunk<_>> = backing_data
            .chunks_mut(side_length * side_length)
            .map(|morton_chunk| MortonChunk::new(morton_chunk, side_length))
            .collect();
        assert!(morton_chunks.len() == (width / side_length) * (height / side_length));
        Morton {
            morton_chunks: morton_chunks,
            width: width,
            height: height,
            side_length: side_length,
        }
    }
}


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
      if cfg!(target_pointer_width = "64") {
        let z = z as u64;

        let z = (z | (z << 31)) & 0x55555555_55555555;
        let z = (z | (z >> 1)) & 0x33333333_33333333;
        let z = (z | (z >> 2)) & 0x0F0F0F0F_0F0F0F0F;
        let z = (z | (z >> 4)) & 0x00FF00FF_00FF00FF;
        let z = (z | (z >> 8)) & 0x0000FFFF_0000FFFF;

        let x = (z & 0x00000000_0000FFFF) as u32;
        let y = ((z >> 32) & 0x00000000_0000FFFF) as u32;

        (x,y)
      } else {
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
