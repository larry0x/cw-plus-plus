#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::marker::PhantomData;

#[cfg(feature = "iterator")]
use cosmwasm_std::Order;
#[cfg(feature = "counter")]
use cosmwasm_std::StdError;
use cosmwasm_std::{Empty, StdResult, Storage};
#[cfg(feature = "counter")]
use cw_storage_plus::Item;
#[cfg(feature = "iterator")]
use cw_storage_plus::{Bound, Prefix, Prefixer};
use cw_storage_plus::{Key, KeyDeserialize, Path, PrimaryKey};

/// A set of non-duplicate items.
///
/// On a high level, a `Set<T>` is equivalent to a `Map<T, Empty>`, but offers
/// a more intuitive API similar to native HashSet and BTreeSet.
pub struct Set<'a, T> {
    namespace: &'a [u8],

    #[cfg(feature = "counter")]
    counter: Item<u64>,

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
    pub const fn new(namespace: &'a str, counter_namespace: &'static str) -> Self {
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
    fn key(&self, item: T) -> Path<Empty> {
        Path::new(self.namespace, &item.key().iter().map(Key::as_ref).collect::<Vec<_>>())
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
    /// Copied from cw-storage-plus:
    /// https://github.com/CosmWasm/cw-storage-plus/blob/v0.16.0/src/map.rs#L55-L57
    fn no_prefix_raw(&self) -> Prefix<Vec<u8>, Empty, T> {
        Prefix::new(self.namespace, &[])
    }

    /// Access items in the set under the given prefix.\
    ///
    /// Copied from cw-storage-plus:
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

    /// Delete all elements from the set.
    ///
    /// Copied from cw-storage-plus:
    /// https://github.com/CosmWasm/cw-storage-plus/blob/v0.16.0/src/map.rs#L115-L132
    pub fn clear(&self, store: &mut dyn Storage) {
        const TAKE: usize = 10;
        let mut cleared = false;

        while !cleared {
            let paths = self
                .no_prefix_raw()
                .keys_raw(store, None, None, Order::Ascending)
                .map(|raw_key| Path::<Empty>::new(self.namespace, &[raw_key.as_slice()]))
                .take(TAKE)
                .collect::<Vec<_>>();

            for path in &paths {
                store.remove(path);
            }

            cleared = paths.len() < TAKE;
        }

        #[cfg(feature = "counter")]
        self.counter.remove(store);
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[cfg(feature = "iterator")]
    use std::ops::Range;

    use cosmwasm_std::testing::MockStorage;

    use super::*;

    const NAMESPACE: &str = "names";

    #[cfg(not(feature = "counter"))]
    const NAMES: Set<&str> = Set::new(NAMESPACE);
    #[cfg(feature = "counter")]
    const NAMES: Set<&str> = Set::new(NAMESPACE, "names__counter");

    #[cfg(all(feature = "iterator", not(feature = "counter")))]
    const TUPLES: Set<(u64, &str)> = Set::new("tuples");
    #[cfg(all(feature = "iterator", feature = "counter"))]
    const TUPLES: Set<(u64, &str)> = Set::new("tuples", "tuples__counter");

    /// Returns the raw storage key for a given name.
    /// The key is: length of namespace (2 bytes) + namespace + the name
    fn key(name: &str) -> Vec<u8> {
        let length_bytes = (NAMESPACE.len() as u32).to_be_bytes();
        let mut out = Vec::with_capacity(2 + NAMESPACE.len() + name.len());
        out.extend_from_slice(&[length_bytes[2], length_bytes[3]]);
        out.extend_from_slice(NAMESPACE.as_bytes());
        out.extend_from_slice(name.as_bytes());
        out
    }

    /// Return a list of mockup names for use in testing.
    #[cfg(feature = "iterator")]
    fn mock_names(indexes: Range<usize>) -> Vec<String> {
        let mut names = indexes.map(|i| format!("test-name-{i}")).collect::<Vec<_>>();
        names.sort();
        names
    }

    /// Insert mock names into a set.
    #[cfg(feature = "iterator")]
    fn insert_mock_names(set: Set<&str>, store: &mut dyn Storage) {
        for name in mock_names(1..100) {
            set.insert(store, &name).unwrap();
        }
    }

    #[test]
    fn containing() {
        let mut store = MockStorage::default();

        NAMES.insert(&mut store, "larry").unwrap();
        assert!(NAMES.contains(&store, "larry"));
        assert!(!NAMES.contains(&store, "jake"));

        NAMES.insert(&mut store, "jake").unwrap();
        assert!(NAMES.contains(&store, "larry"));
        assert!(NAMES.contains(&store, "jake"));
    }

    #[test]
    fn inserting() {
        let mut store = MockStorage::default();

        let new = NAMES.insert(&mut store, "larry").unwrap();
        assert!(new);
        assert_eq!(store.get(&key("larry")), Some(b"{}".to_vec()));
        assert_eq!(store.get(&key("jake")), None);

        let new = NAMES.insert(&mut store, "larry").unwrap();
        assert!(!new);
        assert_eq!(store.get(&key("larry")), Some(b"{}".to_vec()));
        assert_eq!(store.get(&key("jake")), None);

        let new = NAMES.insert(&mut store, "jake").unwrap();
        assert!(new);
        assert_eq!(store.get(&key("larry")), Some(b"{}".to_vec()));
        assert_eq!(store.get(&key("jake")), Some(b"{}".to_vec()));
    }

    #[test]
    fn removing() {
        let mut store = MockStorage::default();

        NAMES.insert(&mut store, "larry").unwrap();

        let existed = NAMES.remove(&mut store, "larry").unwrap();
        assert!(existed);
        assert_eq!(store.get(&key("larry")), None);

        let existed = NAMES.remove(&mut store, "jake").unwrap();
        assert!(!existed);
        assert_eq!(store.get(&key("jake")), None);
    }

    #[cfg(feature = "counter")]
    #[test]
    fn counting() {
        let mut store = MockStorage::default();

        let count = NAMES.count(&store).unwrap();
        assert_eq!(count, 0);

        NAMES.insert(&mut store, "larry").unwrap();
        assert_eq!(NAMES.count(&store).unwrap(), 1);

        NAMES.insert(&mut store, "jake").unwrap();
        assert_eq!(NAMES.count(&store).unwrap(), 2);

        NAMES.insert(&mut store, "pumpkin").unwrap();
        assert_eq!(NAMES.count(&store).unwrap(), 3);

        #[cfg(feature = "iterator")]
        {
            NAMES.clear(&mut store);
            assert_eq!(NAMES.count(&store).unwrap(), 0);
        }
    }

    #[cfg(feature = "iterator")]
    #[test]
    fn iterating() {
        let mut store = MockStorage::default();

        insert_mock_names(NAMES, &mut store);

        let names = NAMES
            .items(&store, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(names, mock_names(1..100));

        let start_after = Bound::ExclusiveRaw(b"test-name-2".to_vec());
        let names = NAMES
            .items(&store, Some(start_after), None, Order::Ascending)
            .take(10)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(names, mock_names(20..30));
    }

    #[cfg(feature = "iterator")]
    #[test]
    fn clearing() {
        let mut store = MockStorage::default();

        insert_mock_names(NAMES, &mut store);

        NAMES.clear(&mut store);

        let names = NAMES
            .items(&store, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(names.len(), 0);
    }

    #[cfg(feature = "iterator")]
    #[test]
    fn prefixes() {
        let mut store = MockStorage::default();

        let tuples = vec![(1u64, "larry"), (1u64, "jake"), (2u64, "pumpkin")];

        for tuple in &tuples {
            TUPLES.insert(&mut store, *tuple).unwrap();
        }

        let names = TUPLES
            .prefix(1)
            .keys(&store, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(names, vec!["jake", "larry"]);

        let names = TUPLES
            .prefix(2)
            .keys(&store, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()
            .unwrap();
        assert_eq!(names, vec!["pumpkin"]);
    }
}
