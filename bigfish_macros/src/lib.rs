use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

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
