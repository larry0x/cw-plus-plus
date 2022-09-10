use std::ops::Range;

use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Order, StdResult};

use cw_item_set::Set;
use cw_storage_plus::Bound;

const NAMESPACE: &str = "names";
const NAMES: Set<&str> = Set::new(NAMESPACE);

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

/// Return a list of mockup names for use in testing
fn mock_names(indexes: Range<usize>) -> Vec<String> {
    let mut names = indexes.map(|i| format!("test-name-{}", i)).collect::<Vec<_>>();
    names.sort();
    names
}

#[test]
fn containing() {
    let mut deps = mock_dependencies();

    NAMES.insert(deps.as_mut().storage, "larry").unwrap();

    assert!(NAMES.contains(deps.as_ref().storage, "larry"));
    assert!(!NAMES.contains(deps.as_ref().storage, "jake"));

    NAMES.insert(deps.as_mut().storage, "jake").unwrap();

    assert!(NAMES.contains(deps.as_ref().storage, "larry"));
    assert!(NAMES.contains(deps.as_ref().storage, "jake"));
}

#[test]
fn inserting() {
    let mut deps = mock_dependencies();

    let new = NAMES.insert(deps.as_mut().storage, "larry").unwrap();

    assert!(new);
    assert_eq!(deps.as_ref().storage.get(&key("larry")), Some(b"{}".to_vec()));
    assert_eq!(deps.as_ref().storage.get(&key("jake")), None);

    let new = NAMES.insert(deps.as_mut().storage, "larry").unwrap();

    assert!(!new);
    assert_eq!(deps.as_ref().storage.get(&key("larry")), Some(b"{}".to_vec()));
    assert_eq!(deps.as_ref().storage.get(&key("jake")), None);

    let new = NAMES.insert(deps.as_mut().storage, "jake").unwrap();

    assert!(new);
    assert_eq!(deps.as_ref().storage.get(&key("larry")), Some(b"{}".to_vec()));
    assert_eq!(deps.as_ref().storage.get(&key("jake")), Some(b"{}".to_vec()));
}

#[test]
fn removing() {
    let mut deps = mock_dependencies();

    NAMES.insert(deps.as_mut().storage, "larry").unwrap();

    let existed = NAMES.remove(deps.as_mut().storage, "larry").unwrap();

    assert!(existed);
    assert_eq!(deps.as_ref().storage.get(&key("larry")), None);

    let existed = NAMES.remove(deps.as_mut().storage, "jake").unwrap();

    assert!(!existed);
    assert_eq!(deps.as_ref().storage.get(&key("jake")), None);
}

#[test]
fn iterating() {
    let mut deps = mock_dependencies();

    mock_names(1..100)
        .iter()
        .try_for_each(|name| -> StdResult<_> {
            NAMES.insert(deps.as_mut().storage, name)?;
            Ok(())
        })
        .unwrap();

    let names = NAMES
        .items(deps.as_ref().storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()
        .unwrap();
    assert_eq!(names, mock_names(1..100));

    let start_after = Bound::ExclusiveRaw(b"test-name-2".to_vec());
    let names = NAMES
        .items(deps.as_ref().storage, Some(start_after), None, Order::Ascending)
        .take(10)
        .collect::<StdResult<Vec<_>>>()
        .unwrap();
    assert_eq!(names, mock_names(20..30));
}
