use cmov::Cmov;

use crate::{query::bench_search::{SearchScheme, Searchable}, utils::prefetch_index};

fn search_result_to_index(idx: usize) -> usize {
    idx >> (idx.trailing_ones() + 1)
}

#[repr(align(64))]
pub struct Eytzinger {
    vals: Vec<u32>,
    num_iters: usize, 
}

impl Searchable for Eytzinger{
    fn new(sorted_vals: &[u32]) -> Self {
        let len = sorted_vals.len() + 1;
        let mut eytz_vec = vec![0; len];
        eytz_vec[0] = u32::MAX;

        fn recurse(eytz_vec: &mut Vec<u32>, sorted_vals: &[u32], k: usize, i: &mut usize) {
            if k <= sorted_vals.len(){
                recurse(eytz_vec, sorted_vals, k * 2, i);
                eytz_vec[k] = sorted_vals[*i];
                *i += 1;
                recurse(eytz_vec, sorted_vals, 2 * k + 1, i);
            }
        }

        recurse(&mut eytz_vec, sorted_vals, 1, &mut 0);
        Self{
            vals: eytz_vec,
            num_iters: len.ilog2() as usize, 
        }
    }

    fn get_funcs() -> Vec<&'static dyn SearchScheme<Self>> {
        //&[&Eytzinger::eyz_search, &Eytzinger::search_prefetch, &Eytzinger::search_branchless, &Eytzinger::search_branchless_prefetch]
        vec!(&Eytzinger::search_prefetch)
    }


}

impl Eytzinger{
    fn get(&self, index: usize) -> u32 {
        unsafe { *self.vals.get_unchecked(index) }
    }

    fn get_next_index_branchless(&self, idx: usize, q: u32) -> usize {
        let mut idx_u64 = 2 * idx as u64;
        let candidate = (2 * idx + 1) as u64;
        // the OR here is a hack; it is done to achieve the same result algorithmica does.
        // We have to do this because we're using unsigned integers and they are using signed, so they use -1 as their "value not found"
        // retval. Therefore, they can do their last check against -1 at position 0 in the vector, which always results in the comparison
        // being true.

        let in_bounds = idx < self.vals.len();
        let idx = if in_bounds { idx } else { 0 };
        idx_u64.cmovnz(&candidate, (q > self.get(idx) || !in_bounds) as u8);
        idx_u64 as usize
    }


    #[inline(never)]
    pub fn eyz_search(&self, q: u32) -> u32 {
        let mut idx = 1;
        while idx < self.vals.len() {
            idx = 2 * idx + (q > self.get(idx)) as usize;
        }
        idx = search_result_to_index(idx);
        self.get(idx)
    }

    #[inline(never)]
    pub fn search_branchless(&self, q: u32) -> u32 {
        let mut idx = 1;
        // do a constant number of iterations
        for _ in 0..self.num_iters {
            let jump_to = (q > self.get(idx)) as usize;
            idx = 2 * idx + jump_to;
        }

        // let cmp_idx = if idx < self.vals.len() { idx } else { 0 };
        idx = self.get_next_index_branchless(idx, q);
        idx = search_result_to_index(idx);
        self.get(idx)
    }

    #[inline(never)]
    pub fn search_prefetch(&self, q: u32) -> u32 {
        let mut idx = 1;
        while (1 << 4) * idx < self.vals.len() {
            idx = 2 * idx + (q > self.get(idx)) as usize;
            prefetch_index(&self.vals, (1 << 4) * idx);
        }
        while idx < self.vals.len() {
            idx = 2 * idx + (q > self.get(idx)) as usize;
        }
        idx = search_result_to_index(idx);
        self.get(idx)
    }

    #[inline(never)]
    pub fn search_branchless_prefetch(&self, q: u32) -> u32 {
        let mut idx = 1;
        let prefetch_until = self.num_iters as isize - 4_isize;
        for _ in 0..prefetch_until {
            let jump_to = (q > self.get(idx)) as usize;
            idx = 2 * idx + jump_to;
            // the extra prefetch is apparently very slow here; why?
            prefetch_index(&self.vals, (1 << 4) * idx);
        }

        for _ in prefetch_until..(self.num_iters as isize) {
            let jump_to = (q > self.get(idx)) as usize;
            idx = 2 * idx + jump_to;
        }

        idx = self.get_next_index_branchless(idx, q);
        idx = search_result_to_index(idx);
        self.get(idx)
    }

}

