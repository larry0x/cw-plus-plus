/// Marks either `String` or `cosmwasm_std::Addr`.
///
/// String is used in unverified types, such as messages and query responses.
/// Addr is used in verified types, which are to be stored in blockchain state.
pub trait AddressLike {}

impl AddressLike for String {}
impl AddressLike for cosmwasm_std::Addr {}