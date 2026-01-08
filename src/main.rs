use std::{
    ffi::CStr,
    mem::MaybeUninit,
    process::{Child, Command, Stdio},
};

use clap::Parser;

mod dart_api;

use dart_api::{Runtime, RuntimeConfig};

use crate::dart_api::sys::{
    Dart_GetError, Dart_SetNativeResolver, Dart_SetReturnValue, Dart_StringToCString,
};

#[derive(clap::Parser)]
struct Args {
    #[clap(long, default_value = if cfg!(debug_assertions) { "true" } else { "false" })]
    hmr: bool,
}

fn main() {
    let args = Args::parse();

    let engine = Runtime::initialize(RuntimeConfig {
        service_port: 5858,
        start_service_isolate: args.hmr,
    })
    .unwrap();
    let mut isolate = engine
        .load_script(
            c"./app/lib/main.dart",
            c"./app/.dart_tool/package_config.json",
            std::ptr::null_mut(),
        )
        .unwrap();

    // Start the Dart hot-reload watcher CLI (best-effort).
    // Requested command: `dart run cli/bin/cli.dart app/lib`
    let mut hot_reload_proc: Option<Child> = if args.hmr {
        Some(
            Command::new("dart")
                .args(["run", "cli/bin/cli.dart", "app/lib"])
                .stdin(Stdio::null())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .ok()
                .unwrap(),
        )
    } else {
        None
    };

    // let long_living_ptr = unsafe { Box::into_raw(Box::new(5u32)) };
    // let long_living_ptr_address = long_living_ptr as isize;
    // dbg!(long_living_ptr_address);

    // {
    //     let mut scope = isolate.enter();
    //     let class = scope.get_class("MyClass").unwrap();
    //     let class_instance = unsafe {
    //         dart_api::sys::Dart_AllocateWithNativeFields(
    //             class.raw(),
    //             1,
    //             &[long_living_ptr as isize] as *const isize,
    //         )
    //     };
    //     // check if error
    //     if unsafe { dart_api::sys::Dart_IsError(class_instance) } {
    //         let error = scope.get_error_message(class_instance);
    //         println!("Error: {}", error);
    //     }
    //     // unsafe {
    //     //     dart_api::sys::Dart_SetNativeInstanceField(
    //     //         class_instance,
    //     //         0,
    //     //         long_living_ptr as isize,
    //     //     );
    //     // }
    //     // let value = unsafe {
    //     //     let mut hello: isize = 0;
    //     //     dart_api::sys::Dart_GetNativeInstanceField(class_instance, 0, &mut hello);
    //     //     dbg!(hello);
    //     //     // let our_box_value = Box::from_raw(ptr_addr.assume_init() as *mut u32);
    //     //     // println!("our_box_value: {:?}", our_box_value);
    //     // };
    //     // class.
    //     // // let obj = isolate.
    //     // // Dart_SetNativeInstanceField
    //     // dart_api::sys::Dart_GetClass(library, class_name)
    //     // dart_api::sys::Dart_New(type_, constructor_name, number_of_arguments, arguments)
    // }
    // Basic tick loop (no window/event subsystem in this example binary).
    unsafe {
        let scope = isolate.enter();
        let library = scope.library();
        dart_api::sys::Dart_SetNativeResolver(library.raw(), Some(native_resolver), None);
    }
    for _ in 0..600 {
        let mut scope = isolate.enter();
        scope.invoke("tick", &mut []).unwrap();
        engine.drain_microtask_queue(&scope).unwrap();

        // 60 ticks
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // Clean up watcher when we exit.
    if let Some(mut child) = hot_reload_proc.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    std::process::exit(0);
}

unsafe extern "C" fn hello(args: dart_api::sys::Dart_NativeArguments) {
    let isolate = dart_api::Isolate::current().unwrap();
    // let args = dart_api::sys::Dart_GetNativeArgument(args, 0);
    let string = isolate.new_string("Hello gang!").unwrap();
    Dart_SetReturnValue(args, string.raw());
}

unsafe extern "C" fn native_resolver(
    name: dart_api::sys::Dart_Handle,
    num_of_arguments: ::std::os::raw::c_int,
    auto_setup_scope: *mut bool,
) -> dart_api::sys::Dart_NativeFunction {
    let mut cstr = MaybeUninit::<*const i8>::uninit();
    let res = Dart_StringToCString(name, cstr.as_mut_ptr());
    debug_assert!(!res.is_null(), "Dart_StringToCString returned null");
    let name = CStr::from_ptr(cstr.assume_init());
    match name.to_str().unwrap() {
        "hello" => Some(hello),
        _ => None,
    }
}
