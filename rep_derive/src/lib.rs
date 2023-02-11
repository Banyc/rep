mod utils;

use utils::*;

extern crate proc_macro;

use quote::quote;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Error, Fields, ImplItemMethod, ItemImpl, Lit, Meta,
    NestedMeta,
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

    let mut checks = vec![];
    let mut check_errors = vec![];
    let mut errors = vec![];
    let mut fields_to_recurse_on = vec![];

    if let Data::Struct(data_struct) = data {
        let fields = data_struct.fields;
        let mut fields_contents = None;
        if let Fields::Named(fields_named) = fields {
            fields_contents = Some(fields_named.named);
        // } else if let Fields::Unnamed(fields_unnamed) = fields {
        //     fields_contents = Some(fields_unnamed.unnamed);
        } else {
            errors.push(Error::new(fields.span(), "expected named fields").to_compile_error());
        }

        for (_, field) in fields_contents.unwrap().iter().enumerate() {
            for attr in field.attrs.clone() {
                let maybe_meta = attr.parse_meta();

                if let Ok(meta) = maybe_meta {
                    if let Meta::List(meta_list) = meta {
                        if meta_list.path.is_ident("rep") {
                            if meta_list.nested.len() == 1 {
                                let nested = meta_list.nested.first().unwrap();

                                // #[rep] comes in 2 varieties
                                // 1. literals like #[rep(eq ="my_func")]
                                // 2. paths like #[rep(always_true)]
                                if let NestedMeta::Meta(nested_meta) = nested {
                                    let field_name = field.ident.clone().unwrap(); //_or(Ident::new(&i.to_string(), Span::call_site()));
                                    let field_name_name = field.ident.clone().unwrap().to_string();
                                    let field_type = field.ty.clone();
                                    match nested_meta {
                                        Meta::Path(p) => {
                                            if p.is_ident("check") {
                                                fields_to_recurse_on.push(field_name);
                                            } else if p.is_ident("assert_default") {
                                                checks.push(quote! {
                                                    {
                                                        let default: #field_type = Default::default();
                                                        self.#field_name == default
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be default, not {}", #field_name_name, self.#field_name)
                                                });
                                            } else if p.is_ident("assert_true") {
                                                checks.push(quote! {
                                                    {
                                                        self.#field_name
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be true", #field_name_name)
                                                });
                                            } else if p.is_ident("assert_false") {
                                                checks.push(quote! {
                                                    {
                                                        !self.#field_name
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be false", #field_name_name)
                                                });
                                            } else {
                                                errors.push(
                                                    Error::new(
                                                        p.span(),
                                                        "unsupported representation invariant",
                                                    )
                                                    .to_compile_error(),
                                                );
                                            }
                                        }
                                        Meta::NameValue(v) => {
                                            if v.path.is_ident("assert_eq") {
                                                let val = v.lit.clone();
                                                checks.push(quote! {
                                                    {
                                                        self.#field_name == #val
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be {}, not {}", #field_name_name, #val, self.#field_name)
                                                });
                                            } else if v.path.is_ident("assert_ne") {
                                                let val = v.lit.clone();
                                                checks.push(quote! {
                                                    {
                                                        self.#field_name != #val
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must not be {}", #field_name_name, #val)
                                                });
                                            } else if v.path.is_ident("assert_gt") {
                                                let val = v.lit.clone();
                                                checks.push(quote! {
                                                    {
                                                        self.#field_name > #val
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be > {}, not {}", #field_name_name, #val, self.#field_name)
                                                });
                                            } else if v.path.is_ident("assert_lt") {
                                                let val = v.lit.clone();
                                                checks.push(quote! {
                                                    {
                                                        self.#field_name < #val
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be < {}, not {}", #field_name_name, #val, self.#field_name)
                                                });
                                            } else if v.path.is_ident("assert_ge") {
                                                let val = v.lit.clone();
                                                checks.push(quote! {
                                                    {
                                                        self.#field_name >= #val
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be >= {}, not {}", #field_name_name, #val, self.#field_name)
                                                });
                                            } else if v.path.is_ident("assert_le") {
                                                let val = v.lit.clone();
                                                checks.push(quote! {
                                                    {
                                                        self.#field_name <= #val
                                                    }
                                                });
                                                check_errors.push(quote! {
                                                    format!("self.{} must be <= {}, not {}", #field_name_name, #val, self.#field_name)
                                                });
                                            } else if v.path.is_ident("assert_with") {
                                                let val = v.lit.clone();
                                                if let Lit::Str(fn_name) = val.clone() {
                                                    let fn_to_call = fn_name.parse::<syn::Path>();
                                                    if let Ok(fn_to_call) = fn_to_call.clone() {
                                                        checks.push(quote! {
                                                        {
                                                            #fn_to_call ( self.#field_name )
                                                        }
                                                        });
                                                        check_errors.push(quote! {
                                                            format!("{}(self.{}) must be true when self.{} == {}", #fn_name, #field_name_name, #field_name_name, self.#field_name)
                                                        });
                                                    } else {
                                                        errors.push(Error::new(val.span(), "assert_with can only be used with the name of a function to call").to_compile_error());
                                                    }
                                                } else {
                                                    errors.push(Error::new(val.span(), "assert_with can only be used with the name of a function to call").to_compile_error());
                                                }
                                            } else {
                                                errors.push(
                                                    Error::new(
                                                        v.span(),
                                                        "unsupported representation invariant",
                                                    )
                                                    .to_compile_error(),
                                                );
                                            }
                                        }
                                        _ => {
                                            errors.push(
                                                Error::new(
                                                    nested_meta.span(),
                                                    "unsupported representation invariant",
                                                )
                                                .to_compile_error(),
                                            );
                                        }
                                    }
                                } else {
                                    errors.push(
                                        Error::new(nested.span(), "invalid usage of #[rep]")
                                            .to_compile_error(),
                                    );
                                }
                            } else {
                                errors.push(
                                    Error::new(meta_list.span(), "expected just 1 item")
                                        .to_compile_error(),
                                );
                            }
                        }
                    }
                }
            }
        }
    } else {
        errors.push(Error::new(name.span(), "expected name of structure").to_compile_error());
    }

    let expanded = if errors.len() > 0 {
        quote! {
            #(#errors)*
        }
    } else {
        quote! {
            impl rep::CheckIndieFields for #name {
                fn check_indie_fields(&self, e: &mut RepErrors) {
                    #( if ! #checks { e.add( #check_errors ); } )*
                    #(
                        self. #fields_to_recurse_on .check_indie_fields(e);
                    )*
                }
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
        wrap_checks_in_impl(impl_block, true, true, true)
            .to_token_stream()
            .into()
    } else if let Ok(method) = syn::parse::<ImplItemMethod>(item.clone()) {
        // insert calls to check rep at start and end of method
        let new_method = wrap_checks_in_method(method, true, true, true);

        new_method.to_token_stream().into()
    } else {
        error_quote("expected impl block or method").into()
    }
}

/// A macro that inserts a call to `check_rep` at the start of given method
#[proc_macro_attribute]
pub fn require_rep(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if let Ok(method) = syn::parse::<ImplItemMethod>(item) {
        // insert calls to check rep at start of method
        let new_method = wrap_checks_in_method(method, true, false, false);

        new_method.to_token_stream().into()
    } else {
        error_quote("expected method").into()
    }
}

/// A macro that inserts a call to `check_rep` at the end of given method
#[proc_macro_attribute]
pub fn ensure_rep(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    if let Ok(method) = syn::parse::<ImplItemMethod>(item) {
        // insert calls to check rep at end of method
        let new_method = wrap_checks_in_method(method, false, true, true);

        new_method.to_token_stream().into()
    } else {
        error_quote("expected method").into()
    }
}
