//! Safe-ish Rust facade over the Dart native embedding API (`dart_api.h`),
//! with full raw bindings available under [`sys`].

/// Raw bindgen-generated bindings for `dart_dll.h` + `dart_api.h`.
///
/// This surface is inherently **unsafe**: most functions require a current
/// isolate, and many handles are only valid until the current Dart API scope
/// exits.
pub mod sys {
    #![allow(
        non_upper_case_globals,
        non_camel_case_types,
        non_snake_case,
        unused,
        clippy::all
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod dart_api;
pub mod dart;

