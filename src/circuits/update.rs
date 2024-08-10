use serde::{Deserialize, Serialize};

use crate::Hash;

use super::{
    imt_root,
    node::{Hashor, IMTNode, Key, Value},
    node_exists,
};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IMTUpdate<K: Key, V: Value> {
    pub old_root: Hash,
    pub size: u64,
    pub node: IMTNode<K, V>,
    pub node_siblings: Vec<Option<Hash>>,
    pub new_value: V,
}

impl<K: Key, V: Value> IMTUpdate<K, V> {
    /// Apply the IMT update and return the new updated root.
    ///
    /// Before performong the update, the state is checked to make sure it is coherent.
    pub fn apply<H: Hashor>(&self, hasher: H, old_root: Hash) -> Hash {
        // Make sure the IMTMutate old_root matches the expected old_root.
        assert_eq!(old_root, self.old_root, "IMTMutate.old_root is stale");

        // Verify that the node to update is already in the IMT.
        assert!(
            node_exists(
                hasher.clone(),
                &self.old_root,
                self.size,
                &self.node,
                &self.node_siblings
            ),
            "IMTMutate.node is not in the IMT"
        );

        // Compute the new root from the updated node.
        let updated_node = IMTNode {
            value: self.new_value,
            ..self.node
        };

        imt_root(hasher, self.size, &updated_node, &self.node_siblings)
    }
}
