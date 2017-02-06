#![allow(unused_features)]
#![feature(test)]
#[cfg(test)] extern crate rand;
#[cfg(test)] extern crate test;

#[derive(Debug)]
pub struct MortonChunk<'m, T: 'm> {
    morton_chunk: &'m mut [T],
    x: usize,
    y: usize,
}

impl<'m, T> MortonChunk<'m, T> {
    pub fn new(morton_chunk: &mut [T], x: usize, y: usize) -> MortonChunk<T> {
        assert!(morton_chunk.len() == (((morton_chunk.len() as f64).sqrt() as usize) as f64).powi(2) as usize);
        MortonChunk {
            morton_chunk: morton_chunk,
            x: x,
            y: y,
        }
    }
}

//#[derive(Debug)]
//pub struct MortonChunkIterator<'m, T: 'm> {
//    morton_chunk: &'m MortonChunk<'m, T>,
//    morton_index: usize,
//}
//
//impl<'m, T> Iterator for MortonChunkIterator<'m, T> {
//    type Item = &'m T;
//    fn next(&mut self) -> Option<Self::Item> {
//        let result = if self.morton_index < self.morton_chunk.morton_chunk.len() {
//            Some(&self.morton_chunk.morton_chunk[self.morton_index])
//        } else {
//            None
//        };
//        self.morton_index += 1;
//        result
//    }
//}

#[derive(Debug)]
pub struct Morton<'m, T: 'm> {
    backing_data: std::cell::UnsafeCell<Vec<T>>, // holds the data that is used by morton chunks
    morton_chunks: Vec<MortonChunk<'m, T>>,
    width: usize,
    height: usize,
    morton_side_length: usize,
}

impl<'m, T> Morton<'m, T> {
    pub fn new(width: usize, height: usize, data: Vec<T>) -> Morton<'m, T> {
        assert!(data.len() == width * height);
        // greatest common single digit diviser
        let mut morton_side_length = 1;
        {
            let mut width = width;
            let mut height = height;
            while {
                width >>= 1;
                height >>= 1;
                width > 0 && height > 0 && morton_side_length <= (std::u16::MAX as usize) // To make sure something % morton_side_length fits in an u16
            } { morton_side_length <<= 1; }
            // morton_side_length is minimum of both most significant bits
            // morton_side_length is now an upper bound of the binary gcd
        }
        while (width / morton_side_length) * morton_side_length < width || (height / morton_side_length) * morton_side_length < height  {
            morton_side_length >>= 1;
        }
        // morton_side_length divides width and height in morton_side_length equal parts

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
            let start_index = (y / morton_side_length) * morton_side_length * width + (x / morton_side_length) * morton_side_length;
            let morton_idx = interleave_morton((x % morton_side_length) as u16, (y % morton_side_length) as u16) as usize;
            println!("x: {}, y: {}, start_index: {}, morton_idx: {}", x, y, start_index, morton_idx);
            backing_data_opt[start_index + morton_idx] = Some(element);
        }
        // make backing data of type T instead of Option<T>
        // backing data needs to be saved with the morton struct while the mut references are used by the chunks
        let backing_data: std::cell::UnsafeCell<Vec<T>> = std::cell::UnsafeCell::new(Vec::with_capacity(width * height));
        let morton_chunks: Vec<MortonChunk<T>>;
        unsafe{
            let ref mut backing_data = *backing_data.get();
            backing_data.extend(backing_data_opt.into_iter().map(|element| element.unwrap()));
            // split morton chunks for easy iteration
            let morton_width = width / morton_side_length;

            morton_chunks = backing_data
                .chunks_mut(morton_side_length * morton_side_length)
                .enumerate()
                .map(|(morton_idx, morton_chunk)| MortonChunk::new(morton_chunk, morton_idx % morton_width, morton_idx / morton_width))
                .collect();
        }
        assert!(morton_chunks.len() == (width / morton_side_length) * (height / morton_side_length));
        Morton {
            backing_data: backing_data,
            morton_chunks: morton_chunks,
            width: width,
            height: height,
            morton_side_length: morton_side_length,
        }
    }
}

//#[derive(Debug)]
//pub struct MortonIterator<'m, T: 'm> {
//    morton_chunk: &'m Morton<'m, T>,
//    morton_index: usize,
//}
//
//impl<'m, T> Iterator for MortonIterator<'m, T> {
//    type Item = &'m T;
//    fn next(&mut self) -> Option<Self::Item> {
//        let result = if self.morton_index < self.morton_chunk.morton_chunk.len() {
//            Some(&self.morton_chunk.morton_chunk[self.morton_index])
//        } else {
//            None
//        };
//        self.morton_index += 1;
//        result
//    }
//}


// http://graphics.stanford.edu/~seander/bithacks.html#InterleaveBMN
#[inline]
pub fn interleave_morton(x: u16, y: u16) -> u32 {
    if cfg!(target_pointer_width = "64") {
        let x = x as u64;
        let y = y as u64;

        let z = y << 32 | x;
        let z = (z | (z << 8)) & 0x00FF00FF_00FF00FF;
        let z = (z | (z << 4)) & 0x0F0F0F0F_0F0F0F0F;
        let z = (z | (z << 2)) & 0x33333333_33333333;
        let z = (z | (z << 1)) & 0x55555555_55555555;

        let z = z | ((z >> 32) << 1);
        z as u32
    } else {
        let x = x as u32;
        let x = (x | (x << 8)) & 0x00FF00FF;
        let x = (x | (x << 4)) & 0x0F0F0F0F;
        let x = (x | (x << 2)) & 0x33333333;
        let x = (x | (x << 1)) & 0x55555555;

        let y = y as u32;
        let y = (y | (y << 8)) & 0x00FF00FF;
        let y = (y | (y << 4)) & 0x0F0F0F0F;
        let y = (y | (y << 2)) & 0x33333333;
        let y = (y | (y << 1)) & 0x55555555;

        let z = x | (y << 1);
        z
    }
}

// http://stackoverflow.com/questions/4909263/how-to-efficiently-de-interleave-bits-inverse-morton
#[inline]
pub fn deinterleave_morton(z: u32) -> (u16, u16) {
      if cfg!(target_pointer_width = "64") {
        let z = z as u64;

        let z = (z | ((z >> 1) << 32)) & 0x55555555_55555555;
        let z = (z | (z >> 1)) & 0x33333333_33333333;
        let z = (z | (z >> 2)) & 0x0F0F0F0F_0F0F0F0F;
        let z = (z | (z >> 4)) & 0x00FF00FF_00FF00FF;
        let z = (z | (z >> 8)) & 0x0000FFFF_0000FFFF;

        let x = (z & 0x00000000_0000FFFF) as u16;
        let y = ((z >> 32) & 0x00000000_0000FFFF) as u16;

        (x,y)
      } else {
        let x = z & 0x55555555;
        let x = (x | (x >> 1)) & 0x33333333;
        let x = (x | (x >> 2)) & 0x0F0F0F0F;
        let x = (x | (x >> 4)) & 0x00FF00FF;
        let x = ((x | (x >> 8)) & 0x0000FFFF) as u16;

        let y = (z >> 1) & 0x55555555;
        let y = (y | (y >> 1)) & 0x33333333;
        let y = (y | (y >> 2)) & 0x0F0F0F0F;
        let y = (y | (y >> 4)) & 0x00FF00FF;
        let y = ((y | (y >> 8)) & 0x0000FFFF) as u16;

        (x,y)
      }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::thread_rng;

    use test::Bencher;

    fn idx_tile(x: usize, y: usize, stride: usize) -> usize { stride * y + x }
    fn idx_tile_tuple(xy: (u16,u16), stride: usize) -> usize { let (x,y) = xy; stride * y as usize + x as usize }

    #[test]
    fn test_interleave() {
        let mut tile_morton = [0;32*32]; // 1024 locations
        let mut tile_normal = [0;32*32]; // 1024 locations
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
    fn test_deinterleave() {
        let mut tile_morton = [0;32*32]; // 1024 locations
        let mut tile_normal = [0;32*32]; // 1024 locations
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
        for _ in 0..1024 {
            let x = thread_rng().gen::<u16>();
            let y = thread_rng().gen::<u16>();
            let morton = interleave_morton(x,y);
            let (d_x,d_y) = deinterleave_morton(morton);
            assert!(d_x == x && d_y == y);
        }
    }
    #[test]
    fn rand_deinterleave_interleave_1000() {
        for _ in 0..1024 {
            let z = thread_rng().gen::<u32>();
            let (x,y) = deinterleave_morton(z);
            let morton = interleave_morton(x,y);
            assert!(morton == z);
        }
    }

    // benchmarks
    #[bench]
    fn bench_interleave_1000(b: &mut Bencher) {
        let x = thread_rng().gen::<u16>();
        let y = thread_rng().gen::<u16>();
        b.iter(|| for _ in 0..1000 { test::black_box(interleave_morton(x, y)); });
    }
    #[bench]
    fn bench_deinterleave_1000(b: &mut Bencher) {
        let morton = thread_rng().gen::<u32>();
        b.iter(|| for _ in 0..1000 { test::black_box(deinterleave_morton(morton)); });
    }
    #[bench]
    fn bench_interleave_deinterleave_1000(b: &mut Bencher) {
        let x = thread_rng().gen::<u16>();
        let y = thread_rng().gen::<u16>();
        b.iter(|| for _ in 0..1000 { test::black_box(deinterleave_morton(interleave_morton(x, y))); });
    }
    #[bench]
    fn bench_deinterleave_interleave_1000(b: &mut Bencher) {
        let morton = thread_rng().gen::<u32>();
        b.iter(|| for _ in 0..1000 {
            let (x,y) = deinterleave_morton(morton);
            test::black_box(interleave_morton(x,y));
        });
    }
    #[bench]
    fn bench_horizontal_access_normal(b: &mut Bencher) {
        let mut tile_normal = vec![0;2048*2048]; // 16MB allocate more then largest cache
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
                    test::black_box(tile_normal[idx_tile(x, y, 2048)]);
                }
            }
        });
    }
    #[bench]
    fn bench_vertical_access_normal(b: &mut Bencher) {
        let mut tile_normal = vec![0;2048*2048]; // 16MB allocate more then largest cache
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
                    test::black_box(tile_normal[idx_tile(x, y, 2048) as usize]);
                }
            }
        });
    }
    #[bench]
    fn bench_morton_access_normal(b: &mut Bencher) {
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with some random numbers
        for z in 0..2048*2048 {
            let random = thread_rng().gen::<u32>();
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
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
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
                    test::black_box(tile_morton[interleave_morton(x,y) as usize]);
                }
            }
        });
    }
    #[bench]
    fn bench_vertical_access_morton(b: &mut Bencher) {
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
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
                    test::black_box(tile_morton[interleave_morton(x,y) as usize]);
                }
            }
        });
    }
    #[bench]
    fn bench_morton_access_morton(b: &mut Bencher) {
        let mut tile_morton = vec![0;2048*2048]; // 16MB allocate more then largest cache
        // fill tiles with some random numbers
        for z in 0..2048*2048 {
            let random = thread_rng().gen::<u32>();
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
