use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, Error, Field, Lit, Meta};

pub fn check_indie_fields(
    meta: &Meta,
    field: &Field,
    check_conditions: &mut Vec<TokenStream>,
    check_errors: &mut Vec<TokenStream>,
    errors: &mut Vec<Error>,
    fields_to_recurse_on: &mut Vec<syn::Ident>,
) {
    let Some(field_ident) = &field.ident else {
        errors.push(Error::new(field.span(), "expected named fields"));
        return;
    };
    let field_ident_str = field_ident.to_string();
    let field_type = &field.ty;
    match meta {
        Meta::Path(p) => {
            if p.is_ident("check") {
                fields_to_recurse_on.push(field_ident.clone());
            } else if p.is_ident("assert_default") {
                check_conditions.push(quote! {
                    {
                        let default: #field_type = Default::default();
                        self.#field_ident == default
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be default, not {}", #field_ident_str, self.#field_ident)
                });
            } else if p.is_ident("assert_true") {
                check_conditions.push(quote! {
                    {
                        self.#field_ident
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be true", #field_ident_str)
                });
            } else if p.is_ident("assert_false") {
                check_conditions.push(quote! {
                    {
                        !self.#field_ident
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be false", #field_ident_str)
                });
            } else {
                errors.push(Error::new(p.span(), "unsupported representation invariant"));
            }
        }
        Meta::NameValue(v) => {
            if v.path.is_ident("assert_eq") {
                let val = v.lit.clone();
                check_conditions.push(quote! {
                    {
                        self.#field_ident == #val
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be {}, not {}", #field_ident_str, #val, self.#field_ident)
                });
            } else if v.path.is_ident("assert_ne") {
                let val = v.lit.clone();
                check_conditions.push(quote! {
                    {
                        self.#field_ident != #val
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must not be {}", #field_ident_str, #val)
                });
            } else if v.path.is_ident("assert_gt") {
                let val = v.lit.clone();
                check_conditions.push(quote! {
                    {
                        self.#field_ident > #val
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be > {}, not {}", #field_ident_str, #val, self.#field_ident)
                });
            } else if v.path.is_ident("assert_lt") {
                let val = v.lit.clone();
                check_conditions.push(quote! {
                    {
                        self.#field_ident < #val
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be < {}, not {}", #field_ident_str, #val, self.#field_ident)
                });
            } else if v.path.is_ident("assert_ge") {
                let val = v.lit.clone();
                check_conditions.push(quote! {
                    {
                        self.#field_ident >= #val
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be >= {}, not {}", #field_ident_str, #val, self.#field_ident)
                });
            } else if v.path.is_ident("assert_le") {
                let val = v.lit.clone();
                check_conditions.push(quote! {
                    {
                        self.#field_ident <= #val
                    }
                });
                check_errors.push(quote! {
                    format!("self.{} must be <= {}, not {}", #field_ident_str, #val, self.#field_ident)
                });
            } else if v.path.is_ident("assert_with") {
                let val = v.lit.clone();
                if let Lit::Str(fn_name) = val.clone() {
                    let fn_to_call = fn_name.parse::<syn::Path>();
                    if let Ok(fn_to_call) = fn_to_call.clone() {
                        check_conditions.push(quote! {
                            {
                                #fn_to_call ( self.#field_ident )
                            }
                        });
                        check_errors.push(quote! {
                            format!("{}(self.{}) must be true when self.{} == {}", #fn_name, #field_ident_str, #field_ident_str, self.#field_ident)
                        });
                    } else {
                        errors.push(Error::new(
                            val.span(),
                            "assert_with can only be used with the name of a function to call",
                        ));
                    }
                } else {
                    errors.push(Error::new(
                        val.span(),
                        "assert_with can only be used with the name of a function to call",
                    ));
                }
            } else {
                errors.push(Error::new(v.span(), "unsupported representation invariant"));
            }
        }
        _ => {
            errors.push(Error::new(
                meta.span(),
                "unsupported representation invariant",
            ));
        }
    }
}
