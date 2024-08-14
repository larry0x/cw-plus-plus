#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

use std::fmt::Display;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Attribute, BlockInfo, DepsMut, StdError, StdResult, Storage};
use cw_address_like::AddressLike;
use cw_storage_plus::Item;

// re-export the proc macros and the Expiration class
pub use cw_ownable_derive::{cw_ownable_execute, cw_ownable_query};
pub use cw_utils::Expiration;

/// The contract's ownership info
#[cw_serde]
pub struct Ownership<T: AddressLike> {
    /// The contract's current owner.
    /// `None` if the ownership has been renounced.
    pub owner: Option<T>,

    /// The account who has been proposed to take over the ownership.
    /// `None` if there isn't a pending ownership transfer.
    pub pending_owner: Option<T>,

    /// The deadline for the pending owner to accept the ownership.
    /// `None` if there isn't a pending ownership transfer, or if a transfer
    /// exists and it doesn't have a deadline.
    pub pending_expiry: Option<Expiration>,
}

pub struct OwnershipStore {
    pub item: Item<Ownership<Addr>>,
}

impl OwnershipStore {
    pub const fn new(key: &'static str) -> Self {
        Self {
            item: Item::new(key),
        }
    }

    /// Set the given address as the contract owner.
    ///
    /// This function is only intended to be used only during contract instantiation.
    pub fn initialize_owner(
        &self,
        storage: &mut dyn Storage,
        api: &dyn Api,
        owner: Option<&str>,
    ) -> StdResult<Ownership<Addr>> {
        let ownership = Ownership {
            owner: owner.map(|h| api.addr_validate(h)).transpose()?,
            pending_owner: None,
            pending_expiry: None,
        };
        self.item.save(storage, &ownership)?;
        Ok(ownership)
    }

    /// Return Ok(true) if the contract has an owner and it's the given address.
    /// Return Ok(false) if the contract doesn't have an owner, of if it does but
    /// it's not the given address.
    /// Return Err if fails to load ownership info from storage.
    pub fn is_owner(&self, store: &dyn Storage, addr: &Addr) -> StdResult<bool> {
        let ownership = self.item.load(store)?;

        if let Some(owner) = ownership.owner {
            if *addr == owner {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Assert that an account is the contract's current owner.
    pub fn assert_owner(&self, store: &dyn Storage, sender: &Addr) -> Result<(), OwnershipError> {
        let ownership = self.item.load(store)?;
        self.check_owner(&ownership, sender)
    }

    /// Assert that an account is the contract's current owner.
    fn check_owner(
        &self,
        ownership: &Ownership<Addr>,
        sender: &Addr,
    ) -> Result<(), OwnershipError> {
        // the contract must have an owner
        let Some(current_owner) = &ownership.owner else {
            return Err(OwnershipError::NoOwner);
        };

        // the sender must be the current owner
        if sender != current_owner {
            return Err(OwnershipError::NotOwner);
        }

        Ok(())
    }

    /// Update the contract's ownership info based on the given action.
    /// Return the updated ownership.
    pub fn update_ownership(
        &self,
        deps: DepsMut,
        block: &BlockInfo,
        sender: &Addr,
        action: Action,
    ) -> Result<Ownership<Addr>, OwnershipError> {
        match action {
            Action::TransferOwnership {
                new_owner,
                expiry,
            } => self.transfer_ownership(deps.api, deps.storage, sender, &new_owner, expiry),
            Action::AcceptOwnership => self.accept_ownership(deps.storage, block, sender),
            Action::RenounceOwnership => self.renounce_ownership(deps.storage, sender),
        }
    }

    /// Get the current ownership value.
    pub fn get_ownership(&self, storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
        self.item.load(storage)
    }

    /// Propose to transfer the contract's ownership to the given address, with an
    /// optional deadline.
    fn transfer_ownership(
        &self,
        api: &dyn Api,
        storage: &mut dyn Storage,
        sender: &Addr,
        new_owner: &str,
        expiry: Option<Expiration>,
    ) -> Result<Ownership<Addr>, OwnershipError> {
        self.item.update(storage, |ownership| {
            // the contract must have an owner
            self.check_owner(&ownership, sender)?;

            // NOTE: We don't validate the expiry, i.e. asserting it is later than
            // the current block time.
            //
            // This is because if the owner submits an invalid expiry, it won't have
            // any negative effect - it's just that the pending owner won't be able
            // to accept the ownership.
            //
            // By not doing the check, we save a little bit of gas.
            //
            // To fix the erorr, the owner can simply invoke `transfer_ownership`
            // again with the correct expiry and overwrite the invalid one.
            Ok(Ownership {
                pending_owner: Some(api.addr_validate(new_owner)?),
                pending_expiry: expiry,
                ..ownership
            })
        })
    }

    /// Accept a pending ownership transfer.
    fn accept_ownership(
        &self,
        store: &mut dyn Storage,
        block: &BlockInfo,
        sender: &Addr,
    ) -> Result<Ownership<Addr>, OwnershipError> {
        self.item.update(store, |ownership| {
            // there must be an existing ownership transfer
            let Some(pending_owner) = &ownership.pending_owner else {
                return Err(OwnershipError::TransferNotFound);
            };

            // the sender must be the pending owner
            if sender != pending_owner {
                return Err(OwnershipError::NotPendingOwner);
            };

            // if the transfer has a deadline, it must not have been reached
            if let Some(expiry) = &ownership.pending_expiry {
                if expiry.is_expired(block) {
                    return Err(OwnershipError::TransferExpired);
                }
            }

            Ok(Ownership {
                owner: ownership.pending_owner,
                pending_owner: None,
                pending_expiry: None,
            })
        })
    }

    /// Set the contract's ownership as vacant permanently.
    fn renounce_ownership(
        &self,
        store: &mut dyn Storage,
        sender: &Addr,
    ) -> Result<Ownership<Addr>, OwnershipError> {
        self.item.update(store, |ownership| {
            self.check_owner(&ownership, sender)?;

            Ok(Ownership {
                owner: None,
                pending_owner: None,
                pending_expiry: None,
            })
        })
    }
}

/// Actions that can be taken to alter the contract's ownership
#[cw_serde]
pub enum Action {
    /// Propose to transfer the contract's ownership to another account,
    /// optionally with an expiry time.
    ///
    /// Can only be called by the contract's current owner.
    ///
    /// Any existing pending ownership transfer is overwritten.
    TransferOwnership {
        new_owner: String,
        expiry: Option<Expiration>,
    },

    /// Accept the pending ownership transfer.
    ///
    /// Can only be called by the pending owner.
    AcceptOwnership,

    /// Give up the contract's ownership and the possibility of appointing
    /// a new owner.
    ///
    /// Can only be invoked by the contract's current owner.
    ///
    /// Any existing pending ownership transfer is canceled.
    RenounceOwnership,
}

/// Errors associated with the contract's ownership
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum OwnershipError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Contract ownership has been renounced")]
    NoOwner,

    #[error("Caller is not the contract's current owner")]
    NotOwner,

    #[error("Caller is not the contract's pending owner")]
    NotPendingOwner,

    #[error("There isn't a pending ownership transfer")]
    TransferNotFound,

    #[error("A pending ownership transfer exists but it has expired")]
    TransferExpired,
}

/// Storage constant for the contract's ownership
const OWNERSHIP_KEY: &str = "ownership";
const OWNERSHIP: OwnershipStore = OwnershipStore::new(OWNERSHIP_KEY);

/// Set the given address as the contract owner.
///
/// This function is only intended to be used only during contract instantiation.
pub fn initialize_owner(
    storage: &mut dyn Storage,
    api: &dyn Api,
    owner: Option<&str>,
) -> StdResult<Ownership<Addr>> {
    OWNERSHIP.initialize_owner(storage, api, owner)
}

/// Return Ok(true) if the contract has an owner and it's the given address.
/// Return Ok(false) if the contract doesn't have an owner, of if it does but
/// it's not the given address.
/// Return Err if fails to load ownership info from storage.
pub fn is_owner(store: &dyn Storage, addr: &Addr) -> StdResult<bool> {
    OWNERSHIP.is_owner(store, addr)
}

/// Assert that an account is the contract's current owner.
pub fn assert_owner(store: &dyn Storage, sender: &Addr) -> Result<(), OwnershipError> {
    OWNERSHIP.assert_owner(store, sender)
}

/// Update the contract's ownership info based on the given action.
/// Return the updated ownership.
pub fn update_ownership(
    deps: DepsMut,
    block: &BlockInfo,
    sender: &Addr,
    action: Action,
) -> Result<Ownership<Addr>, OwnershipError> {
    OWNERSHIP.update_ownership(deps, block, sender, action)
}

/// Get the current ownership value.
pub fn get_ownership(storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
    OWNERSHIP.get_ownership(storage)
}

impl<T: AddressLike> Ownership<T> {
    /// Serializes the current ownership state as attributes which may
    /// be used in a message response. Serialization is done according
    /// to the std::fmt::Display implementation for `T` and
    /// `cosmwasm_std::Expiration` (for `pending_expiry`). If an
    /// ownership field has no value, `"none"` will be serialized.
    ///
    /// Attribute keys used:
    ///  - owner
    ///  - pending_owner
    ///  - pending_expiry
    ///
    /// Callers should take care not to use these keys elsewhere
    /// in their response as CosmWasm will override reused attribute
    /// keys.
    ///
    /// # Example
    ///
    /// ```rust
    /// use cw_utils::Expiration;
    ///
    /// assert_eq!(
    ///     Ownership {
    ///         owner: Some("blue"),
    ///         pending_owner: None,
    ///         pending_expiry: Some(Expiration::Never {})
    ///     }
    ///     .into_attributes(),
    ///     vec![
    ///         Attribute::new("owner", "blue"),
    ///         Attribute::new("pending_owner", "none"),
    ///         Attribute::new("pending_expiry", "expiration: never")
    ///     ],
    /// )
    /// ```
    pub fn into_attributes(self) -> Vec<Attribute> {
        vec![
            Attribute::new("owner", none_or(self.owner.as_ref())),
            Attribute::new("pending_owner", none_or(self.pending_owner.as_ref())),
            Attribute::new("pending_expiry", none_or(self.pending_expiry.as_ref())),
        ]
    }
}

// This is a nice helper, maybe move to dedicated utils package?
fn none_or<T: Display>(or: Option<&T>) -> String {
    or.map_or_else(|| "none".to_string(), |or| or.to_string())
}

//------------------------------------------------------------------------------
// Tests
//------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use cosmwasm_std::{testing::{mock_dependencies, MockApi}, Timestamp};

    use super::*;

    fn mock_addresses(api: &MockApi) -> [Addr; 3] {
        [
            api.addr_make("larry"),
            api.addr_make("jake"),
            api.addr_make("pumpkin"),
        ]
    }

    fn mock_block_at_height(height: u64) -> BlockInfo {
        BlockInfo {
            height,
            time: Timestamp::from_seconds(10000),
            chain_id: "".into(),
        }
    }

    #[test]
    fn initializing_ownership() {
        let mut deps = mock_dependencies();
        let [larry, _, _] = mock_addresses(&deps.api);

        let ownership =
            OWNERSHIP.initialize_owner(&mut deps.storage, &deps.api, Some(larry.as_str())).unwrap();

        // ownership returned is same as ownership stored.
        assert_eq!(ownership, OWNERSHIP.item.load(deps.as_ref().storage).unwrap());

        assert_eq!(
            ownership,
            Ownership {
                owner: Some(larry),
                pending_owner: None,
                pending_expiry: None,
            },
        );
    }

    #[test]
    fn initialize_ownership_no_owner() {
        let mut deps = mock_dependencies();
        let ownership = OWNERSHIP.initialize_owner(&mut deps.storage, &deps.api, None).unwrap();
        assert_eq!(
            ownership,
            Ownership {
                owner: None,
                pending_owner: None,
                pending_expiry: None,
            },
        );
    }

    #[test]
    fn asserting_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, _] = mock_addresses(&deps.api);

        // case 1. owner has not renounced
        {
            OWNERSHIP.initialize_owner(&mut deps.storage, &deps.api, Some(larry.as_str())).unwrap();

            let res = OWNERSHIP.assert_owner(deps.as_ref().storage, &larry);
            assert!(res.is_ok());

            let res = OWNERSHIP.assert_owner(deps.as_ref().storage, &jake);
            assert_eq!(res.unwrap_err(), OwnershipError::NotOwner);
        }

        // case 2. owner has renounced
        {
            OWNERSHIP.renounce_ownership(deps.as_mut().storage, &larry).unwrap();

            let res = OWNERSHIP.assert_owner(deps.as_ref().storage, &larry);
            assert_eq!(res.unwrap_err(), OwnershipError::NoOwner);
        }
    }

    #[test]
    fn transferring_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, pumpkin] = mock_addresses(&deps.api);

        OWNERSHIP.initialize_owner(&mut deps.storage, &deps.api, Some(larry.as_str())).unwrap();

        // non-owner cannot transfer ownership
        {
            let err = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(12345),
                    &jake,
                    Action::TransferOwnership {
                        new_owner: pumpkin.to_string(),
                        expiry: None,
                    },
                )
                .unwrap_err();
            assert_eq!(err, OwnershipError::NotOwner);
        }

        // owner properly transfers ownership
        {
            let ownership = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(12345),
                    &larry,
                    Action::TransferOwnership {
                        new_owner: pumpkin.to_string(),
                        expiry: Some(Expiration::AtHeight(42069)),
                    },
                )
                .unwrap();
            assert_eq!(
                ownership,
                Ownership {
                    owner: Some(larry),
                    pending_owner: Some(pumpkin),
                    pending_expiry: Some(Expiration::AtHeight(42069)),
                },
            );

            let saved_ownership = OWNERSHIP.item.load(deps.as_ref().storage).unwrap();
            assert_eq!(saved_ownership, ownership);
        }
    }

    #[test]
    fn accepting_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, pumpkin] = mock_addresses(&deps.api);

        OWNERSHIP
            .initialize_owner(&mut deps.storage, &deps.api.clone(), Some(larry.as_str()))
            .unwrap();

        // cannot accept ownership when there isn't a pending ownership transfer
        {
            let err = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(12345),
                    &pumpkin,
                    Action::AcceptOwnership,
                )
                .unwrap_err();
            assert_eq!(err, OwnershipError::TransferNotFound);
        }

        OWNERSHIP
            .transfer_ownership(
                &deps.api.clone().clone(),
                deps.as_mut().storage,
                &larry,
                pumpkin.as_str(),
                Some(Expiration::AtHeight(42069)),
            )
            .unwrap();

        // non-pending owner cannot accept ownership
        {
            let err = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(12345),
                    &jake,
                    Action::AcceptOwnership,
                )
                .unwrap_err();
            assert_eq!(err, OwnershipError::NotPendingOwner);
        }

        // cannot accept ownership if deadline has passed
        {
            let err = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(69420),
                    &pumpkin,
                    Action::AcceptOwnership,
                )
                .unwrap_err();
            assert_eq!(err, OwnershipError::TransferExpired);
        }

        // pending owner properly accepts ownership before deadline
        {
            let ownership = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(10000),
                    &pumpkin,
                    Action::AcceptOwnership,
                )
                .unwrap();
            assert_eq!(
                ownership,
                Ownership {
                    owner: Some(pumpkin),
                    pending_owner: None,
                    pending_expiry: None,
                },
            );

            let saved_ownership = OWNERSHIP.item.load(deps.as_ref().storage).unwrap();
            assert_eq!(saved_ownership, ownership);
        }
    }

    #[test]
    fn renouncing_ownership() {
        let mut deps = mock_dependencies();
        let [larry, jake, pumpkin] = mock_addresses(&deps.api);

        let ownership = Ownership {
            owner: Some(larry.clone()),
            pending_owner: Some(pumpkin),
            pending_expiry: None,
        };
        OWNERSHIP.item.save(deps.as_mut().storage, &ownership).unwrap();

        // non-owner cannot renounce
        {
            let err = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(12345),
                    &jake,
                    Action::RenounceOwnership,
                )
                .unwrap_err();
            assert_eq!(err, OwnershipError::NotOwner);
        }

        // owner properly renounces
        {
            let ownership = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(12345),
                    &larry,
                    Action::RenounceOwnership,
                )
                .unwrap();

            // ownership returned is same as ownership stored.
            assert_eq!(ownership, OWNERSHIP.item.load(deps.as_ref().storage).unwrap());

            assert_eq!(
                ownership,
                Ownership {
                    owner: None,
                    pending_owner: None,
                    pending_expiry: None,
                },
            );
        }

        // cannot renounce twice
        {
            let err = OWNERSHIP
                .update_ownership(
                    deps.as_mut(),
                    &mock_block_at_height(12345),
                    &larry,
                    Action::RenounceOwnership,
                )
                .unwrap_err();
            assert_eq!(err, OwnershipError::NoOwner);
        }
    }

    #[test]
    fn into_attributes_works() {
        use cw_utils::Expiration;
        assert_eq!(
            Ownership {
                owner: Some("blue".to_string()),
                pending_owner: None,
                pending_expiry: Some(Expiration::Never {})
            }
            .into_attributes(),
            vec![
                Attribute::new("owner", "blue"),
                Attribute::new("pending_owner", "none"),
                Attribute::new("pending_expiry", "expiration: never")
            ],
        );
    }
}
