#![cfg(feature = "iterator")]

use std::ops::Range;

use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Order, StdResult};
use cw_storage_plus::Bound;

use cw_item_set::Set;

const NAMESPACE: &str = "names";
const NAMES: Set<&str> = Set::new(NAMESPACE);

/// Return a list of mockup names for use in testing
fn mock_names(indexes: Range<usize>) -> Vec<String> {
    let mut names = indexes.map(|i| format!("test-name-{}", i)).collect::<Vec<_>>();
    names.sort();
    names
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
