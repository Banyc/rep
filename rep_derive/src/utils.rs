use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Block, Error, FnArg, ImplItem, ImplItemMethod, ItemImpl, ReturnType, Visibility};

pub fn error_quote(message: &str) -> TokenStream {
    let error = Error::new(Span::call_site(), message).to_compile_error();

    quote! {
        #error
    }
}

pub fn wrap_checks_in_impl(
    impl_block: ItemImpl,
    prepend: bool,
    append: bool,
    return_self: bool,
) -> ItemImpl {
    let mut new_impl_block = impl_block.clone();
    new_impl_block.items = vec![];

    // loop through all items
    // see if the item is pub, accepts &mut self
    // if so, insert calls to check rep
    for impl_item in impl_block.items {
        let mut new_impl_item = impl_item.clone();

        if let ImplItem::Method(method) = impl_item {
            let new_method = wrap_checks_in_method(method, prepend, append, return_self);
            new_impl_item = ImplItem::Method(new_method);
        }

        new_impl_block.items.push(new_impl_item);
    }

    new_impl_block
}

pub fn wrap_checks_in_method(
    method: ImplItemMethod,
    prepend: bool,
    append: bool,
    return_self: bool,
) -> ImplItemMethod {
    let mut new_method = method.clone();

    if let Visibility::Public(_) = method.vis {
        let (prepend_, append_) = if method.sig.inputs.iter().any(|input| {
            if let FnArg::Receiver(receiver) = input {
                receiver.mutability.is_some()
            } else {
                false
            }
        }) {
            (prepend, append)
        } else {
            (false, false)
        };

        let mut return_self_ = false;
        if return_self {
            if let ReturnType::Type(_, ty) = method.sig.output {
                if let syn::Type::Path(type_path) = ty.as_ref() {
                    if type_path.path.segments.len() == 1 {
                        if let Some(ident) =
                            type_path.path.segments.first().map(|s| s.ident.clone())
                        {
                            if ident == "Self" {
                                return_self_ = true;
                            }
                        }
                    }
                }
            }
        };

        // replace the method's body with the new block
        new_method.block = wrap_checks(
            method.block,
            CheckOption {
                prepend: prepend_,
                append: append_,
                return_self: return_self_,
            },
        );
    }

    new_method
}

fn wrap_checks(block: Block, option: CheckOption) -> Block {
    let Block {
        brace_token: _,
        stmts,
    } = block;

    let check_rep_quote = |some| {
        if some {
            quote! {
                self.check_rep();
            }
        } else {
            quote! {}
        }
    };

    let result_quote = check_rep_quote(option.prepend);
    let prepend_quote = check_rep_quote(option.append);
    let append_quote = if option.return_self {
        quote! {
            __result.check_rep();
        }
    } else {
        quote! {}
    };

    let new_block = syn::parse::<Block>(
        quote! {
            {
                #prepend_quote
                let __result = (|| {
                    #(#stmts)*
                })();
                #append_quote
                #result_quote
                __result
            }
        }
        .into(),
    )
    .unwrap();

    new_block
}

struct CheckOption {
    prepend: bool,
    append: bool,
    return_self: bool,
}
