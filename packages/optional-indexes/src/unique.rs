use std::marker::PhantomData;

use cosmwasm_std::{Binary, Order, StdError, StdResult, Storage};
use cw_storage_plus::{Bound, Index, KeyDeserialize, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct UniqueRef<T> {
    pk: Binary,
    value: T,
}

/// Similar to `UniqueIndex`, but the index function returns an _optional_ index
/// key. Only saves an entry in the index map if it is `Some`.
///
/// In cw-sdk, this is used in the `ACCOUNTS` map, where smart contract accounts
/// are indexed by their labels such that we can enforce that the labels are
/// unique, while base accounts are not indexed.
pub struct OptionalUniqueIndex<IK, T, PK = ()> {
    index: fn(&T) -> Option<IK>,
    idx_map: Map<IK, UniqueRef<T>>,
    phantom: PhantomData<PK>,
}

impl<IK, T, PK> OptionalUniqueIndex<IK, T, PK> {
    pub const fn new(idx_fn: fn(&T) -> Option<IK>, idx_namespace: &'static str) -> Self {
        Self {
            index: idx_fn,
            idx_map: Map::new(idx_namespace),
            phantom: PhantomData,
        }
    }
}

impl<'a, IK, T, PK> OptionalUniqueIndex<IK, T, PK>
where
    PK: KeyDeserialize,
    IK: PrimaryKey<'a>,
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn load(&self, store: &dyn Storage, key: IK) -> StdResult<(PK::Output, T)> {
        let UniqueRef {
            pk,
            value,
        } = self.idx_map.load(store, key)?;
        let key = PK::from_slice(&pk)?;
        Ok((key, value))
    }

    pub fn may_load(&self, store: &dyn Storage, key: IK) -> StdResult<Option<(PK::Output, T)>> {
        match self.idx_map.may_load(store, key)? {
            Some(UniqueRef {
                pk,
                value,
            }) => {
                let key = PK::from_slice(&pk)?;
                Ok(Some((key, value)))
            },
            None => Ok(None),
        }
    }

    pub fn range<'c>(
        &self,
        store: &'c dyn Storage,
        min: Option<Bound<'a, IK>>,
        max: Option<Bound<'a, IK>>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<(PK::Output, T)>> + 'c>
    where
        T: 'c,
    {
        let iter = self
            .idx_map
            .range_raw(store, min, max, order)
            .map(|res| {
                let (_, item) = res?;
                let key = PK::from_slice(&item.pk)?;
                Ok((key, item.value))
            });
        Box::new(iter)
    }
}

impl<'a, IK, T, PK> Index<T> for OptionalUniqueIndex<IK, T, PK>
where
    T: Serialize + DeserializeOwned + Clone,
    IK: PrimaryKey<'a>,
{
    fn save(&self, store: &mut dyn Storage, pk: &[u8], data: &T) -> StdResult<()> {
        // only save data in idx_map if the index is `Some`
        if let Some(idx) = (self.index)(data) {
            self.idx_map.update(store, idx, |opt| {
                if opt.is_some() {
                    // TODO: return a more informative error message,
                    // e.g. what the index and associated primary keys are
                    return Err(StdError::generic_err("Violates unique constraint on index"));
                }
                Ok(UniqueRef {
                    pk: pk.into(),
                    value: data.clone(),
                })
            })?;
        }
        Ok(())
    }

    fn remove(&self, store: &mut dyn Storage, _pk: &[u8], old_data: &T) -> StdResult<()> {
        if let Some(idx) = (self.index)(old_data) {
            self.idx_map.remove(store, idx);
        }
        Ok(())
    }
}

// TODO: add tests
