# CW Item Set

Set of non-duplicate items for [CosmWasm](https://github.com/CosmWasm/cosmwasm) smart contract store.

## Background

Developers often find themselves in need to store a set of non-duplicate items in the contract store. A good example is a liquid staking protocol maintaining a whitelist of validators.

There are typically two ways to achieve this:

### The singleton approach

The first is to serialize the whole set and store it under a single storage key. For example:

```rust
use std::collection::HashSet;
use cw_storage_plus::Item;

const VALIDATORS: Item<HashSet<String>> = Item::new("validators");
```

The obvious drawback of this is that every time we need to add or remove an element, or check whether an element exists in the set, we need to load the whole set from the store. This can be quite expensive if the set gets large.

### The map approach

A better approach is to use a map, where the elements are the keys, with the values being empty, i.e.

```rust
use cosmwasm_std::Empty;
use cw_storage_plus::Map;

const VALIDATORS: Map<&str, Empty> = Map::new("validators");
```

An element is considered to be in the set if the corresponding key exists in the map. For example:

```rust
use cosmwasm_std::{StdResult, Storage};

fn is_whitelisted(store: &dyn Storage, validator: &str) -> StdResult<bool> {
    let opt = VALIDATORS.may_load(store, validator)?;
    Ok(opt.is_some())
}
```

With this approach, we can add/remove/find an element in the set with O(1) complexity. It is also possible to enumerate all elements in the set.

However, this approach is not without drawbacks:

- It may be confusing to new developers (I've had quite a few people asking, "why do you store a map with empty values?")
- The total count of elements in the set needs to be tracked separately
- It does not provide some useful methods that are typically expected from sets, such as `union`, `intersection`, `filter`, and of course, a command to delete the whole set.

## This work

This crate attempts to fix some of these drawbacks by implementing a `Set` class. To use:

```rust
use cw_item_set::Set;

const VALIDATORS: Set<&str> = Set::new("validators");
```

The `Set` class provides many useful methods:

```rust
use cosmwasm_std::{DepsMut, Order, StdResult};

fn example(deps: DepsMut) -> StdResult<()> {
    // add a new element to the set
    VALIDATORS.insert(deps.storage, "larry")?;

    // remove an existing element from the set
    VALIDATORS.remove(deps.storage, "jake")?;

    // check whether an element is in the set
    let is_whitelisted = VALIDATORS.contains(deps.storage, "pumpkin");

    // enumerate elements in the set
    VALIDATORS
        .items(deps.storage, None, None, Order::Ascending)
        .for_each(|validator| {
            println!("{} is whitelisted!", validator);
        });
}
```

## License

Contents of this repository are open source under [GNU General Public License v3](https://github.com/st4k3h0us3/cw-plus-plus/blob/master/LICENSE) or later.
