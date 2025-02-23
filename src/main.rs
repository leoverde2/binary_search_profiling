use std::{env, path::{Path, PathBuf}};

use query::bench_search::{run_exps, QueryResult};
use rand::Rng;
use searches::binary_search::SortedVec;

pub mod searches;
pub mod query;

#[inline(never)]
fn main() {
    let sizes = sizes();
    let vals = gen_vals(*sizes.last().unwrap());
    let mut results: Vec<QueryResult> = Vec::new();

    for &size in &sizes{
        let len = size / std::mem::size_of::<u32>();
        let vals = &vals[..len];
        let queries = get_queries();

        run_exps::<SortedVec>(&mut results, vals, &queries, size)
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
    (0..1_000_000)
        .map(|_| rand::rng().random_range(0..i32::MAX as u32))
        .collect()
}

pub fn sizes() -> Vec<usize>{
    let mut result = Vec::new();
    let from: usize = 4;
    let to: usize = 30;

    for b in from..=to{
        let base = 1 << b;
        result.push(base);
    }
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
