# CW Ownable

Utility for controlling ownership of [CosmWasm](https://github.com/CosmWasm/cosmwasm) smart contracts.

## How to use

Use the `#[cw_ownable]` macro to define your execute message:

```rust
use cosmwasm_schema::cw_serde;
use cw_ownable::{cw_ownable, Expiration};

#[cw_ownable]
#[cw_serde]
enum ExecuteMsg {
    Foo {},
    Bar {},
}
```

The macro inserts three variants, `{Transfer,Accept,Renounce}Ownership` to the enum:

```rust
#[cw_serde]
enum ExecuteMsg {
    TransferOwnership {
        new_owner: String,
        expiry: Option<Expiration>,
    },
    AcceptOwnership {},
    RenounceOwnership {},
    Foo {},
    Bar {},
}
```

Handle the messages using the functions provided by this crate:

```rust
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{cw_serde, OwnershipError};

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, OwnershipError> {
    match msg {
        ExecuteMsg::TransferOwnership {
            new_owner,
            expiry,
        } => {
            cw_ownable::transfer_ownership(deps, &info.sender, &new_owner, expiry)?;
        },
        ExecuteMsg::AcceptOwnership {} => {
            cw_ownable::accept_ownership(deps.storage, &env.block, info.sender)?;
        },
        ExecuteMsg::RenounceOwnership {} => {
            cw_ownable::renounce_ownership(deps.storage, &info.sender)?;
        },
        _ => unimplemneted!(),
    }
    Ok(Response::new())
}
```

## License

Contents of this crate are open source under [GNU Affero General Public License v3](../../LICENSE) or later.
