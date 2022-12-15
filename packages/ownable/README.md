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

The macro inserts a new variant, `UpdateOwnership` to the enum:

```rust
#[cw_serde]
enum ExecuteMsg {
    UpdateOwnership(cw_ownable::Action),
    Foo {},
    Bar {},
}
```

Where `Action` can be one of three:

- Propose to transfer the contract's ownership to another account
- Accept the proposed ownership transfer
- Renounce the ownership, permanently setting the contract's owner to vacant

Handle the messages using the `update_ownership` function provided by this crate:

```rust
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw_ownable::{cw_serde, update_ownership, OwnershipError};

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, OwnershipError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps, &env.block, &info.sender, action)?;
        }
        _ => unimplemneted!(),
    }
    Ok(Response::new())
}
```

## License

Contents of this crate are open source under [GNU Affero General Public License v3](../../LICENSE) or later.
