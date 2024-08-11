use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Hash;

use super::{
    insert::IMTInsert,
    node::{Hashor, IMTNode, Key, Value},
    update::IMTUpdate,
};

#[derive(Debug, Deserialize, Serialize)]
pub enum IMTMutate<K: Key, V: Value> {
    Insert(IMTInsert<K, V>),
    Update(IMTUpdate<K, V>),
}

impl<K: Key, V: Value> IMTMutate<K, V> {
    /// Create a new IMTMutate for insertion.
    pub fn insert(
        old_root: Hash,
        old_size: u64,
        ln_node: IMTNode<K, V>,
        ln_siblings: Vec<Option<Hash>>,

        node: IMTNode<K, V>,
        node_siblings: Vec<Option<Hash>>,
        updated_ln_siblings: Vec<Option<Hash>>,
    ) -> Self {
        Self::Insert(IMTInsert {
            old_root,
            old_size,
            ln_node,
            ln_siblings,
            node,
            node_siblings,
            updated_ln_siblings,
        })
    }

    /// Create a new IMTMutate for udpate.
    pub fn update(
        old_root: Hash,
        size: u64,
        node: IMTNode<K, V>,
        node_siblings: Vec<Option<Hash>>,
        new_value: V,
    ) -> Self {
        Self::Update(IMTUpdate {
            old_root,
            size,
            node,
            node_siblings,
            new_value,
        })
    }

    /// Apply the IMT mutation and return the new updated root.
    ///
    /// Before performong the mutation, the state is checked to make sure it is coherent.
    /// In case of any inconsistency, `None` is returned.
    pub fn apply<H: Hashor>(&self, hasher: H, old_root: Hash) -> Result<Hash> {
        match &self {
            IMTMutate::Insert(insert) => insert.apply(hasher, old_root),
            IMTMutate::Update(update) => update.apply(hasher, old_root),
        }
    }
}
