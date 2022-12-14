use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, AttributeArgs, DataEnum, DeriveInput};

/// Merges the variants of two enums.
///
/// Adapted from DAO DAO:
/// https://github.com/DA0-DA0/dao-contracts/blob/74bd3881fdd86829e5e8b132b9952dd64f2d0737/packages/dao-macros/src/lib.rs#L9
fn merge_variants(metadata: TokenStream, left: TokenStream, right: TokenStream) -> TokenStream {
    use syn::Data::Enum;

    // parse metadata
    let args = parse_macro_input!(metadata as AttributeArgs);
    if let Some(first_arg) = args.first() {
        return syn::Error::new_spanned(first_arg, "macro takes no arguments")
            .to_compile_error()
            .into();
    }

    // parse the left enum
    let mut left: DeriveInput = parse_macro_input!(left);
    let Enum(DataEnum {
        variants,
        ..
    }) = &mut left.data else {
        return syn::Error::new(left.ident.span(), "only enums can accept variants")
            .to_compile_error()
            .into();
    };

    // parse the right enum
    let right: DeriveInput = parse_macro_input!(right);
    let Enum(DataEnum {
        variants: to_add,
        ..
    }) = right.data else {
        return syn::Error::new(left.ident.span(), "only enums can provide variants")
            .to_compile_error()
            .into();
    };

    // insert variants from the right to the left
    variants.extend(to_add.into_iter());

    quote! { #left }.into()
}

#[proc_macro_attribute]
pub fn cw_ownable(metadata: TokenStream, input: TokenStream) -> TokenStream {
    merge_variants(
        metadata,
        input,
        quote! {
            enum Right {
                /// Propose to transfer the contract's ownership to another account,
                /// optionally with an expiry time.
                ///
                /// Can only be called by the contract's current owner.
                ///
                /// Any existing pending ownership transfer is overwritten.
                TransferOwnership {
                    new_owner: ::std::string::String,
                    expiry: ::std::option::Option<::cw_utils::Expiration>,
                },

                /// Accept the pending ownership transfer.
                ///
                /// Can only be called by the pending owner.
                AcceptOwnership {},

                /// Give up the contract's ownership and the possibility of appointing
                /// a new owner.
                ///
                /// Can only be invoked by the contract's current owner.
                ///
                /// Any existing pending ownership transfer is canceled.
                RenounceOwnership {},
            }
        }
        .into(),
    )
}
