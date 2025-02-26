use rand::Fill;

use crate::query::bench_search::{SearchScheme, Searchable};

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

    fn get_funcs<'a>() -> &'a [&'a dyn SearchScheme<Self>] {
        &[&Self::search_linear]
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
        self.key(last + node_idx, key_idx)
    }

    fn search_linear(&self, value: u32) -> u32{
        self.search_with_find_impl(value, STreeNode::find_linear)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree(){
        let arr = [0, 1, 2, 3, 4, 5, 6, 7 ,8 ,9 ,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32];
        let tree = STree::new(&arr);
        println!("{:?}", tree);
    }
}
