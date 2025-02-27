use std::{hint::black_box, intrinsics::{prefetch_read_data, select_unpredictable}};

use crate::{query::bench_search::{SearchScheme, Searchable}, utils::prefetch_index};

#[repr(align(64))]
pub struct SortedVec{
    pub vals: Vec<u32>,
}

impl Searchable for SortedVec{
    fn new(sorted_vals: &[u32]) -> Self {
        SortedVec{vals: sorted_vals.to_vec()}
    }

    fn get_funcs() -> Vec<&'static dyn SearchScheme<Self>> {
        //&[&Self::binary_search_normal, &Self::binary_search_branchless_prefetching, &Self::binary_search_branchless, &Self::std_binary_search, &Self::binary_search_random]
        vec!(&Self::binary_search_branchless_prefetching)
    }
}

impl SortedVec{
    pub fn get(&self, index: usize) -> u32 {
        unsafe { *self.vals.get_unchecked(index) }
    }

    #[inline(never)]
    fn std_binary_search(&self, num: u32) -> u32{
        let idx = self.vals.binary_search(&num).unwrap_or_else(|i| i);
        self.get(idx)
    }

    #[inline(never)]
    fn binary_search_random(&self, num: u32) -> u32{
        let mut l = 0;
        let mut r = self.vals.len() - 1;
        while l < r {
            let ran = rand::random_range(0..=10_000_000);
            let m = l + ran % (r - l);
            if self.vals[m] >= num{
                r = m;
            } else {
                l = m + 1;
            }
        }
        black_box(self.get(l))
    }

    #[inline(never)]
    fn binary_search_normal(&self, num: u32) -> u32{
        let mut l = 0;
        let mut r = self.vals.len() - 1;
        while l < r{
            let m = (l + r) / 2;
            if self.vals[m] >= num{
                r = m;
            }
            else {
                l = m + 1;
            }
        }
        black_box(self.get(l))
    }

    #[inline(never)]
    fn binary_search_branchless_prefetching(&self, num: u32) -> u32{
        let mut base = 0;
        let mut len = self.vals.len();

        while len > 1 {
            let half = len / 2;
            prefetch_index(&self.vals, base + half / 2 - 1);
            prefetch_index(&self.vals, base + half + half / 2 - 1);
            let cmp = self.get(base + half - 1) < num;
            base = cmp.select_unpredictable(base + half, base);
            len -= half;
        }

        black_box(self.get(base))
    }

    #[inline(never)]
    fn binary_search_branchless(&self, num: u32) -> u32{
        let mut base = 0;
        let mut len = self.vals.len();

        while len > 1 {
            let half = len / 2;
            let cmp = self.get(base + half - 1) < num;
            base = cmp.select_unpredictable(base + half, base);
            len -= half;
        }

        black_box(self.get(base))
    }
}

