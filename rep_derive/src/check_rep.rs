use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{Block, Error, FnArg, ImplItem, ItemImpl, ReturnType, Signature, Visibility};

pub fn wrap_checks_in_impl_or_method(
    item: TokenStream,
    prepend: bool,
    append: bool,
    return_self: bool,
) -> TokenStream {
    match syn::parse2(item) {
        Ok(syn::Item::Impl(impl_block)) => {
            match wrap_checks_in_impl(impl_block, prepend, append, return_self) {
                Ok(impl_block) => impl_block.to_token_stream(),
                Err(e) => e.to_compile_error(),
            }
        }
        Ok(syn::Item::Fn(mut func)) => {
            // insert calls to check rep at start and end of func
            match wrap_checks_in_fn(
                &func.vis,
                &func.sig,
                &func.block,
                prepend,
                append,
                return_self,
            ) {
                Ok(new_block) => {
                    if let Some(new_block) = new_block {
                        func.block = Box::new(new_block);
                    }
                    func.to_token_stream()
                }
                Err(e) => e.to_compile_error(),
            }
        }
        _ => error_quote("expected impl block or func"),
    }
}

fn error_quote(message: &str) -> TokenStream {
    Error::new(Span::call_site(), message).to_compile_error()
}

fn wrap_checks_in_impl(
    impl_block: ItemImpl,
    prepend: bool,
    append: bool,
    return_self: bool,
) -> Result<ItemImpl, Error> {
    let mut new_impl_block = impl_block.clone();
    new_impl_block.items = vec![];

    // loop through all items
    // see if the item is pub, accepts &mut self
    // if so, insert calls to check rep
    for mut impl_item in impl_block.items {
        if let ImplItem::Method(method) = &mut impl_item {
            let new_block = wrap_checks_in_fn(
                &method.vis,
                &method.sig,
                &method.block,
                prepend,
                append,
                return_self,
            )?;
            if let Some(new_block) = new_block {
                method.block = new_block;
            }
        }

        new_impl_block.items.push(impl_item);
    }

    Ok(new_impl_block)
}

fn wrap_checks_in_fn(
    vis: &Visibility,
    sig: &Signature,
    block: &Block,
    prepend: bool,
    append: bool,
    return_self: bool,
) -> Result<Option<Block>, Error> {
    let Visibility::Public(_) = vis else {
        return Ok(None);
    };
    if sig.asyncness.is_some() {
        // TODO: support async
        return Ok(None);
    }

    let (prepend_, append_) = if sig.inputs.iter().any(|input| {
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
        if let ReturnType::Type(_, ty) = &sig.output {
            if let syn::Type::Path(type_path) = ty.as_ref() {
                if type_path.path.segments.len() == 1 {
                    if let Some(ident) = type_path.path.segments.first().map(|s| s.ident.clone()) {
                        if ident == "Self" {
                            return_self_ = true;
                        }
                    }
                }
            }
        }
    };

    // replace the method's body with the new block
    let new_block = wrap_checks(
        block,
        CheckOption {
            prepend: prepend_,
            append: append_,
            return_self: return_self_,
        },
    )?;

    Ok(Some(new_block))
}

fn wrap_checks(block: &Block, option: CheckOption) -> Result<Block, Error> {
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

    let prepend_quote = check_rep_quote(option.prepend);
    let append_quote = check_rep_quote(option.append);
    let result_quote = if option.return_self {
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
    )?;

    Ok(new_block)
}

struct CheckOption {
    prepend: bool,
    append: bool,
    return_self: bool,
}
