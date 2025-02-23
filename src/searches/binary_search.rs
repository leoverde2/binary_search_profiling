use std::hint::black_box;

use crate::query::bench_search::{SearchScheme, Searchable};

pub struct SortedVec{
    pub vals: Vec<u32>,
}

impl Searchable for SortedVec{
    fn new(sorted_vals: &[u32]) -> Self {
        SortedVec{vals: sorted_vals.to_vec()}
    }

    fn get_funcs<'a>() -> &'a [&'static dyn SearchScheme<Self>] {
        &[&Self::binary_search, &Self::binary_search_branchless, &Self::std_binary_search]
    }
}

impl SortedVec{
    pub fn get(&self, index: usize) -> u32 {
        unsafe { *self.vals.get_unchecked(index) }
    }

    #[inline(never)]
    fn std_binary_search(&self, num: u32) -> u32{
        self.vals.binary_search(&num).unwrap_or_default() as u32
    }

    #[inline(never)]
    fn binary_search(&self, num: u32) -> u32{
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
    fn binary_search_branchless(&self, num: u32) -> u32{
        let mut base = 0;
        let mut len = self.vals.len();

        while len > 1 {
            let half = len / 2;
            let condition = unsafe {(self.vals.get_unchecked(base + half - 1) < &num) as usize};
            base += condition * half;
            len -= half;
        }

        black_box(self.get(base))
    }
}

