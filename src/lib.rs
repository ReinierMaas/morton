/// Convert 2D spatial coordinates to Morton z-order value
///
/// Uses bithacks as described in:
/// http://graphics.stanford.edu/~seander/bithacks.html#InterleaveBMN
#[inline]
pub const fn interleave_morton(x: u16, y: u16) -> u32 {
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

        x | (y << 1)
    }
}

/// Convert Morton z-order value to 2D spatial coordinates
///
/// Uses bithacks as described in:
/// http://stackoverflow.com/questions/4909263/how-to-efficiently-de-interleave-bits-inverse-morton
#[inline]
pub const fn deinterleave_morton(z: u32) -> (u16, u16) {
    if cfg!(target_pointer_width = "64") {
        let z = z as u64;

        let z = (z | ((z >> 1) << 32)) & 0x55555555_55555555;
        let z = (z | (z >> 1)) & 0x33333333_33333333;
        let z = (z | (z >> 2)) & 0x0F0F0F0F_0F0F0F0F;
        let z = (z | (z >> 4)) & 0x00FF00FF_00FF00FF;
        let z = (z | (z >> 8)) & 0x0000FFFF_0000FFFF;

        let x = (z & 0x00000000_0000FFFF) as u16;
        let y = ((z >> 32) & 0x00000000_0000FFFF) as u16;

        (x, y)
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

        (x, y)
    }
}

/// Private test and bench helper functions
#[doc(hidden)]
pub mod utils {
    pub fn idx_tile_tuple(xy: (u16, u16), stride: usize) -> usize {
        let (x, y) = xy;
        stride * y as usize + x as usize
    }

    pub fn idx_tile(x: usize, y: usize, stride: usize) -> usize {
        stride * y + x
    }
}
