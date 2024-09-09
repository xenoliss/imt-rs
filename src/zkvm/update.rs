use std::num::NonZeroU64;

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::{
    imt::{node::IMTNode, Hashor, Key, Value},
    Hash,
};

use super::{imt_root, node_exists};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IMTUpdate<K, V> {
    pub old_root: Hash,
    pub size: NonZeroU64,
    pub node: IMTNode<K, V>,
    pub node_siblings: Vec<Option<Hash>>,
    pub new_value: V,
}

impl<K: Key, V: Value> IMTUpdate<K, V> {
    /// Verifies the IMT update and return the new updated root.
    ///
    /// Before performing the update, the state is checked to make sure it is coherent.
    pub fn verify<H: Hashor>(&self, hasher_factory: fn() -> H, old_root: Hash) -> Result<Hash> {
        // Make sure the IMTMutate old_root matches the expected old_root.
        ensure!(old_root == self.old_root, "IMTMutate.old_root is stale");

        // Verify that the node to update is already in the IMT.
        ensure!(
            node_exists(
                hasher_factory,
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

        Ok(imt_root(
            hasher_factory,
            self.size,
            &updated_node,
            &self.node_siblings,
        ))
    }
}

#[cfg(test)]
mod tests {
    use tiny_keccak::Keccak;

    use crate::{imt::Imt, utils::btree_imt_storage::BTreeIMTStorage, zkvm::mutate::IMTMutate};

    #[test]
    fn test_verify_invalid_old_root() {
        // Instanciate an IMT with a few nodes.
        let storage = BTreeIMTStorage::default();
        let mut imt = Imt::new(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32]);
        imt.insert_node([2; 32], [42; 32]);
        imt.insert_node([3; 32], [42; 32]);

        // Create an IMTUpdate and call `.verify()` with a different `old_root`.
        if let IMTMutate::Update(sut) = imt.update_node([2; 32], [43; 32]) {
            let res = sut.verify(Keccak::v256, [0xff; 32]);
            assert!(matches!(res, Err(e) if e.to_string() == "IMTMutate.old_root is stale"));
        }
    }

    #[test]
    fn test_verify_node_does_not_exist() {
        // Instanciate an IMT with a few nodes.
        let storage = BTreeIMTStorage::default();
        let mut imt = Imt::new(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32]);
        imt.insert_node([2; 32], [42; 32]);
        imt.insert_node([3; 32], [42; 32]);

        // Create an IMTUpdate and call `.verify()` with a different `old_root`.
        if let IMTMutate::Update(mut sut) = imt.update_node([2; 32], [43; 32]) {
            sut.node.key = [4; 32];
            let res = sut.verify(Keccak::v256, sut.old_root);
            assert!(matches!(res, Err(e) if e.to_string() == "IMTMutate.node is not in the IMT"));
        } else {
            panic!("invalid result")
        }
    }

    #[test]
    fn test_verify() {
        // Instanciate an IMT with a few nodes.
        let storage = BTreeIMTStorage::default();
        let mut imt = Imt::new(Keccak::v256, storage);
        imt.insert_node([1; 32], [42; 32]);
        imt.insert_node([2; 32], [42; 32]);
        imt.insert_node([3; 32], [42; 32]);

        let keys = vec![[1; 32], [2; 32], [3; 32]];

        // Insert all the keys in the IMT and ensure verifying the returned `IMTInsert` succeed.
        keys.into_iter().for_each(|node_key| {
            for i in 0..=255 {
                if let IMTMutate::Update(sut) = imt.update_node(node_key, [i; 32]) {
                    let res = sut.verify(Keccak::v256, sut.old_root);
                    assert!(res.is_ok())
                } else {
                    panic!("invalid result")
                }
            }
        });
    }
}
