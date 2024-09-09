use std::{fmt::Debug, num::NonZeroU64};

use node::IMTNode;
use storage::IMTStorage;
use tiny_keccak::{Hasher, Keccak};

use crate::{zkvm::mutate::IMTMutate, Hash};

pub mod node;
pub mod storage;

pub trait Hashor = Hasher;
pub trait Key = Default + Clone + Copy + Eq + std::hash::Hash + AsRef<[u8]>;
pub trait Value = Default + Clone + Copy + AsRef<[u8]>;

#[derive(Debug)]
pub struct Imt<H, S, K, V> {
    pub root: Hash,
    pub size: NonZeroU64,
    pub depth: u8,

    hasher_factory: fn() -> H,
    storage: S,

    _phantom_data_k: std::marker::PhantomData<K>,
    _phantom_data_v: std::marker::PhantomData<V>,
}

impl<H, S, K, V> Imt<H, S, K, V>
where
    H: Hashor,
    S: IMTStorage<K, V>,
    K: Key,
    V: Value,
{
    /// Instanciate a new IMT linked to the given `storage`.
    pub fn new(hasher_factory: fn() -> H, storage: S) -> Self {
        let size = storage.get_size();

        let mut imt = Self {
            root: Default::default(),
            size: size.unwrap_or(unsafe { NonZeroU64::new_unchecked(1) }),
            depth: Default::default(),
            hasher_factory,
            storage,

            _phantom_data_k: std::marker::PhantomData,
            _phantom_data_v: std::marker::PhantomData,
        };

        // If the tree was empty in storage, insert the 0 node. Else initialize its in memory fields.
        match size {
            None => {
                let init_node = IMTNode {
                    index: Default::default(),
                    key: Default::default(),
                    value: Default::default(),
                    next_key: Default::default(),
                };
                imt._set_node(&init_node);

                // Save the size (1) in storage.
                // NOTE: Don't need to refresh the depth as the depth for a size of 1 is still 0.
                imt.storage.set_size(imt.size);
            }
            Some(_) => {
                imt._refresh_depth();
                imt._refresh_root();
            }
        };

        imt
    }

    /// Inserts a new (key; value) in the IMT.
    ///
    /// Returns the corresponding `IMTInsert` to use for zkVM verification.
    pub fn insert_node(&mut self, key: K, value: V) -> IMTMutate<K, V> {
        // Ensure key does not already exist in the tree.
        assert!(self.storage.get_node(&key).is_none(), "node already exists");

        let old_root = self.root;
        let old_size = self.size;

        // Get the ln node.
        let mut ln_node = self.low_nullifier(&key);
        let ln_siblings = self._siblings(&ln_node);

        // Create the new node.
        let node = IMTNode {
            index: old_size.get(),
            key,
            value,
            next_key: ln_node.next_key,
        };

        // Update the ln node and refresh the tree.
        ln_node.next_key = key;
        self._set_node(&ln_node);

        // Increment the IMT size.
        // NOTE: Must be done prior to inserting the new node as the depth of the tree might change
        //       which has an impact on computing the inserted node siblings.
        self._increment_size();

        // Insert the new node and refresh the tree.
        let node_siblings = self._set_node(&node);
        let updated_ln_siblings = self._siblings(&ln_node);

        // NOTE: Reset the `ln_node.next_key` value before using it in IMTMutate::insert.
        // TODO: Improve this to avoid doing this hacky reset.
        ln_node.next_key = node.next_key;

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

        let mut node = self.storage.get_node(&key).expect("node does not exist");
        let old_node = node;
        node.value = value;

        let node_siblings = self._set_node(&node);

        IMTMutate::update(old_root, self.size, old_node, node_siblings, value)
    }

    /// Returns the Low Nulifier node for the given `node_key`.
    pub fn low_nullifier(&self, node_key: &K) -> IMTNode<K, V> {
        self.storage
            .get_ln_node(node_key)
            .expect("failed to found ln node")
    }

    /// Returns the list of siblings for the given `node`.
    fn _siblings(&self, node: &IMTNode<K, V>) -> Vec<Option<Hash>> {
        let mut siblings = Vec::with_capacity(self.depth.into());
        let mut index = node.index;

        for level in 0..self.depth {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling_hash = self.storage.get_hash(level, sibling_index);

            siblings.push(sibling_hash);
            index /= 2;
        }

        siblings
    }

    /// Sets the given `node` in the IMT.
    /// This also refreshes the list of hashes based on the provided `node` and as well as the IMT root.
    ///
    /// Returns the updated list of siblings for the given `node`.
    fn _set_node(&mut self, node: &IMTNode<K, V>) -> Vec<Option<Hash>> {
        self.storage.set_node(node);
        let mut index = node.index;

        let hasher_factory = self.hasher_factory;

        // Recompute and cache the node hash.
        let mut hash = node.hash(hasher_factory());
        self.storage.set_hash(0, index, &hash);

        // Climb up the tree and refresh the hashes.
        let mut siblings = Vec::with_capacity(self.depth as _);
        for level in 0..self.depth {
            let sibling_index = if index % 2 == 0 { index + 1 } else { index - 1 };
            let sibling_hash = self.storage.get_hash(level, sibling_index);
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

            self.storage.set_hash(level + 1, index, &hash);
        }

        // Refreshes the IMT root.
        self._refresh_root();

        siblings
    }

    /// Increments and stores the IMT size and recompute the correspondig depth.
    fn _increment_size(&mut self) {
        self.size = self.size.checked_add(1).expect("max size overflow");
        self.storage.set_size(self.size);

        // Refreshes the IMT depth to be able to store `self.size` nodes.
        self._refresh_depth();
    }

    /// Refreshes the IMT depth based on its current size.
    fn _refresh_depth(&mut self) {
        let depth = (u64::BITS - self.size.leading_zeros() - 1) as u8;
        self.depth = if self.size.get() == (1_u64 << depth) {
            depth
        } else {
            depth + 1
        }
    }

    /// Returns the IMT root (including its size).
    fn _refresh_root(&mut self) {
        // TODO: Is it always safe to unwrap_or_default here?
        let root = self.storage.get_hash(self.depth, 0).unwrap_or_default();

        let mut root_with_size = [0; 32];
        let mut k = Keccak::v256();
        k.update(&root);
        k.update(&self.size.get().to_be_bytes());
        k.finalize(&mut root_with_size);

        self.root = root_with_size;
    }
}
