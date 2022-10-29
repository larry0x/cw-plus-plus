use cosmwasm_std::testing::MockStorage;
use cosmwasm_std::Storage;

use cw_item_set::Set;

const NAMESPACE: &str = "names";
const NAMES: Set<&str> = Set::new(NAMESPACE, "names__counter");

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
