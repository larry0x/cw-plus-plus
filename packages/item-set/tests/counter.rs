use cosmwasm_std::testing::MockStorage;

use cw_item_set::Set;

const NAMESPACE: &str = "names";
const NAMES: Set<&str> = Set::new(NAMESPACE, "names__counter");

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

    NAMES.clear(&mut store);
    assert_eq!(NAMES.count(&store).unwrap(), 0);
}
