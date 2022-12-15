use cosmwasm_schema::cw_serde;
use cw_ownable::{cw_ownable, Action};

#[cw_ownable]
#[cw_serde]
enum MyEnum {
    Foo,
    Bar(u64),
    Fuzz {
        buzz: String,
    },
}

#[test]
fn derive_execute_variants() {
    let my_enum = MyEnum::Foo;

    // If this compiles we have won.
    match my_enum {
        MyEnum::UpdateOwnership(Action::TransferOwnership {
            new_owner: _,
            expiry: _,
        })
        | MyEnum::UpdateOwnership(Action::AcceptOwnership)
        | MyEnum::UpdateOwnership(Action::RenounceOwnership)
        | MyEnum::Foo
        | MyEnum::Bar(_)
        | MyEnum::Fuzz {
            ..
        } => "yay",
    };
}
