use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use tiny_keccak::Hasher;

use crate::Hash;

pub trait Hashor = Clone + Hasher;
pub trait Key = Default + Clone + Copy + Eq + std::hash::Hash + AsRef<[u8]>;
pub trait Value = Default + Clone + Copy + AsRef<[u8]>;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct IMTNode<K: Key, V: Value> {
    pub index: u64,
    pub key: K,
    pub value: V,
    pub next_key: K,
}

impl<K: Key, V: Value> IMTNode<K, V> {
    pub fn hash<H: Hashor>(&self, mut hasher: H) -> Hash {
        let mut h = [0u8; 32];
        // NOTE: index is intentionnaly not hashed.
        hasher.update(self.key.as_ref());
        hasher.update(self.value.as_ref());
        hasher.update(self.next_key.as_ref());

        hasher.finalize(&mut h);
        h
    }

    pub fn is_ln_of(&self, node_key: &K) -> bool {
        self.key.as_ref() < node_key.as_ref()
            && ((self.next_key.as_ref() > node_key.as_ref())
                || (*self.next_key.as_ref() == *K::default().as_ref()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiny_keccak::Keccak;

    #[test]
    fn test_hash() {
        let key = [1; 32];
        let value = [2; 32];
        let next_key = [3; 32];

        let node = IMTNode {
            index: 0,
            key,
            value,
            next_key,
        };

        let hash = node.hash(Keccak::v256());

        // Manually hash the fields to get the expected result
        let mut hasher = Keccak::v256();
        hasher.update(&key);
        hasher.update(&value);
        hasher.update(&next_key);
        let mut expected_hash = [0u8; 32];
        hasher.finalize(&mut expected_hash);

        assert_eq!(hash, expected_hash, "hashes do not match");
    }

    #[test]
    fn test_is_ln_of() {
        let mut ln_node = IMTNode {
            index: 0,
            key: [0; 32],
            value: [0; 32],
            next_key: [0; 32],
        };

        // Should true because ln_node.key < node_key && ln_node.next_key == 0
        let node_key = [5; 32];
        assert!(ln_node.is_ln_of(&node_key), "node should be ln of node_key");

        // Should return true because ln_node.key < node_key < ln_node.next_key
        ln_node.next_key = [10; 32];
        let node_key = [2; 32];
        assert!(ln_node.is_ln_of(&node_key), "node should be ln of node_key");

        // Should return false because ln_node.next_key < node_key
        let node_key = [11; 32];
        assert!(
            !ln_node.is_ln_of(&node_key),
            "node should not be ln of node_key"
        );

        // Should return false because ln_node.key > node_key
        ln_node.key = [12; 32];
        let node_key = [3; 32];
        assert!(
            !ln_node.is_ln_of(&node_key),
            "node should not be ln of node_key"
        );
    }
}
