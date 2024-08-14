use node::{Hashor, IMTNode, Key, Value};

use crate::Hash;

mod insert;
mod update;

pub mod imt;
pub mod mutate;
pub mod node;

/// Computes the IMT root.
fn imt_root<H: Hashor, K: Key, V: Value>(
    hasher_factory: fn() -> H,
    size: u64,
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
            (None, Some(right)) => hasher.update_hashor(right),
            (Some(left), None) => hasher.update_hashor(left),
            (Some(left), Some(right)) => {
                hasher.update_hashor(left);
                hasher.update_hashor(right);
            }
        };

        hasher.finalize_hashor_into(&mut hash);

        index /= 2;
    }

    let mut hasher = hasher_factory();
    hasher.update_hashor(&hash);
    hasher.update_hashor(&size.to_be_bytes());
    hasher.finalize_hashor_into(&mut hash);

    hash
}

/// Returns `true` if the given `node` is part of the tree commited to in `root`.
fn node_exists<H: Hashor, K: Key, V: Value>(
    hasher_factory: fn() -> H,
    root: &Hash,
    size: u64,
    node: &IMTNode<K, V>,
    siblings: &Vec<Option<Hash>>,
) -> bool {
    *root == imt_root(hasher_factory, size, node, siblings)
}
