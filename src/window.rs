use bigfish_macros::native_func;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::dart_api::{native_resolver, sys, Isolate, NativeArguments, PersistentHandle};

pub struct Window {
    ctx: sdl3::Sdl,
    window: sdl3::video::Window,
    update_callback: Option<PersistentHandle>,
    present_callback: Option<PersistentHandle>,
}

// Safety: SDL windows are not thread-safe, but we protect all access with a Mutex.
// This ensures only one thread can access the window at a time, making it safe to Send/Sync.
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

#[native_func]
fn create_window(args: NativeArguments) {
    println!("arg count: {}", args.get_arg_count());
    let instance = args.get_arg(0).unwrap();
    let width = args.get_integer_arg(1).unwrap();
    let height = args.get_integer_arg(2).unwrap();
    let title = args.get_string_arg(3).unwrap().to_string_lossy().unwrap();

    let ctx = sdl3::init().unwrap();
    let window = ctx
        .video()
        .unwrap()
        .window(&title, width as u32, height as u32)
        .build()
        .unwrap();

    let window_struct = Box::new(Window {
        ctx,
        window,
        update_callback: None,
        present_callback: None,
    });

    // Also set up finalizable handle for cleanup
    instance.set_peer(window_struct);
    // instance.new_finalizable_handle(window_struct
}

#[native_func]
fn set_update_callback(args: NativeArguments) {
    let instance = args.get_arg(0).unwrap();
    let callback = args.get_arg(1).unwrap();

    if !callback.is_closure() {
        eprintln!("setUpdateCallback: callback must be a closure");
        return;
    }

    let persistent = match PersistentHandle::new(callback) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create persistent handle: {:?}", e);
            return;
        }
    };

    let window_ptr = instance.get_peer::<Window>().unwrap();
    let window = unsafe { &mut *(window_ptr as *mut Window) };
    window.update_callback = Some(persistent);
}

#[native_func]
fn set_present_callback(args: NativeArguments) {
    let instance = args.get_arg(0).unwrap();
    let callback = args.get_arg(1).unwrap();

    if !callback.is_closure() {
        eprintln!("setPresentCallback: callback must be a closure");
        return;
    }

    let persistent = match PersistentHandle::new(callback) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create persistent handle: {:?}", e);
            return;
        }
    };

    let window_ptr = instance.get_peer::<Window>().unwrap();
    let window = unsafe { &mut *(window_ptr as *mut Window) };
    window.present_callback = Some(persistent);
}

#[native_func]
fn poll(args: NativeArguments) {
    let instance = args.get_arg(0).unwrap();
    let window = instance.get_peer::<Window>().unwrap();

    let mut should_continue = true;
    for event in window.ctx.event_pump().unwrap().poll_iter() {
        match event {
            sdl3::event::Event::Quit { .. } => should_continue = false,
            _ => {}
        }
    }

    args.set_bool_return_value(should_continue);

    // Call update callback
    {
        if let Some(ref update_cb) = window.update_callback {
            let mut callback_args: [crate::dart_api::sys::Dart_Handle; 0] = [];
            let result = unsafe {
                crate::dart_api::sys::Dart_InvokeClosure(
                    update_cb.raw(),
                    callback_args.len() as i32,
                    callback_args.as_mut_ptr(),
                )
            };
            // Check for errors but don't fail the loop
            if unsafe { crate::dart_api::sys::Dart_IsError(result) } {
                eprintln!("Error in update callback");
            }
        }
    }

    // Call present callback
    {
        if let Some(ref present_cb) = window.present_callback {
            let mut callback_args: [crate::dart_api::sys::Dart_Handle; 0] = [];
            let result = unsafe {
                crate::dart_api::sys::Dart_InvokeClosure(
                    present_cb.raw(),
                    callback_args.len() as i32,
                    callback_args.as_mut_ptr(),
                )
            };
            // Check for errors but don't fail the loop
            if unsafe { crate::dart_api::sys::Dart_IsError(result) } {
                eprintln!("Error in present callback");
            }
        }
    }
}
