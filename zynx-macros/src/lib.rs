use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, PatType, Type, parse_macro_input, spanned::Spanned};

/// #[event_handler] on_* free functions
#[proc_macro_attribute]
pub fn event_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_vis = &input.vis;
    let fn_sig = &input.sig;
    let fn_block = &input.block;
    let fn_name = &fn_sig.ident;

    if fn_sig.asyncness.is_none() {
        return syn::Error::new(fn_sig.span(), "event handler must be async")
            .to_compile_error()
            .into();
    }
    if fn_sig
        .inputs
        .iter()
        .any(|arg| matches!(arg, FnArg::Receiver(_)))
    {
        return syn::Error::new(fn_sig.span(), "event handler must be a free function")
            .to_compile_error()
            .into();
    }

    let name_str = fn_name.to_string();
    if !name_str.starts_with("on_") || name_str == "on_" {
        return syn::Error::new(
            fn_name.span(),
            "expected fn name starting with `on_` (e.g. on_ready)",
        )
        .to_compile_error()
        .into();
    }

    // event name: on_message_create -> MessageCreate
    let event_snake = &name_str["on_".len()..];
    let event_pascal = snake_to_pascal(event_snake);
    let event_ident = syn::Ident::new(&event_pascal, fn_name.span());

    // Infer payload type by default: Gateway{Event}DispatchData
    let payload_name = match event_pascal.as_str() {
        "MessagePollVoteAdd" | "MessagePollVoteRemove" => "MessagePollVote",
        _ => &event_pascal,
    };
    let inferred_ty_ident = format_ident!("Gateway{}DispatchData", payload_name);
    let inferred_ty = quote!(::zynx::models::gateway::#inferred_ty_ident);

    // parameters: allow (ctx) or (ctx, payload)
    let mut has_payload = false;

    match fn_sig.inputs.len() {
        1 => {
            // ok: (ctx)
        }
        2 => {
            has_payload = true;
            let second = fn_sig.inputs.iter().nth(1).unwrap();
            match second {
                FnArg::Typed(PatType { ty, .. }) => {
                    if matches!(&**ty, Type::Infer(_)) {
                        // fn param types cannot be `_`; guide the user.
                        return syn::Error::new(
                                ty.span(),
                                "type `_` is not allowed in fn params; omit the payload param or write its concrete type",
                            )
                            .to_compile_error()
                            .into();
                    }
                }
                FnArg::Receiver(_) => {
                    return syn::Error::new(
                        second.span(),
                        "handler must be a free fn, not a method",
                    )
                    .to_compile_error()
                    .into();
                }
            }
        }
        _ => {
            return syn::Error::new(
                fn_sig.inputs.span(),
                "expected signature like: async fn on_<event>(ctx[, payload])",
            )
            .to_compile_error()
            .into();
        }
    }

    // Bind the context to the registered event, even with a second argument.
    let payload_ty = {
        if event_pascal == "Resumed" {
            quote!(())
        } else {
            inferred_ty.clone()
        }
    };

    let mut expanded_sig = fn_sig.clone();
    infer_context_type(&mut expanded_sig, &payload_ty);

    // Shim name
    let shim_ident = format_ident!("__df_shim_{}", fn_name);

    // Context-only handlers share one decoded payload. The optional second
    // argument preserves the owned-payload style by cloning the decoded data.
    let call = if has_payload {
        quote! { let payload = ctx.data().clone(); #fn_name(ctx, payload).await }
    } else {
        quote! { #fn_name(ctx).await }
    };
    let shim_call = quote! {
        Box::pin(async move {
            match ctx.decode::<#payload_ty>() {
                Ok(ctx) => { #call }
                Err(error) => ::zynx::log!("WARN", "failed to decode payload for {}: {}", #name_str, error),
            }
        })
    };

    // Expand: original fn + shim + inventory submit
    let expanded = quote! {
        #fn_vis #expanded_sig #fn_block

        #[doc(hidden)]
        fn #shim_ident(
            ctx: ::zynx::framework::context::Ctx,
        ) -> ::zynx::framework::events::registry::DynFut {
            #shim_call
        }

        ::zynx::__private::inventory::submit! {
            ::zynx::framework::events::registry::Handler {
                event: ::zynx::models::gateway::GatewayDispatchEvents::#event_ident,
                f: #shim_ident,
                name: #name_str,
            }
        }
    };
    expanded.into()
}

// Rewrite only an unparameterized Ctx. Explicit types remain intact so the
// generated call still rejects a context that does not match the event.
fn infer_context_type(signature: &mut syn::Signature, payload: &proc_macro2::TokenStream) {
    let Some(FnArg::Typed(argument)) = signature.inputs.first_mut() else {
        return;
    };
    let Type::Path(path) = argument.ty.as_mut() else {
        return;
    };
    if path.qself.is_some() {
        return;
    }
    let Some(segment) = path.path.segments.last_mut() else {
        return;
    };
    if segment.ident == "Ctx" && matches!(segment.arguments, syn::PathArguments::None) {
        segment.arguments = syn::PathArguments::AngleBracketed(syn::parse_quote!(<#payload>));
    }
}

fn snake_to_pascal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up = true;
    for ch in s.chars() {
        if ch == '_' {
            up = true;
            continue;
        }
        if up {
            out.extend(ch.to_uppercase());
            up = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_bare_and_qualified_contexts() {
        for mut signature in [
            syn::parse_quote!(async fn on_ready(ctx: Ctx)),
            syn::parse_quote!(async fn on_ready(ctx: zynx::prelude::Ctx)),
        ] {
            infer_context_type(&mut signature, &quote!(Ready));
            let FnArg::Typed(arg) = signature.inputs.first().unwrap() else {
                panic!()
            };
            let Type::Path(path) = arg.ty.as_ref() else {
                panic!()
            };
            let args = &path.path.segments.last().unwrap().arguments;
            assert_eq!(quote!(#args).to_string(), quote!(<Ready>).to_string());
        }
    }

    #[test]
    fn preserves_explicit_context_types_and_aliases() {
        for mut signature in [
            syn::parse_quote!(async fn on_ready(ctx: Ctx<MessageCreate>)),
            syn::parse_quote!(async fn on_ready(ctx: ReadyContext)),
        ] {
            let before = quote!(#signature).to_string();
            infer_context_type(&mut signature, &quote!(Ready));
            assert_eq!(quote!(#signature).to_string(), before);
        }
    }
}
