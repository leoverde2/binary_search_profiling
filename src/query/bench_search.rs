use std::{hint::black_box, time::{Duration, Instant}};

pub trait Searchable: Sized{
    fn new(sorted_vals: &[u32]) -> Self;
    fn get_funcs<'a>() -> &'a [&'a dyn SearchScheme<Self>];
    fn get_name(&self) -> String{
        std::any::type_name::<Self>().to_string()
    }
}

pub trait SearchScheme<I: Searchable> {
    fn query_one(&self, searchable: &I, value: u32) -> u32;
    fn get_name(&self) -> String{
        std::any::type_name::<Self>().to_string()
    }
}

impl <I: Searchable, F: Fn(&I, u32) -> u32> SearchScheme<I> for F {
    fn query_one(&self, searchable: &I, value: u32) -> u32 {
        self(searchable, value)
    }
}



pub fn run_exps<I: Searchable>(
    results: &mut Vec<QueryResult>,
    vals: &[u32],
    queries: &[u32],
    size: usize,
) {
    let searchable = I::new(vals);
    for func in I::get_funcs(){
        let query_result = QueryResult::new(&searchable, queries, *func, size);
        results.push(query_result);
    }
}


#[derive(serde::Serialize)]
pub struct QueryResult{
    pub duration: Duration,
    pub searchable_name: String,
    pub scheme_name: String,
    // Input size in bytes
    pub size: usize,
    // Latency, or inverse throughput, per operation
    pub latency: f64,
}

impl QueryResult{
    pub fn new<I: Searchable>(
        searchable: &I,
        queries: &[u32],
        scheme: &dyn SearchScheme<I>,
        size: usize,

    ) -> Self
    {
        let now = Instant::now();

        for query in queries{
            black_box(scheme.query_one(searchable, *query));
        }

        let duration = now.elapsed();
        let latency = duration.as_nanos() as f64 / queries.len() as f64;

        let sz = size::Size::from_bytes(size);
        let sz = format!("{}", sz);

        println!("Query size: {sz:>8}");

        QueryResult{
            duration,
            searchable_name: searchable.get_name(),
            size,
            latency,
            scheme_name: scheme.get_name(),
        }
    }
}
