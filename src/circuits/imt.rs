use std::collections::HashMap;
use tiny_keccak::{Hasher, Keccak};

use crate::{
    circuits::{
        mutate::IMTMutate,
        node::{Hashor, IMTNode, Key, Value},
    },
    Hash,
};

#[derive(Debug)]
pub struct Imt<H: Hashor, K: Key, V: Value> {
    pub root: Hash,
    pub size: u64,
    pub depth: u8,

    hasher_factory: fn() -> H,
    nodes: HashMap<K, IMTNode<K, V>>,
    hashes: HashMap<u8, HashMap<u64, Hash>>,
}

impl<H: Hashor, K: Key, V: Value> Imt<H, K, V> {
    /// Insanciate a new IMT with the zero node.
    pub fn new(hasher_factory: fn() -> H) -> Self {
        let mut imt = Self {
            root: Default::default(),
            size: 1,
            depth: Default::default(),

            hasher_factory,
            nodes: Default::default(),
            hashes: Default::default(),
        };

        let init_node_key = K::default();
        let init_node = IMTNode {
            index: Default::default(),
            key: Default::default(),
            value: Default::default(),
            next_key: Default::default(),
        };
        imt.nodes.insert(init_node_key, init_node);
        imt.refresh_tree(&init_node_key);

        imt
    }

    /// Inserts a new (key; value) in the IMT.
    ///
    /// Returns the corresponding `IMTInsert` to use for zkVM verification.
    pub fn insert_node(&mut self, key: K, value: V) -> IMTMutate<K, V> {
        // Ensure key does not already exist in the tree.
        assert!(!self.nodes.contains_key(&key), "key conflict");

        let old_root = self.root;
        let old_size = self.size;

        // Get the ln node.
        let ln_node = self.low_nullifier(&key);
        let ln_siblings = self.siblings(&ln_node.key);

        // Update the ln node and refresh the tree.
        self.nodes
            .get_mut(&ln_node.key)
            .expect("failed to get node")
            .next_key = key;
        self.refresh_tree(&ln_node.key);

        self.size += 1;
        self.refresh_depth();

        // Create the new node.
        let node = IMTNode {
            index: old_size,
            key,
            value,
            next_key: ln_node.next_key,
        };

        // Insert the new node and refresh the tree.
        self.nodes.insert(node.key, node);
        let node_siblings = self.refresh_tree(&key);

        let updated_ln_siblings = self.siblings(&ln_node.key);

        // Return the IMTMutate insertion to use for proving.
        IMTMutate::insert(
            old_root,
            old_size,
            ln_node,
            ln_siblings,
            node,
            node_siblings,
            updated_ln_siblings,
        )
    }

    /// Updates the given `key` to `value` in the IMT.
    ///
    /// Returns the corresponding `IMTUpdate` to use for zkVM verification.
    pub fn update_node(&mut self, key: K, value: V) -> IMTMutate<K, V> {
        let old_root = self.root;

        let node = self.nodes.get_mut(&key).expect("node does not exist");
        let old_node = *node;

        node.value = value;
        let node_siblings = self.refresh_tree(&key);

        IMTMutate::update(old_root, self.size, old_node, node_siblings, value)
    }

    /// Finds the Low Nulifier node for the given `node_key`.
    pub fn low_nullifier(&self, node_key: &K) -> IMTNode<K, V> {
        let ln = self
            .nodes
            .values()
            .find(|node| node.is_ln_of(node_key))
            .expect("failed to found ln node");

        *ln
    }

    /// Returns the list of siblings for the given `node_key`.
    pub fn siblings(&self, node_key: &K) -> Vec<Option<Hash>> {
        let node = self.nodes.get(node_key).expect("node does not exist");

        let mut siblings = Vec::with_capacity(self.depth.into());
        let mut index = node.index;

        for level in 0..self.depth {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling_hash = self
                .hashes
                .get(&level)
                .and_then(|m| m.get(&sibling_index).cloned());

            siblings.push(sibling_hash);
            index /= 2;
        }

        siblings
    }

    /// Refreshes the list of hashes based on the provided `node_key` and registers the new root.
    /// Also returns the updated list of siblings for the given `node_key`.
    fn refresh_tree(&mut self, node_key: &K) -> Vec<Option<Hash>> {
        let node = self.nodes.get(node_key).expect("failed to get node");
        let mut index = node.index;

        let hasher_factory = self.hasher_factory;

        // Recompute and cache the node hash.
        let mut hash = node.hash(hasher_factory());
        self.hashes.entry(0).or_default().insert(index, hash);

        // Climb up the tree and refresh the hashes.
        let mut siblings = Vec::with_capacity(self.depth as _);
        for level in 0..self.depth {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling_hash = self
                .hashes
                .entry(level)
                .or_default()
                .get(&sibling_index)
                .cloned();

            siblings.push(sibling_hash);

            let (left, right) = if index % 2 == 0 {
                (Some(hash), sibling_hash)
            } else {
                (sibling_hash, Some(hash))
            };

            let mut hasher = hasher_factory();
            match (left, right) {
                (None, None) => unreachable!(),
                (None, Some(right)) => hasher.update(&right),
                (Some(left), None) => hasher.update(&left),
                (Some(left), Some(right)) => {
                    hasher.update(&left);
                    hasher.update(&right);
                }
            };

            hasher.finalize(&mut hash);

            index /= 2;

            self.hashes
                .entry(level + 1)
                .or_default()
                .insert(index, hash);
        }

        // Refresh the root hash.
        self.root = {
            let mut root_hash = [0; 32];

            let mut k = Keccak::v256();
            k.update(&hash);
            k.update(&self.size.to_be_bytes());
            k.finalize(&mut root_hash);

            root_hash
        };

        siblings
    }

    /// Refreshes the IMT depth to be able to store `self.size` nodes.
    fn refresh_depth(&mut self) {
        let depth = (u64::BITS - self.size.leading_zeros() - 1) as u8;
        self.depth = if self.size == (1_u64 << depth) {
            depth
        } else {
            depth + 1
        }
    }
}
