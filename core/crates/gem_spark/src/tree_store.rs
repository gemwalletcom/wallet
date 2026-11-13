use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use async_trait::async_trait;
use spark_wallet::{InMemoryTreeStore, LeafPedigree, LeafSelection, Leaves, LeavesReservation, LeavesReservationId, ReservationPurpose, ReserveResult, TargetAmounts, TreeNode, TreeNodeId, TreeServiceError, TreeStore, VerifiedLeafKeys};
use tokio::sync::{Mutex, watch};

use crate::error::LightningError;
use crate::store::{LightningLeaf, LightningStore};

pub struct StoredTreeStore {
    tree: InMemoryTreeStore,
    store: Arc<dyn LightningStore>,
    stored: Mutex<HashMap<String, String>>,
}

impl StoredTreeStore {
    pub async fn open(store: Arc<dyn LightningStore>) -> Result<Self, LightningError> {
        let stored: HashMap<String, String> = store.get_leaves().await?.into_iter().map(|leaf| (leaf.id, leaf.data)).collect();
        let pedigrees: Vec<LeafPedigree> = stored.values().filter_map(|data| serde_json::from_str(data).ok()).collect();
        let leaves: Vec<TreeNode> = pedigrees.iter().map(|pedigree| pedigree.leaf.clone()).collect();
        let chains: Vec<LeafPedigree> = pedigrees.into_iter().filter(|pedigree| !pedigree.ancestors.is_empty()).collect();
        let tree = InMemoryTreeStore::new();
        tree.add_leaves(&leaves).await?;
        tree.store_ancestors(&chains).await?;
        Ok(Self { tree, store, stored: Mutex::new(stored) })
    }

    async fn snapshot(&self) -> Result<HashMap<String, String>, TreeServiceError> {
        let leaves = self.tree.get_leaves().await?;
        let leaves: Vec<TreeNode> = [leaves.available, leaves.not_available, leaves.available_missing_from_operators, leaves.reserved_for_payment, leaves.reserved_for_swap].concat();
        let ids: Vec<TreeNodeId> = leaves.iter().map(|leaf| leaf.id.clone()).collect();
        let mut chains: HashMap<TreeNodeId, Vec<TreeNode>> = self.tree.get_exit_chains(&ids).await?.into_iter().map(|pedigree| (pedigree.leaf.id, pedigree.ancestors)).collect();
        leaves
            .into_iter()
            .map(|leaf| {
                let id = leaf.id.to_string();
                let ancestors = chains.remove(&leaf.id).unwrap_or_default();
                let data = serde_json::to_string(&LeafPedigree { leaf, ancestors }).map_err(|error| TreeServiceError::Generic(error.to_string()))?;
                Ok((id, data))
            })
            .collect()
    }

    async fn save(&self) {
        let mut stored = self.stored.lock().await;
        let Ok(snapshot) = self.snapshot().await else {
            return;
        };
        let leaves: Vec<LightningLeaf> = snapshot
            .iter()
            .filter(|(id, data)| stored.get(*id) != Some(*data))
            .map(|(id, data)| LightningLeaf { id: id.clone(), data: data.clone() })
            .collect();
        let delete_ids: Vec<String> = stored.keys().filter(|id| !snapshot.contains_key(*id)).cloned().collect();
        if leaves.is_empty() && delete_ids.is_empty() {
            return;
        }
        if self.store.update_leaves(leaves, delete_ids).await.is_ok() {
            *stored = snapshot;
        }
    }
}

#[async_trait]
impl TreeStore for StoredTreeStore {
    async fn add_leaves(&self, leaves: &[TreeNode]) -> Result<(), TreeServiceError> {
        self.tree.add_leaves(leaves).await?;
        self.save().await;
        Ok(())
    }

    async fn store_ancestors(&self, pedigrees: &[LeafPedigree]) -> Result<(), TreeServiceError> {
        self.tree.store_ancestors(pedigrees).await?;
        self.save().await;
        Ok(())
    }

    async fn leaves_missing_exit_chains(&self) -> Result<Vec<TreeNodeId>, TreeServiceError> {
        self.tree.leaves_missing_exit_chains().await
    }

    async fn get_exit_chains(&self, leaf_ids: &[TreeNodeId]) -> Result<Vec<LeafPedigree>, TreeServiceError> {
        self.tree.get_exit_chains(leaf_ids).await
    }

    async fn get_leaves(&self) -> Result<Leaves, TreeServiceError> {
        self.tree.get_leaves().await
    }

    async fn get_available_balance(&self) -> Result<u64, TreeServiceError> {
        self.tree.get_available_balance().await
    }

    async fn get_verified_leaf_keys(&self) -> Result<HashMap<TreeNodeId, VerifiedLeafKeys>, TreeServiceError> {
        self.tree.get_verified_leaf_keys().await
    }

    async fn set_leaves(&self, leaves: &[TreeNode], missing_operators_leaves: &[TreeNode], refresh_started_at: SystemTime) -> Result<(), TreeServiceError> {
        self.tree.set_leaves(leaves, missing_operators_leaves, refresh_started_at).await?;
        self.save().await;
        Ok(())
    }

    async fn cancel_reservation(&self, id: &LeavesReservationId, leaves_to_keep: &[TreeNode]) -> Result<(), TreeServiceError> {
        self.tree.cancel_reservation(id, leaves_to_keep).await?;
        self.save().await;
        Ok(())
    }

    async fn finalize_reservation(&self, id: &LeavesReservationId, new_leaves: Option<&[TreeNode]>) -> Result<(), TreeServiceError> {
        self.tree.finalize_reservation(id, new_leaves).await?;
        self.save().await;
        Ok(())
    }

    async fn try_reserve_leaves(&self, target_amounts: Option<&TargetAmounts>, exact_only: bool, purpose: ReservationPurpose) -> Result<ReserveResult, TreeServiceError> {
        self.tree.try_reserve_leaves(target_amounts, exact_only, purpose).await
    }

    async fn try_reserve_leaves_by_ids(&self, leaf_ids: &[TreeNodeId], purpose: ReservationPurpose) -> Result<LeavesReservation, TreeServiceError> {
        self.tree.try_reserve_leaves_by_ids(leaf_ids, purpose).await
    }

    async fn try_select_leaves(&self, target_amounts: Option<&TargetAmounts>) -> Result<LeafSelection, TreeServiceError> {
        self.tree.try_select_leaves(target_amounts).await
    }

    async fn now(&self) -> Result<SystemTime, TreeServiceError> {
        self.tree.now().await
    }

    fn subscribe_balance_changes(&self) -> watch::Receiver<()> {
        self.tree.subscribe_balance_changes()
    }

    async fn update_reservation(&self, reservation_id: &LeavesReservationId, reserved_leaves: &[TreeNode], change_leaves: &[TreeNode]) -> Result<LeavesReservation, TreeServiceError> {
        let reservation = self.tree.update_reservation(reservation_id, reserved_leaves, change_leaves).await?;
        self.save().await;
        Ok(reservation)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};

    use spark_wallet::TreeNodeStatus;
    use spark_wallet::tree_store_tests::create_test_node_with_parent;

    use super::*;
    use crate::store::LightningPendingSend;
    use crate::testkit::MemoryLightningStore;

    #[derive(Default)]
    struct FailingStore {
        store: MemoryLightningStore,
        fails: AtomicBool,
    }

    #[async_trait]
    impl LightningStore for FailingStore {
        async fn get_leaves(&self) -> Result<Vec<LightningLeaf>, LightningError> {
            self.store.get_leaves().await
        }

        async fn update_leaves(&self, leaves: Vec<LightningLeaf>, delete_ids: Vec<String>) -> Result<(), LightningError> {
            if self.fails.load(Ordering::SeqCst) {
                return Err(LightningError::Storage("full".to_string()));
            }
            self.store.update_leaves(leaves, delete_ids).await
        }

        async fn get_pending_sends(&self) -> Result<Vec<LightningPendingSend>, LightningError> {
            self.store.get_pending_sends().await
        }

        async fn add_pending_send(&self, send: LightningPendingSend) -> Result<(), LightningError> {
            self.store.add_pending_send(send).await
        }

        async fn delete_pending_send(&self, transfer_id: String) -> Result<(), LightningError> {
            self.store.delete_pending_send(transfer_id).await
        }
    }

    #[tokio::test]
    async fn test_leaves_and_exit_chains_survive_reopen() {
        let store = Arc::new(MemoryLightningStore::default());
        let root = create_test_node_with_parent("root", None, TreeNodeStatus::Available);
        let leaf = create_test_node_with_parent("leaf", Some("root"), TreeNodeStatus::Available);
        let spent = create_test_node_with_parent("spent", None, TreeNodeStatus::Available);
        let pedigree = LeafPedigree {
            leaf: leaf.clone(),
            ancestors: vec![root.clone()],
        };

        let tree = StoredTreeStore::open(store.clone()).await.unwrap();
        tree.add_leaves(&[leaf.clone(), spent.clone()]).await.unwrap();
        tree.store_ancestors(std::slice::from_ref(&pedigree)).await.unwrap();
        let reservation = tree.try_reserve_leaves_by_ids(std::slice::from_ref(&spent.id), ReservationPurpose::Payment).await.unwrap();
        tree.finalize_reservation(&reservation.id, None).await.unwrap();

        let reopened = StoredTreeStore::open(store.clone()).await.unwrap();

        assert_eq!(reopened.get_leaves().await.unwrap().available, vec![leaf.clone()]);
        assert_eq!(reopened.get_exit_chains(std::slice::from_ref(&leaf.id)).await.unwrap()[0].ancestors, vec![root]);
        assert_eq!(store.get_leaves().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_a_failed_write_is_retried_with_the_next_change() {
        let store = Arc::new(FailingStore::default());
        let first = create_test_node_with_parent("first", None, TreeNodeStatus::Available);
        let second = create_test_node_with_parent("second", None, TreeNodeStatus::Available);
        let tree = StoredTreeStore::open(store.clone()).await.unwrap();

        store.fails.store(true, Ordering::SeqCst);
        tree.add_leaves(std::slice::from_ref(&first)).await.unwrap();

        assert_eq!(tree.get_leaves().await.unwrap().available, vec![first]);
        assert!(store.get_leaves().await.unwrap().is_empty());

        store.fails.store(false, Ordering::SeqCst);
        tree.add_leaves(std::slice::from_ref(&second)).await.unwrap();

        assert_eq!(store.get_leaves().await.unwrap().len(), 2);
    }
}
