use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemFn, ItemImpl, Type};

#[proc_macro_attribute]
pub fn native_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let shim_name = syn::Ident::new(&format!("__shim_{}", fn_name), fn_name.span());

    // Generate the original function + the shim + the inventory submission
    let expanded = quote! {
        #input_fn

        #[no_mangle]
        pub unsafe extern "C" fn #shim_name(args: crate::dart_api::sys::Dart_NativeArguments) {
            let args = crate::dart_api::NativeArguments::from_raw(args);
            #fn_name(args);
        }

        ::inventory::submit! {
            crate::dart_api::NativeFunction::new(::core::stringify!(#fn_name), #shim_name)
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn native_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(item as ItemImpl);
    let self_ty = &*input_impl.self_ty;

    let type_tag = match self_ty {
        Type::Path(tp) if tp.qself.is_none() => tp
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_else(|| "ty".to_string()),
        _ => "ty".to_string(),
    };

    let mut shim_items = Vec::new();

    for impl_item in &input_impl.items {
        let ImplItem::Fn(f) = impl_item else {
            continue;
        };

        if f.sig.receiver().is_some() {
            let msg = format!(
                "native_impl only supports associated functions (no self parameter): {}::{}",
                type_tag, f.sig.ident
            );
            shim_items.push(quote! {
                ::core::compile_error!(#msg);
            });
            continue;
        }

        let fn_name = &f.sig.ident;
        let shim_name =
            syn::Ident::new(&format!("__shim_{}_{}", type_tag, fn_name), fn_name.span());
        let dart_fn_name =
            syn::Ident::new(&format!("{}_{}", type_tag, f.sig.ident), f.sig.ident.span());

        let fn_attrs = &f.attrs;

        shim_items.push(quote! {
            #(#fn_attrs)*
            #[no_mangle]
            pub unsafe extern "C" fn #shim_name(args: crate::dart_api::sys::Dart_NativeArguments) {
                let args = crate::dart_api::NativeArguments::from_raw(args);
                #self_ty::#fn_name(args);
            }

            #(#fn_attrs)*
            ::inventory::submit! {
                crate::dart_api::NativeFunction::new(::core::stringify!(#dart_fn_name), #shim_name)
            }
        });
    }

    let expanded = quote! {
        #input_impl
        #(#shim_items)*
    };

    TokenStream::from(expanded)
}
