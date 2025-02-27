use std::array::from_fn;

use rand::Fill;

use crate::{query::bench_search::{batched, Batched, SearchScheme, Searchable}, utils::prefetch_index};

use super::s_tree_node::STreeNode;

pub const NODE_LEN: usize = 16;
pub const MAX: u32 = i32::MAX as u32;


#[derive(Debug)]
pub struct STree{
    nodes: Vec<STreeNode>,
    offsets: Vec<usize>,
}

impl STree{
    pub fn blocks_needed(key_amount: usize) -> usize{
        key_amount.div_ceil(NODE_LEN)
    }

    pub fn prev_keys(key_amount: usize) -> usize{
        Self::blocks_needed(key_amount).div_ceil(NODE_LEN + 1) * NODE_LEN
    }

    pub fn height(key_amount: usize) -> usize{
        if key_amount <= NODE_LEN{
            1
        } else {
            Self::height(Self::prev_keys(key_amount)) + 1
        }
    }

    pub fn layer_size(mut key_amount: usize, h: usize, total_height: usize) -> usize {
        for _ in h..total_height - 1 {
            key_amount = Self::prev_keys(key_amount);
        }
        key_amount
    }

}

impl Searchable for STree{
    fn new(sorted_vals: &[u32]) -> Self {
        let len = sorted_vals.len();

        let height = Self::height(len);
        let layer_sizes: Vec<usize> = 
            (0..height).map(|h| Self::layer_size(len, h, height).div_ceil(NODE_LEN)).collect();

        let n_blocks = layer_sizes.iter().sum::<usize>();

        let offsets: Vec<usize> = layer_sizes.iter()
            .scan(0, |state, layer| {
                let res = *state;
                *state += layer;
                Some(res)
            })
        .collect();

        let mut nodes = vec![STreeNode{keys: [MAX; NODE_LEN]}; n_blocks];

        let leaf_layer_offset = offsets[height - 1];
        for (i, val) in sorted_vals.iter().enumerate(){
            nodes[leaf_layer_offset + i / NODE_LEN].keys[i % NODE_LEN] = *val;
        };

        if len / NODE_LEN < layer_sizes[height - 1]{
            nodes[leaf_layer_offset + len / NODE_LEN].keys[len % NODE_LEN..].fill(MAX);
        };

        for h in (0..height - 1).rev() {
            let offset = offsets[h];

            for i in 0..layer_sizes[h] * NODE_LEN{
                let node_idx = i / NODE_LEN;
                let key_idx = i % NODE_LEN;

                let mut leaf_node_idx = node_idx;
                leaf_node_idx = leaf_node_idx * (NODE_LEN + 1) + 1 + key_idx;
                for _ in h..height - 2{
                    leaf_node_idx *= NODE_LEN + 1;
                }

                nodes[offset + node_idx].keys[key_idx] = if leaf_node_idx * NODE_LEN < len {
                    nodes[leaf_layer_offset + leaf_node_idx].keys[0]
                } else {
                    MAX
                };
            };
        };

        Self {offsets, nodes}
    }

    fn get_funcs() -> Vec<&'static dyn SearchScheme<Self>> {
        let batch_2 = Box::leak(Box::new(batched(Self::batch::<2>)));
        let batch_4 = Box::leak(Box::new(batched(Self::batch::<4>)));
        let batch_8 = Box::leak(Box::new(batched(Self::batch::<8>)));
        let batch_16 = Box::leak(Box::new(batched(Self::batch::<16>)));
        let batch_32 = Box::leak(Box::new(batched(Self::batch::<32>)));
        let batch_64 = Box::leak(Box::new(batched(Self::batch::<64>)));
        let batch_128 = Box::leak(Box::new(batched(Self::batch::<128>)));
        let batch_128_prefetch = Box::leak(Box::new(batched(Self::batch_prefetch::<128>)));
        //vec!(&Self::search_popcnt, batch_2, batch_4, batch_8, batch_16, batch_32, batch_64, batch_128)
        vec!(batch_128, batch_128_prefetch)
    }
}

impl STree {
    fn node(&self, node_idx: usize) -> &STreeNode {
        unsafe { self.nodes.get_unchecked(node_idx) }
    }

    fn key(&self, node_idx: usize, key_idx: usize) -> u32 {
        unsafe { *self.nodes.get_unchecked(node_idx).keys.get_unchecked(key_idx) }
    }


    fn search_with_find_impl(&self, value: u32, find: impl Fn(&STreeNode, u32) -> usize) -> u32{
        let mut node_idx = 0;
        for [offset, _] in self.offsets.array_windows(){
            let jump_to = find(self.node(offset + node_idx), value);
            node_idx = node_idx * (NODE_LEN + 1) + jump_to;
        }

        let last = self.offsets.last().unwrap();
        let node = self.node(last + node_idx);
        let key_idx = find(node, value);
        self.key(last + node_idx + key_idx / NODE_LEN, key_idx % NODE_LEN)
    }

    #[inline(never)]
    fn search_linear(&self, value: u32) -> u32{
        self.search_with_find_impl(value, STreeNode::find_linear)
    }

    #[inline(never)]
    fn search_linear_count(&self, value: u32) -> u32{
        self.search_with_find_impl(value, STreeNode::find_linear_count)
    }

    #[inline(never)]
    fn search_manual_simd(&self, value: u32) -> u32 {
        self.search_with_find_impl(value, STreeNode::find_simd)
    }

    #[inline(never)]
    fn search_popcnt(&self, value: u32) -> u32 {
        #[cfg(not(target_feature = "avx2"))]
        compile_error!("AVX2 support is required to compile this program");
        self.search_with_find_impl(value, |node, val| unsafe { node.find_popcnt(val)})
    }

    #[inline(never)]
    fn batch<const P: usize>(&self, values: &[u32; P]) -> [u32; P]{
        let mut k = [0; P];
        for [o, _] in self.offsets.array_windows() {
            for i in 0..P{
                let jump_to = self.node(o + k[i]).find_popcnt(values[i]);
                k[i] = k[i] * (NODE_LEN + 1) + jump_to;
            }
        }

        let o = self.offsets.last().unwrap();
        from_fn(|i| {
            let idx = self.node(o + k[i]).find_popcnt(values[i]);
            self.key(o + k[i] + idx / NODE_LEN, idx % NODE_LEN)
        })
    }

    #[inline(never)]
    fn batch_prefetch<const P: usize>(&self, values: &[u32; P]) -> [u32; P]{
        let mut k = [0; P];
        for [o, o2] in self.offsets.array_windows() {
            for i in 0..P{
                let jump_to = self.node(o + k[i]).find_popcnt(values[i]);
                k[i] = k[i] * (NODE_LEN + 1) + jump_to;
                prefetch_index(&self.nodes, o2 + k[i])
            }
        }

        let o = self.offsets.last().unwrap();
        from_fn(|i| {
            let idx = self.node(o + k[i]).find_popcnt(values[i]);
            self.key(o + k[i] + idx / NODE_LEN, idx % NODE_LEN)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree(){
        let arr = [0, 1, 2, 3, 4, 5, 6, 7 ,8 ,9 ,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32, 33, 34, 35, 36 ,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64,65,66,67,68,69,70,71,72,73,74,75,76,77,78,79,80,81,82,83,84,85,86,87,88];
        let tree = STree::new(&arr);
        println!("{:?}", tree);
        assert_eq!(1, 2);
    }
}
