use std::{
    collections::{BTreeMap, HashMap},
    num::NonZeroU64,
};

use crate::{
    imt::{node::IMTNode, storage::IMTStorage, Key, Value},
    Hash,
};

#[derive(Debug, Default)]
pub struct BTreeIMTStorage<K, V> {
    size: Option<NonZeroU64>,
    nodes: BTreeMap<K, IMTNode<K, V>>,
    hashes: HashMap<u8, HashMap<u64, Hash>>,
}

impl<K, V> IMTStorage<K, V> for BTreeIMTStorage<K, V>
where
    K: Key + Ord,
    V: Value,
{
    fn get_node(&self, key: &K) -> Option<IMTNode<K, V>> {
        self.nodes.get(key).cloned()
    }

    fn get_ln_node(&self, key: &K) -> Option<IMTNode<K, V>> {
        let c = self.nodes.upper_bound(std::ops::Bound::Excluded(key));
        c.peek_prev().map(|r| r.1).cloned()
    }

    fn get_hash(&self, level: u8, index: u64) -> Option<Hash> {
        self.hashes.get(&level)?.get(&index).cloned()
    }

    fn get_size(&self) -> Option<NonZeroU64> {
        self.size
    }

    fn set_node(&mut self, node: &IMTNode<K, V>) {
        self.nodes.insert(node.key, *node);
    }

    fn set_hash(&mut self, level: u8, index: u64, hash: &Hash) {
        self.hashes.entry(level).or_default().insert(index, *hash);
    }

    fn set_size(&mut self, size: NonZeroU64) {
        self.size = Some(size)
    }
}
