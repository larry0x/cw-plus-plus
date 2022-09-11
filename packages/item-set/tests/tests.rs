use cosmwasm_std::testing::mock_dependencies;

use cw_item_set::Set;

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

    let existed = NAMES.remove(deps.as_mut().storage, "larry");

    assert!(existed);
    assert_eq!(deps.as_ref().storage.get(&key("larry")), None);

    let existed = NAMES.remove(deps.as_mut().storage, "jake");

    assert!(!existed);
    assert_eq!(deps.as_ref().storage.get(&key("jake")), None);
}
