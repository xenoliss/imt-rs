use node::{IMTNode, Key, Value};
use tiny_keccak::Hasher;

use crate::Hash;

mod insert;
mod update;

pub mod mutate;
pub mod node;

/// Computes the IMT root.
fn imt_root<H: Clone + Hasher, K: Key, V: Value>(
    mut hasher: H,
    size: u64,
    node: &IMTNode<K, V>,
    siblings: &Vec<Option<Hash>>,
) -> Hash {
    let mut hash = node.hash(hasher.clone());

    let mut index = node.index;
    for sibling in siblings {
        let node_hash = Some(hash);

        let (left, right) = if index % 2 == 0 {
            (&node_hash, sibling)
        } else {
            (sibling, &node_hash)
        };

        let mut hasher = hasher.clone();
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

    hasher.update(&hash);
    hasher.update(&size.to_be_bytes());
    hasher.finalize(&mut hash);

    hash
}

/// Returns `true` if the given `node` is part of the tree commited to in `root`.
fn node_exists<H: Clone + Hasher, K: Key, V: Value>(
    hasher: H,
    root: &Hash,
    size: u64,
    node: &IMTNode<K, V>,
    siblings: &Vec<Option<Hash>>,
) -> bool {
    *root == imt_root(hasher, size, node, siblings)
}
