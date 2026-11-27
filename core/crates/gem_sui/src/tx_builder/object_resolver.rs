use crate::rpc::proto::{Object, OwnerKind};
use crate::{SuiClient, SuiError, sui_framework_package_address};
use std::{
    collections::{BTreeSet, HashMap},
    str::FromStr,
};

use super::{Argument, TransactionBuilder};
use crate::types::{Address, Digest, ObjectInput};

const COIN_OBJECT_TYPE: &str = "::coin::Coin<";

#[derive(Clone, Debug)]
pub struct ResolvedObjectInput {
    object_id: Address,
    owner: ResolvedObjectOwner,
    object_type: Option<String>,
}

#[derive(Clone, Debug)]
enum ResolvedObjectOwner {
    Shared { initial_shared_version: u64 },
    Owned { version: u64, digest: Digest },
    Immutable { version: u64, digest: Digest },
}

impl ResolvedObjectInput {
    pub async fn get_multiple(client: &SuiClient, object_ids: Vec<String>) -> Result<Vec<Self>, SuiError> {
        client.get_multiple_objects(object_ids).await.map_err(SuiError::from_display)?.into_iter().map(Self::from_rpc_object).collect()
    }

    pub(crate) fn from_rpc_object(object: Object) -> Result<Self, SuiError> {
        let object_id = object.object_id.ok_or_else(|| SuiError::invalid_input("missing Sui object id"))?;
        let owner = object.owner.ok_or_else(|| SuiError::invalid_input(format!("Sui object owner is missing: {object_id}")))?;
        let owner_kind = owner.kind();
        let object_id = Address::from_str(&object_id).map_err(|err| SuiError::invalid_input(format!("Invalid Sui object id {object_id}: {err}")))?;

        let owner = match owner_kind {
            OwnerKind::Shared => ResolvedObjectOwner::Shared {
                initial_shared_version: owner.version.ok_or_else(|| SuiError::invalid_input(format!("Sui shared object version is missing: {object_id}")))?,
            },
            OwnerKind::Address | OwnerKind::Object => ResolvedObjectOwner::Owned {
                version: object.version.ok_or_else(|| SuiError::invalid_input(format!("Sui object version is missing: {object_id}")))?,
                digest: digest(object.digest, object_id)?,
            },
            OwnerKind::Immutable => ResolvedObjectOwner::Immutable {
                version: object.version.ok_or_else(|| SuiError::invalid_input(format!("Sui object version is missing: {object_id}")))?,
                digest: digest(object.digest, object_id)?,
            },
            OwnerKind::Unknown | OwnerKind::ConsensusAddress => {
                return Err(SuiError::invalid_input(format!("Unsupported Sui object owner kind for {object_id}: {owner_kind:?}")));
            }
        };
        Ok(Self {
            object_id,
            owner,
            object_type: object.object_type,
        })
    }

    pub(crate) fn is_owned(&self) -> bool {
        match self.owner {
            ResolvedObjectOwner::Owned { .. } => true,
            ResolvedObjectOwner::Shared { .. } | ResolvedObjectOwner::Immutable { .. } => false,
        }
    }

    pub(crate) fn coin_type(&self) -> Option<&str> {
        let (package, coin_type) = self.object_type.as_deref()?.split_once(COIN_OBJECT_TYPE)?;
        (Address::from_str(package).ok()? == sui_framework_package_address()).then(|| coin_type.strip_suffix('>')).flatten()
    }

    pub fn input(&self, mutable: bool) -> ObjectInput {
        match &self.owner {
            ResolvedObjectOwner::Shared { initial_shared_version } => ObjectInput::shared(self.object_id, *initial_shared_version, mutable),
            ResolvedObjectOwner::Owned { version, digest } | ResolvedObjectOwner::Immutable { version, digest } => ObjectInput::immutable_or_owned(self.object_id, *version, *digest),
        }
    }
}

fn digest(value: Option<String>, object_id: Address) -> Result<Digest, SuiError> {
    let value = value.ok_or_else(|| SuiError::invalid_input(format!("Sui object digest is missing: {object_id}")))?;
    Digest::from_str(&value).map_err(|err| SuiError::invalid_input(format!("Invalid Sui object digest for {object_id}: {err}")))
}

pub struct ObjectResolver {
    shared_versions: HashMap<String, u64>,
}

impl ObjectResolver {
    pub async fn load(client: &SuiClient, object_ids: Vec<String>, pinned: &HashMap<String, u64>) -> Result<Self, SuiError> {
        let unique_ids: Vec<String> = object_ids.into_iter().collect::<BTreeSet<_>>().into_iter().collect();
        let missing: Vec<String> = unique_ids.iter().filter(|id| !pinned.contains_key(*id)).cloned().collect();

        let objects = if missing.is_empty() {
            Vec::new()
        } else {
            client.get_multiple_objects(missing).await.map_err(SuiError::from_display)?
        };

        let mut shared_versions: HashMap<String, u64> = objects
            .into_iter()
            .filter_map(|object| {
                let id = object.object_id?;
                object.owner.and_then(|owner| match owner.kind() {
                    OwnerKind::Shared => owner.version.map(|version| (id, version)),
                    _ => None,
                })
            })
            .collect();
        for id in &unique_ids {
            if let Some(&version) = pinned.get(id) {
                shared_versions.insert(id.clone(), version);
            }
        }
        Ok(Self { shared_versions })
    }

    pub fn initial_shared_version(&self, object_id: &str) -> Option<u64> {
        self.shared_versions.get(object_id).copied()
    }

    pub fn shared_object_input(&self, object_id: &str, mutable: bool) -> Result<ObjectInput, SuiError> {
        let version = self.shared_versions.get(object_id).copied().ok_or_else(|| SuiError::invalid_input(format!("Sui shared object was not context: {object_id}")))?;
        let address = Address::from_str(object_id).map_err(|err| SuiError::invalid_input(format!("Invalid Sui address {object_id}: {err}")))?;
        Ok(ObjectInput::shared(address, version, mutable))
    }

    pub fn shared_object(&self, txb: &mut TransactionBuilder, object_id: &str, mutable: bool) -> Result<Argument, SuiError> {
        Ok(txb.object(self.shared_object_input(object_id, mutable)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SUI_COIN_TYPE, rpc::proto::Owner};

    #[test]
    fn test_coin_type() {
        let object = |kind: i32, object_type: &str| {
            ResolvedObjectInput::from_rpc_object(Object {
                object_id: Some(format!("0x{:064x}", 1)),
                version: Some(1),
                digest: Some("11111111111111111111111111111111".to_string()),
                owner: Some(Owner {
                    kind: Some(kind),
                    address: None,
                    version: Some(1),
                }),
                object_type: Some(object_type.to_string()),
                ..Default::default()
            })
            .unwrap()
        };
        let full = object(1, &format!("0x{:064x}::coin::Coin<{SUI_COIN_TYPE}>", 2));

        assert!(full.is_owned());
        assert_eq!(full.coin_type(), Some(SUI_COIN_TYPE));
        assert_eq!(object(1, "0x2::coin::Coin<0x2::sui::SUI>").coin_type(), Some(SUI_COIN_TYPE));
        assert_eq!(object(1, "0x3::coin::Coin<0x2::sui::SUI>").coin_type(), None, "a coin type of another package is not a coin");
        assert_eq!(object(1, "0x2::kiosk::Kiosk").coin_type(), None);
        assert!(!object(3, "0x2::clock::Clock").is_owned(), "a shared object is not the wallet's");
    }
}
