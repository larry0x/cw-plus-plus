# CW++

A collection of [CosmWasm][1] utilities and helper libraries.

> These packages are compatible with CosmWasm `1.x` for all versions `<2.0`.

## Contents

| Crate                    | Version | Description                                                             |
| ------------------------ | ------- | ----------------------------------------------------------------------- |
| [cw-address-like][2]     | v2.0.0  | A trait that marks unchecked or checked address strings                 |
| [cw-item-set][3]         | v2.0.0  | Set of non-duplicate items for storage                                  |
| [cw-optional-indexes][4] | v2.0.0  | Index types for `IndexedMap` where an item may or may not have an index |
| [cw-ownable][5]          | v2.0.0  | Utility for controlling contract ownership                              |
| [cw-paginate][6]         | v2.0.0  | Helper function for interating maps                                     |

## License

Contents of this repository at or prior to commit [`9c8fcf1`][7] are published under [GNU Affero General Public License v3][8] or later; contents after the said commit are published under [Apache-2.0][9] license.

[1]: https://github.com/CosmWasm/cosmwasm
[2]: ./packages/address-like/
[3]: ./packages/item-set/
[4]: ./packages/optional-indexes/
[5]: ./packages/ownable/
[6]: ./packages/paginate/
[7]: https://github.com/steak-enjoyers/cw-plus-plus/commit/9c8fcf1c95b74dd415caf5602068c558e9d16ecc
[8]: https://github.com/steak-enjoyers/cw-plus-plus/blob/9c8fcf1c95b74dd415caf5602068c558e9d16ecc/LICENSE
[9]: ./LICENSE
