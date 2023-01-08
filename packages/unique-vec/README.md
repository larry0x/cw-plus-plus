# CW Unique Vec

A vector type that is guaranteed to only contain non-duplicate items. It throw an error if a duplicate item is encountered when the `push` method is invoked, or when deserializing from a string.

The JSON string representation of `UniqueVec` is the same as `Vec`, so contracts already using `Vec` can simply swap it to `UniqueVec` without breaking data already saved in the storage.

## How to use

> to be added

## License

Contents of this crate are open source under [GNU Affero General Public License v3](../../LICENSE) or later.
