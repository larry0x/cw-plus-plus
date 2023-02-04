# CW Ownable

Utility for controlling ownership of [CosmWasm](https://github.com/CosmWasm/cosmwasm) smart contracts.

## How to use

Initialize the owner during instantiation using the `initialize_owner`
method provided by this crate:

```rust
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw_ownable::OwnershipError;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<Empty>, OwnershipError> {
    cw_ownable::initialize_owner(deps.storage, deps.api, msg.owner.as_deref())?;
	Ok(Response::new())
}
```

Use the `#[cw_ownable_execute]` macro to extend your execute message:

```rust
use cosmwasm_schema::cw_serde;
use cw_ownable::{cw_ownable_execute, Expiration};

#[cw_ownable_execute]
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

Use the `#[cw_ownable_query]` macro to extend your query message:

```rust
use cosmwasm_schema::cw_serde;
use cw_ownable::cw_ownable_query;

#[cw_ownable_query]
#[cw_serde]
pub enum QueryMsg {
	Foo {},
	Bar {},
}
```

The macro inserts a new variant, `Ownership`

```rust
use cosmwasm_schema::cw_serde;
use cw_ownable::Ownership;

#[cw_serde]
enum ExecuteMsg {
    #[returns(Ownership<String>)]
    Ownership {},
    #[returns(FooResponse)]
    Foo {},
    #[returns(BarResponse)]
    Bar {},
}
```

Handle the message using the `get_ownership` function provided by this
crate:

```rust
use cosmwasm_std::{entry_point, Deps, Env, Binary};
use cw_ownable::{cw_serde, get_ownership};

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_binary(&get_ownership(deps.storage)?),
		_ => unimplemneted!(),
    }
	Ok(Binary::default())
}
```

## License

Contents of this crate are open source under [GNU Affero General
Public License v3](../../LICENSE) or later.
