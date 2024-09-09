use std::num::NonZeroU64;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    imt::{node::IMTNode, Hashor, Key, Value},
    Hash,
};

use super::{insert::IMTInsert, update::IMTUpdate};

#[derive(Debug, Deserialize, Serialize)]
pub enum IMTMutate<K, V> {
    Insert(IMTInsert<K, V>),
    Update(IMTUpdate<K, V>),
}

impl<K: Key, V: Value> IMTMutate<K, V> {
    /// Create a new IMTMutate for insertion.
    pub fn insert(
        old_root: Hash,
        old_size: NonZeroU64,
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
        size: NonZeroU64,
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

    /// Verifies the IMT mutation and return the new updated root.
    ///
    /// Before performing the mutation, the state is checked to make sure it is coherent.
    /// In case of any inconsistency, `None` is returned.
    pub fn verify<H: Hashor>(&self, hasher_factory: fn() -> H, old_root: Hash) -> Result<Hash> {
        match &self {
            IMTMutate::Insert(insert) => insert.verify(hasher_factory, old_root),
            IMTMutate::Update(update) => update.verify(hasher_factory, old_root),
        }
    }
}
