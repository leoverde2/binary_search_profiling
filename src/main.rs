#![allow(unused, internal_features)]
#![feature(
    core_intrinsics,
    select_unpredictable,
    array_windows,
    portable_simd,
    array_chunks,
)]

use std::{env, path::PathBuf};

use query::bench_search::{run_exps, QueryResult};
use rand::Rng;
use searches::{binary_search::SortedVec, eytzinger::Eytzinger, s_tree::STree};

pub mod searches;
pub mod query;
pub mod utils;

#[inline(never)]
fn main() {
    let sizes = sizes();
    let vals = gen_vals(*sizes.last().unwrap());
    let mut results: Vec<QueryResult> = Vec::new();

    for &size in &sizes{
        let len = size / std::mem::size_of::<u32>();
        let vals = &vals[..len];
        let queries = get_queries();

        //run_exps::<SortedVec>(&mut results, vals, &queries, size);
        //run_exps::<Eytzinger>(&mut results, vals, &queries, size);
        run_exps::<STree>(&mut results, vals, &queries, size);
    }
    save_results(&results);
}


fn gen_vals(size: usize) -> Vec<u32> {
    let len = size / std::mem::size_of::<u32>();
    let mut vals: Vec<u32> = (0..=len)
        .map(|_| rand::rng().random_range(0..i32::MAX as u32))
        .collect();
    vals.sort_unstable();
    vals
}

fn get_queries() -> Vec<u32>{
    let end = 1_000_000_u32.next_multiple_of(256 * 3);
    (0..end)
        .map(|_| rand::rng().random_range(0..i32::MAX as u32))
        .collect()
}

pub fn sizes() -> Vec<usize>{
    let mut result = Vec::new();
    let from: usize = 4;
    let to: usize = 32;

    for b in from..to{
        let base = 1 << b;
        result.push(base);
        result.push(base * 5 / 4);
        result.push(base * 3 / 2);
        result.push(base * 7 / 4);
    }
    let base = 1 << to;
    result.push(base);
    result
}


fn save_results(results: &Vec<QueryResult>){
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).to_path_buf();
    let result_dir = manifest.join("results");
    std::fs::create_dir_all(&result_dir).unwrap();
    let f = result_dir.join("results.json");
    let f = std::fs::File::create(f).unwrap();
    serde_json::to_writer(f, results).unwrap();
}
