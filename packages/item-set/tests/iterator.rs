#![cfg(feature = "iterator")]

use std::ops::Range;

use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{Order, StdResult};
use cw_storage_plus::Bound;

use cw_item_set::Set;

const NAMES: Set<&str> = Set::new("names", "names__counter");
const TUPLES: Set<(u64, &str)> = Set::new("tuples", "tuples__counter");

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

#[test]
fn prefixes() {
    let mut deps = mock_dependencies();

    let tuples = vec![
        (1u64, "larry"),
        (1u64, "jake"),
        (2u64, "pumpkin"),
    ];

    tuples
        .iter()
        .try_for_each(|tuple| -> StdResult<_> {
            TUPLES.insert(deps.as_mut().storage, *tuple)?;
            Ok(())
        })
        .unwrap();

    let names = TUPLES
        .prefix(1)
        .keys(deps.as_ref().storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()
        .unwrap();
    assert_eq!(names, vec!["jake", "larry"]);

    let names = TUPLES
        .prefix(2)
        .keys(deps.as_ref().storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()
        .unwrap();
    assert_eq!(names, vec!["pumpkin"]);
}
