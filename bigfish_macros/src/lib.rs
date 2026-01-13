use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ImplItem, ItemFn, ItemImpl, Type};

#[proc_macro_attribute]
pub fn native_func(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input function
    let input_fn = parse_macro_input!(item as ItemFn);
    let fn_name = &input_fn.sig.ident;
    let shim_name = syn::Ident::new(&format!("__shim_{}", fn_name), fn_name.span());

    // Check if the function has a scope parameter
    let has_scope = input_fn.sig.inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            // Check for direct Scope type
            if let syn::Type::Path(type_path) = &*pat_type.ty {
                if let Some(last_segment) = type_path.path.segments.last() {
                    if last_segment.ident == "Scope" {
                        return true;
                    }
                }
            }
            // Check for reference to Scope (&Scope)
            if let syn::Type::Reference(type_ref) = &*pat_type.ty {
                if let syn::Type::Path(type_path) = &*type_ref.elem {
                    if let Some(last_segment) = type_path.path.segments.last() {
                        if last_segment.ident == "Scope" {
                            return true;
                        }
                    }
                }
            }
        }
        false
    });

    // Generate the shim call based on whether scope is present
    let shim_call = if has_scope {
        quote! {
            let args = crate::dart_api::NativeArguments::from_raw(args);
            let scope = ::std::mem::ManuallyDrop::into_inner(crate::dart_api::Isolate::current().unwrap());
            #fn_name(args, scope);
        }
    } else {
        quote! {
            let args = crate::dart_api::NativeArguments::from_raw(args);
            #fn_name(args);
        }
    };

    // Generate the original function + the shim + the inventory submission
    let expanded = quote! {
        #input_fn

        #[no_mangle]
        pub unsafe extern "C" fn #shim_name(args: crate::dart_api::sys::Dart_NativeArguments) {
            #shim_call
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

        // Check if the function has a scope parameter
        let has_scope = f.sig.inputs.iter().any(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                // Check for direct Scope type
                if let syn::Type::Path(type_path) = &*pat_type.ty {
                    if let Some(last_segment) = type_path.path.segments.last() {
                        if last_segment.ident == "Scope" {
                            return true;
                        }
                    }
                }
                // Check for reference to Scope (&Scope)
                if let syn::Type::Reference(type_ref) = &*pat_type.ty {
                    if let syn::Type::Path(type_path) = &*type_ref.elem {
                        if let Some(last_segment) = type_path.path.segments.last() {
                            if last_segment.ident == "Scope" {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        });

        // Generate the shim call based on whether scope is present
        let shim_call = if has_scope {
            quote! {
                let args = crate::dart_api::NativeArguments::from_raw(args);
                let scope = crate::dart_api::Isolate::current().unwrap();
                #self_ty::#fn_name(args, scope);
            }
        } else {
            quote! {
                let args = crate::dart_api::NativeArguments::from_raw(args);
                #self_ty::#fn_name(args);
            }
        };

        shim_items.push(quote! {
            #(#fn_attrs)*
            #[no_mangle]
            pub unsafe extern "C" fn #shim_name(args: crate::dart_api::sys::Dart_NativeArguments) {
                #shim_call
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
