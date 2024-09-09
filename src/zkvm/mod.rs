use std::num::NonZeroU64;

use crate::{
    imt::{node::IMTNode, Hashor, Key, Value},
    Hash,
};

mod insert;
mod update;

pub mod mutate;

/// Computes the IMT root.
fn imt_root<H: Hashor, K: Key, V: Value>(
    hasher_factory: fn() -> H,
    size: NonZeroU64,
    node: &IMTNode<K, V>,
    siblings: &Vec<Option<Hash>>,
) -> Hash {
    let mut hash = node.hash(hasher_factory());

    let mut index = node.index;
    for sibling in siblings {
        let node_hash = Some(hash);

        let (left, right) = if index % 2 == 0 {
            (&node_hash, sibling)
        } else {
            (sibling, &node_hash)
        };

        let mut hasher = hasher_factory();
        match (left, right) {
            (None, None) => unreachable!(),
            (None, Some(right)) => hasher.update(right),
            (Some(left), None) => hasher.update(left),
            (Some(left), Some(right)) => {
                hasher.update(left);
                hasher.update(right);
            }
        };

        hasher.finalize(&mut hash);

        index /= 2;
    }

    let mut hasher = hasher_factory();
    hasher.update(&hash);
    hasher.update(&size.get().to_be_bytes());
    hasher.finalize(&mut hash);

    hash
}

/// Returns `true` if the given `node` is part of the tree commited to in `root`.
fn node_exists<H: Hashor, K: Key, V: Value>(
    hasher_factory: fn() -> H,
    root: &Hash,
    size: NonZeroU64,
    node: &IMTNode<K, V>,
    siblings: &Vec<Option<Hash>>,
) -> bool {
    *root == imt_root(hasher_factory, size, node, siblings)
}
