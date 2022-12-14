use cw_ownable::cw_ownable;

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
        MyEnum::TransferOwnership {
            new_owner: _,
            expiry: _,
        }
        | MyEnum::AcceptOwnership {}
        | MyEnum::RenounceOwnership {}
        | MyEnum::Foo
        | MyEnum::Bar(_)
        | MyEnum::Fuzz {
            ..
        } => "yay",
    };
}
