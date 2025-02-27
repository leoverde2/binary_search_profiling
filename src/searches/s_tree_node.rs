use std::{arch::x86_64::{__m256, _mm256_movemask_epi8, _mm256_packs_epi32, _popcnt32}, mem::transmute, simd::{cmp::SimdPartialOrd, Simd}};

use super::s_tree::NODE_LEN;

#[derive(Clone, Copy, Debug)]
#[repr(align(64))]
pub struct STreeNode{
    pub keys: [u32; NODE_LEN],
}

impl STreeNode{

    #[inline(always)]
    pub fn find_linear(&self, value: u32) -> usize{
        for i in 0..NODE_LEN{
            if self.keys[i] >= value{
                return i;
            }
        }
        NODE_LEN
    }

    #[inline(always)]
    pub fn find_linear_count(&self, value: u32) -> usize{
        let mut count = 0;
        for i in 0..NODE_LEN{
            if self.keys[i] < value{
                count += 1;
            }
        }
        count
    }

    #[inline(always)]
    pub fn find_simd(&self, value: u32) -> usize {
        let data: Simd<u32, 16> = Simd::from_slice(&self.keys[0..16]);
        let val_simd = Simd::splat(value);
        let mask = val_simd.simd_le(data);
        mask.first_set().unwrap_or(16)
    }


    #[inline(always)]
    /// # Safety
    ///
    pub fn find_popcnt(&self, value: u32) -> usize{
        let low: Simd<u32, 8> = Simd::from_slice(&self.keys[0..8]);
        let high: Simd<u32, 8> = Simd::from_slice(&self.keys[8..16]);
        let value_simd = Simd::<i32, 8>::splat(value as i32);
        unsafe {
            use std::mem::transmute as t;
            let mask_low = value_simd.simd_gt(t(low));
            let mask_high = value_simd.simd_gt(t(high));
            let merged = _mm256_packs_epi32(t(mask_low), t(mask_high));
            let mask: i32 = _mm256_movemask_epi8(merged);
            _popcnt32(mask as i32) as usize / 2
        }
    }
}
