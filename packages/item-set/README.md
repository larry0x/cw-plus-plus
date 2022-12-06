# CW Item Set

Set of non-duplicate items for [CosmWasm](https://github.com/CosmWasm/cosmwasm) smart contract store.

## Usage

```rust
use cosmwasm_std::{DepsMut, Order, StdResult};
use cw_item_set::Set;

const VALIDATORS: Set<&str> = Set::new("validators", "validators__counter");

fn example(deps: DepsMut) -> StdResult<()> {
    // add a new item to the set
    VALIDATORS.insert(deps.storage, "larry")?;

    // remove an existing item from the set
    VALIDATORS.remove(deps.storage, "jake")?;

    // check whether an item is in the set
    let is_whitelisted = VALIDATORS.contains(deps.as_ref().storage, "pumpkin");

    // check the total number of of items in the set
    let num_validators = VALIDATORS.count(deps.as_ref().storage)?;

    // enumerate items in the set
    for res in VALIDATORS.items(deps.as_ref().storage, None, None, Order::Ascending) {
        let validator = res?;
        println!("{} is whitelisted!", validator);
    }

    // delete all items in the set
    VALIDATORS.clear(deps.storage);

    Ok(())
}
```

## License

Contents of this crate are open source under [GNU Affero General Public License v3](../../LICENSE) or later.
