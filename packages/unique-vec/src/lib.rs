#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::{fmt::Debug, ops::Deref};

use cosmwasm_schema::cw_serde;

/// A vector type that is guaranteed to only contain non-duplicate items.
///
/// It throw an error if aduplicate item is encountered when the `push` method
/// is invoked, or when deserializing from a string.
#[cw_serde]
pub struct UniqueVec<T>(Vec<T>)
where
    T: Debug + PartialEq;

/// Implement Deref trait so that the methods of Vec can be accessed, execpt for
/// the following methods, which involves adding new items to the vector and are
/// overwritten by our custom implementation:
///
/// - append
/// - extend_from_slice
/// - extend_from_within
/// - insert
/// - push
/// - push_within_capacity (nightly only)
impl<T> Deref for UniqueVec<T>
where
    T: Debug + PartialEq,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> UniqueVec<T>
where
    T: Debug + PartialEq,
{
}

/// Error indicating that a duplicate item is encountered by UniqueVec.
#[derive(Debug, thiserror::Error)]
#[error("UniqueVec encountered duplicate item: {value}")]
pub struct DuplicationError {
    value: String,
}

impl DuplicationError {
    pub fn new(value: impl Debug) -> Self {
        Self {
            value: format!("{value:?}"),
        }
    }
}
