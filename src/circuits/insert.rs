use serde::{Deserialize, Serialize};

use crate::Hash;

use super::{
    imt_root,
    node::{Hashor, IMTNode, Key, Value},
    node_exists,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct IMTInsert<K: Key, V: Value> {
    pub old_root: Hash,
    pub old_size: u64,
    pub ln_node: IMTNode<K, V>,
    pub ln_siblings: Vec<Option<Hash>>,

    pub node: IMTNode<K, V>,
    pub node_siblings: Vec<Option<Hash>>,
    pub updated_ln_siblings: Vec<Option<Hash>>,
}

impl<K: Key, V: Value> IMTInsert<K, V> {
    /// Apply the IMT insert and return the new updated root.
    ///
    /// Before performong the insertion, the state is checked to make sure it is coherent.
    pub fn apply<H: Hashor>(&self, hasher: H, old_root: Hash) -> Hash {
        // Make sure the IMTMutate old_root matches the expected old_root.
        assert_eq!(old_root, self.old_root, "IMTMutate.old_root is stale");

        // Verify that the provided ln node is valid.
        assert!(
            self.is_valid_ln(hasher.clone()),
            "IMTMutate.ln_node is invalid"
        );

        // Compute the updated root from the node and the updated ln node.
        let updated_ln = IMTNode {
            next_key: self.node.key,
            ..self.ln_node
        };

        let new_size: u64 = self.old_size + 1;
        let root_from_node = imt_root(hasher.clone(), new_size, &self.node, &self.node_siblings);
        let root_from_updated_ln =
            imt_root(hasher, new_size, &updated_ln, &self.updated_ln_siblings);

        // Make sure both roots are equal.
        assert_eq!(
            root_from_node, root_from_updated_ln,
            "IMTMutate.updated_ln_siblings is invalid"
        );

        root_from_node
    }

    /// Returns `true` if `self.ln_node` is a valid ln node for `self.node`.
    fn is_valid_ln<H: Hashor>(&self, hasher: H) -> bool {
        self.ln_node.is_ln_of(&self.node.key)
            && node_exists(
                hasher,
                &self.old_root,
                self.old_size,
                &self.ln_node,
                &self.ln_siblings,
            )
    }
}

#[cfg(test)]
mod tests {
    use tiny_keccak::Keccak;

    use super::*;

    #[test]
    #[should_panic(expected = "IMTMutate.old_root is stale")]
    fn test_apply_invalid_old_root() {
        let old_root = [0xff; 32];

        let ln_node = IMTNode {
            index: 0,
            key: [0; 32],
            value: [0; 32],
            next_key: [0; 32],
        };

        let node = IMTNode {
            index: 1,
            key: [1; 32],
            value: [42; 32],
            next_key: ln_node.next_key,
        };

        let imt_insert = IMTInsert {
            old_root: [0xb; 32],
            old_size: 1,
            ln_node,
            ln_siblings: vec![None],

            node,
            node_siblings: vec![None],
            updated_ln_siblings: vec![None],
        };

        imt_insert.apply(Keccak::v256(), old_root);
    }
}
