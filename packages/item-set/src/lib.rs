use std::marker::PhantomData;

use cosmwasm_std::{Empty, Order, StdResult, Storage};
use cw_storage_plus::{Bound, Key, KeyDeserialize, Path, PrimaryKey, Prefix};

/// A set of non-duplicate items.
///
/// This implementation is equivalent to storing these items as keys in a `Map<T, Empty>`.
pub struct Set<'a, T> {
    namespace: &'a [u8],
    // TODO: `count` needs to be saved in the contract store instead of being a field in the struct
    count: usize,
    item_type: PhantomData<T>,
}

impl<'a, T> Set<'a, T> {
    pub const fn new(namespace: &'a str) -> Self {
        Set {
            namespace: namespace.as_bytes(),
            count: 0,
            item_type: PhantomData,
        }
    }

    pub fn namespace(&self) -> &'a [u8] {
        self.namespace
    }

    /// Returns the number of items in the set.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns `true` if the set contains no items.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl<'a, T> Set<'a, T>
where
    T: PrimaryKey<'a> + KeyDeserialize,
{
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
    pub fn insert(&mut self, store: &mut dyn Storage, item: T) -> StdResult<bool> {
        let key = self.key(item);
        let new = if key.has(store) {
            false
        } else {
            key.save(store, &Empty {})?;
            self.count += 1;
            true
        };
        Ok(new)
    }

    /// Remove an item from the set. Returns whether the item was present in the set.
    pub fn remove(&mut self, store: &mut dyn Storage, item: T) -> StdResult<bool> {
        let key = self.key(item);
        let existed = if key.has(store) {
            key.remove(store);
            self.count -= 1;
            true
        } else {
            false
        };
        Ok(existed)
    }

    /// Iterates items in the set with the specified bounds and ordering.
    ///
    /// NOTE: Should this be put behind an optional `iterator` feature?
    pub fn range<'c>(
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

    /// Deletes all items in the set.
    pub fn clear(&self, _store: &mut dyn Storage) {
        panic!("unimplemented");
    }

    /// Retains only the items specified by the predicate
    pub fn retain<F>(&self, _store: &mut dyn Storage, _pred: F)
    where
        F: FnMut(&T) -> bool,
    {
        panic!("unimplemented");
    }

    // TODO: implement union and intersection?
}
