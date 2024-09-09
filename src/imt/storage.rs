use std::num::NonZeroU64;

use crate::Hash;

use super::node::IMTNode;

pub trait IMTStorage<K, V> {
    fn get_node(&self, key: &K) -> Option<IMTNode<K, V>>;
    fn get_ln_node(&self, key: &K) -> Option<IMTNode<K, V>>;
    fn get_hash(&self, level: u8, index: u64) -> Option<Hash>;
    fn get_size(&self) -> Option<NonZeroU64>;

    fn set_node(&mut self, node: &IMTNode<K, V>);
    fn set_hash(&mut self, level: u8, index: u64, hash: &Hash);
    fn set_size(&mut self, size: NonZeroU64);
}
