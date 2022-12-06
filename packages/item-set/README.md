# CW Item Set

Set of non-duplicate items for [CosmWasm](https://github.com/CosmWasm/cosmwasm) smart contract store.

## How to use

In this example, we create a whitelist of users. This may be useful in, for example, NFT minting whitelists.

```rust
use cosmwasm_std::{DepsMut, Order, StdResult};
use cw_item_set::Set;

// "whitelist": namespace under which the items are to be stored
// "whitelist__counter": key for storing the total number of items in the set
const WHITELIST: Set<&str> = Set::new("whitelist", "whitelist__counter");

fn example(deps: DepsMut) -> StdResult<()> {
    // Add a new user to the whitelist
    WHITELIST.insert(deps.storage, "larry")?;

    // Remove a user from the whitelist
    // Note that we don't check whether the user already exists in the whitelist.
    // Attempting to remove a non-existent user won't result in error.
    WHITELIST.remove(deps.storage, "jake")?;

    // Check whether a user is in the whitelist
    let is_whitelisted = WHITELIST.contains(deps.as_ref().storage, "pumpkin");

    // Check the total number of users in the whitelist
    let num_users = WHITELIST.count(deps.as_ref().storage)?;

    // Enumerate all users in the whitelist
    for res in WHITELIST.items(deps.as_ref().storage, None, None, Order::Ascending) {
        let user = res?;
        println!("{} is whitelisted!", user);
    }

    // Delete all users in the whitelist
    WHITELIST.clear(deps.storage);

    Ok(())
}
```

## Features

There are two optional features, both enabled by default:

- `iterator`: The `range`, `prefix`, and `clear` functions require this feature.

- `count`: The `count` function requires this feature. If enabled, an `Item<u64>` will be created to store the total number of items in the set. In this case, it is necessary to provide a storage key for the counter when declaring a set:

  ```rust
  // `counter` feature ENABLED

  // The `new` function takes two parameters, the namespace for the set, and the
  // key for the counter.
  const WHITELIST: Set<&str> = Set::new("whitelist", "whitelist__counter");

  // Use the `count` function to get the total number of items in the set.
  let num_users = WHITELIST.count(deps.storage)?;
  ```

  If counting the total number of items in the set is not needed, it is recommended to disable this feature, which saves gas by avoiding having to update the counter every time an item is added or removed.

  In this case, the store key parameter is no longer needed when creating a set.

  ```rust
  // `counter` feature DISABLED

  // Only the set namespace is required
  const WHITELIST: Set<&str> = Set::new("whitelist");

  // This won't compile: `count` function is not supported
  let num_users = WHITELIST.count(deps.storage)?; // ERROR!
  ```

## License

Contents of this crate are open source under [GNU Affero General Public License v3](../../LICENSE) or later.
