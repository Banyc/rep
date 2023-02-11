mod check_indie_fields;
mod check_rep;

use check_indie_fields::*;
use check_rep::*;

extern crate proc_macro;

use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Error, ImplItemMethod, ItemImpl, Meta, NestedMeta,
};

/// A macro for deriving an implementation of `CheckRep`
///
/// The following usages of `#[rep]` are supported.
/// - `#[rep(assert_default)]`
/// - `#[rep(assert_true)]`
/// - `#[rep(assert_false)]`
/// - `#[rep(assert_eq = "---")]`
/// - `#[rep(assert_gt = 0.0)]`
/// - `#[rep(assert_lt = 100.0)]`
/// - `#[rep(assert_ge = 20)]`
/// - `#[rep(assert_le = 40)]`
/// - `#[rep(assert_with = "has_valid_id")]`
/// - `#[rep(check)]`
#[proc_macro_derive(CheckIndieFields, attributes(rep))]
pub fn derive_check_indie_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let data = input.data;

    let mut check_conditions = vec![];
    let mut check_errors = vec![];
    let mut errors = vec![];
    let mut fields_to_recurse_on = vec![];

    if let Data::Struct(data_struct) = data {
        for (_, field) in data_struct.fields.into_iter().enumerate() {
            for attr in field.attrs.clone() {
                let maybe_meta = attr.parse_meta();

                if let Ok(meta) = maybe_meta {
                    if let Meta::List(meta_list) = meta {
                        if meta_list.path.is_ident("rep") {
                            for meta in meta_list.nested {
                                if let NestedMeta::Meta(meta) = meta {
                                    check_indie_fields(
                                        &meta,
                                        &field,
                                        &mut check_conditions,
                                        &mut check_errors,
                                        &mut errors,
                                        &mut fields_to_recurse_on,
                                    );
                                } else {
                                    errors.push(Error::new(meta.span(), "invalid usage of #[rep]"));
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        errors.push(Error::new(name.span(), "expected name of structure"));
    }

    let expanded = match errors.is_empty() {
        true => {
            quote! {
                impl rep::CheckIndieFields for #name {
                    fn check_indie_fields(&self, e: &mut RepErrors) {
                        #( if ! #check_conditions { e.add( #check_errors ); } )*
                        #(
                            self. #fields_to_recurse_on .check_indie_fields(e);
                        )*
                    }
                }
            }
        }
        false => {
            let errors = errors.into_iter().map(|e| e.to_compile_error());
            quote! {
                #(#errors)*
            }
        }
    };

    // hand the output tokens back to the compiler.
    proc_macro::TokenStream::from(expanded)
}

/// A macro that auto-inserts calls to `check_rep`
///
/// This macro can be applied to an `impl` block to inserts calls to `check_rep` only in methods that satisfy the following:
///
/// - Visibility is `pub` and either:
///   - parameters include `&mut self`
///   - the return type is `Self`
///
/// You may also apply it to a method in an `impl` block regardless of the method's signature.
#[proc_macro_attribute]
pub fn check_rep(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if let Ok(impl_block) = syn::parse::<ItemImpl>(item.clone()) {
        match wrap_checks_in_impl(impl_block, true, true, true) {
            Ok(impl_block) => impl_block.to_token_stream(),
            Err(e) => e.to_compile_error(),
        }
    } else if let Ok(method) = syn::parse::<ImplItemMethod>(item) {
        // insert calls to check rep at start and end of method
        match wrap_checks_in_method(method, true, true, true) {
            Ok(method) => method.to_token_stream(),
            Err(e) => e.to_compile_error(),
        }
    } else {
        error_quote("expected impl block or method")
    }
    .into()
}

/// A macro that inserts a call to `check_rep` at the start of given method
#[proc_macro_attribute]
pub fn require_rep(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if let Ok(impl_block) = syn::parse::<ItemImpl>(item.clone()) {
        match wrap_checks_in_impl(impl_block, true, false, false) {
            Ok(impl_block) => impl_block.to_token_stream(),
            Err(e) => e.to_compile_error(),
        }
    } else if let Ok(method) = syn::parse::<ImplItemMethod>(item) {
        // insert calls to check rep at start of method
        match wrap_checks_in_method(method, true, false, false) {
            Ok(method) => method.to_token_stream(),
            Err(e) => e.to_compile_error(),
        }
    } else {
        error_quote("expected method")
    }
    .into()
}

/// A macro that inserts a call to `check_rep` at the end of given method
#[proc_macro_attribute]
pub fn ensure_rep(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if let Ok(impl_block) = syn::parse::<ItemImpl>(item.clone()) {
        match wrap_checks_in_impl(impl_block, false, true, true) {
            Ok(impl_block) => impl_block.to_token_stream(),
            Err(e) => e.to_compile_error(),
        }
    } else if let Ok(method) = syn::parse::<ImplItemMethod>(item) {
        // insert calls to check rep at end of method
        match wrap_checks_in_method(method, false, true, true) {
            Ok(method) => method.to_token_stream(),
            Err(e) => e.to_compile_error(),
        }
    } else {
        error_quote("expected method")
    }
    .into()
}
