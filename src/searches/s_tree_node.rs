use super::s_tree::NODE_LEN;

#[derive(Clone, Copy, Debug)]
pub struct STreeNode{
    pub keys: [u32; NODE_LEN],
}

impl STreeNode{

    pub fn find_linear(&self, value: u32) -> usize{
        for i in 0..NODE_LEN{
            if self.keys[i] >= value{
                return i;
            }
        }
        NODE_LEN - 1
    }

}
