use std::marker::PhantomData;

use cosmwasm_std::{Empty, StdResult, Storage};
use cw_storage_plus::{Key, KeyDeserialize, Path, PrimaryKey};

#[cfg(feature = "counter")]
use cosmwasm_std::StdError;
#[cfg(feature = "counter")]
use cw_storage_plus::Item;

#[cfg(feature = "iterator")]
use cosmwasm_std::Order;
#[cfg(feature = "iterator")]
use cw_storage_plus::{Bound, Prefix, Prefixer};

/// A set of non-duplicate items.
///
/// This implementation is equivalent to storing these items as keys in a `Map<T, Empty>`.
pub struct Set<'a, T> {
    namespace: &'a [u8],

    #[cfg(feature = "counter")]
    counter: Item<'a, u64>,

    item_type: PhantomData<T>,
}

#[cfg(not(feature = "counter"))]
impl<'a, T> Set<'a, T> {
    /// Create a new instance of the item set with the given namespace.
    pub const fn new(namespace: &'a str) -> Self {
        Set {
            namespace: namespace.as_bytes(),
            item_type: PhantomData,
        }
    }
}

#[cfg(feature = "counter")]
impl<'a, T> Set<'a, T> {
    /// Create a new instance of the item set with the given map and counter namespaces.
    pub const fn new(namespace: &'a str, counter_namespace: &'a str) -> Self {
        Set {
            namespace: namespace.as_bytes(),
            counter: Item::new(counter_namespace),
            item_type: PhantomData,
        }
    }

    /// Return the total amount of items in the set.
    pub fn count(&self, store: &dyn Storage) -> StdResult<u64> {
        Ok(self.counter.may_load(store)?.unwrap_or(0))
    }

    /// Increase the item count by 1.
    fn increment_count(&self, store: &mut dyn Storage) -> StdResult<()> {
        let mut count = self.counter.may_load(store)?.unwrap_or(0);
        count += 1;
        self.counter.save(store, &count)
    }

    /// Reduce the item count by 1; throw error if the current count is zero.
    fn reduce_count(&self, store: &mut dyn Storage) -> StdResult<()> {
        match self.counter.may_load(store)? {
            None | Some(0) => {
                Err(StdError::generic_err("[cw-item-set]: count cannot be reduced below zero"))
            },
            Some(mut count) => {
                count -= 1;
                self.counter.save(store, &count)
            },
        }
    }
}

impl<'a, T> Set<'a, T>
where
    T: PrimaryKey<'a> + KeyDeserialize,
{
    /// Returns the key for storing an item
    ///
    /// This is copied from
    /// https://github.com/CosmWasm/cw-plus/blob/v0.14.0/packages/storage-plus/src/map.rs#L47-L52
    pub fn key(&self, item: T) -> Path<Empty> {
        Path::new(
            self.namespace,
            &item.key().iter().map(Key::as_ref).collect::<Vec<_>>(),
        )
    }

    /// Returns `true` if the set contains an item
    pub fn contains(&self, store: &dyn Storage, item: T) -> bool {
        self.key(item).has(store)
    }

    /// Adds an item to the set. Returns whether the item was newly added.
    pub fn insert(&self, store: &mut dyn Storage, item: T) -> StdResult<bool> {
        let key = self.key(item);
        if key.has(store) {
            Ok(false)
        } else {
            key.save(store, &Empty {})?;

            #[cfg(feature = "counter")]
            self.increment_count(store)?;

            Ok(true)
        }
    }

    /// Remove an item from the set. Returns whether the item was present in the set.
    pub fn remove(&self, store: &mut dyn Storage, item: T) -> StdResult<bool> {
        let key = self.key(item);
        if key.has(store) {
            key.remove(store);

            #[cfg(feature = "counter")]
            self.reduce_count(store)?;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[cfg(feature = "iterator")]
impl<'a, T> Set<'a, T>
where
    T: PrimaryKey<'a> + KeyDeserialize,
{
    /// Copied from
    /// https://github.com/CosmWasm/cw-plus/blob/v0.14.0/packages/storage-plus/src/map.rs#L124-126
    pub fn prefix(&self, p: T::Prefix) -> Prefix<T::Suffix, Empty, T::Suffix> {
        Prefix::new(self.namespace, &p.prefix())
    }

    /// Iterates items in the set with the specified bounds and ordering.
    pub fn items<'c>(
        &self,
        store: &'c dyn Storage,
        min: Option<Bound<'a, T>>,
        max: Option<Bound<'a, T>>,
        order: Order,
    ) -> Box<dyn Iterator<Item = StdResult<T::Output>> + 'c>
    where
        T::Output: 'static,
    {
        Prefix::<T, Empty, T>::new(self.namespace, &[]).keys(store, min, max, order)
    }
}
