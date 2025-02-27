use std::{hint::black_box, marker::PhantomData, time::{Duration, Instant}};

pub trait Searchable: Sized{
    fn new(sorted_vals: &[u32]) -> Self;
    fn get_funcs() -> Vec<&'static dyn SearchScheme<Self>>;
    fn get_name(&self) -> String{
        std::any::type_name::<Self>().to_string()
    }
}

pub trait SearchScheme<I: Searchable> {
    fn query(&self, searchable: &I, values: &[u32]) -> Vec<u32>{
        values.iter().copied().map(|val| self.query_one(searchable, val)).collect()
    }

    fn query_one(&self, searchable: &I, value: u32) -> u32 {
        self.query(searchable, &[value])[0]
    }

    fn get_name(&self) -> String{
        std::any::type_name::<Self>().to_string()
    }
}

impl <I: Searchable, F: Fn(&I, u32) -> u32> SearchScheme<I> for F {
    fn query_one(&self, searchable: &I, value: u32) -> u32 {
        self(searchable, value)
    }
}

pub struct Batched<const P: usize, I: Searchable, F: for<'a> Fn(&'a I, &[u32; P]) -> [u32; P]>(
    F,
    PhantomData<fn(&I)>,
);


pub const fn batched<const P: usize, I: Searchable, F: for<'a> Fn(&'a I, &[u32; P]) -> [u32; P]>(
    f: F
) -> Batched<P, I, F>{
    Batched(f, PhantomData)
}

impl<const P: usize, I: Searchable, F: for<'a> Fn(&'a I, &[u32; P]) -> [u32; P]> SearchScheme<I> for Batched<P, I, F> {
    fn query(&self, searchable: &I, values: &[u32]) -> Vec<u32> {
        let it = values.array_chunks();
        assert!(
            it.remainder().is_empty(),
            "Remainder should be empty"
        );
        it.flat_map(|val| (self.0)(searchable, val)).collect()
    }
}


pub fn run_exps<I: Searchable + 'static>(
    results: &mut Vec<QueryResult>,
    vals: &[u32],
    queries: &[u32],
    size: usize,
) {
    let searchable = I::new(vals);
    for func in I::get_funcs(){
        let query_result = QueryResult::new(&searchable, queries, func, size);
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
        black_box(scheme.query(searchable, queries));

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
