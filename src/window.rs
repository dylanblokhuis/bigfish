use bigfish_macros::native_func;

use crate::dart_api::{Isolate, NativeArguments, PersistentHandle};

pub struct Window {
    ctx: sdl3::Sdl,
    #[allow(dead_code)]
    window: sdl3::video::Window,
    #[cfg(target_os = "macos")]
    metal_view: sdl3::sys::metal::SDL_MetalView,
    #[cfg(target_os = "macos")]
    metal_layer: objc2::rc::Retained<objc2_quartz_core::CAMetalLayer>,
    update_callback: Option<PersistentHandle>,
    present_callback: Option<PersistentHandle>,
    clock: chron::Clock,
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

    use std::num::NonZeroU32;

    let updates_per_second = NonZeroU32::new(60).unwrap();

    let clock = chron::Clock::new(updates_per_second);

    #[cfg(target_os = "macos")]
    let (metal_view, metal_layer) = {
        use objc2::rc::Retained;
        use objc2_quartz_core::CAMetalLayer;

        // Create a CAMetalLayer-backed view and attach it to the SDL window.
        //
        // On macOS, SDL does not automatically associate an MTLDevice with the CAMetalLayer;
        // we do that later when initializing the GPU.
        let metal_view = unsafe { sdl3::sys::metal::SDL_Metal_CreateView(window.raw()) };
        if metal_view.is_null() {
            panic!("SDL_Metal_CreateView returned null");
        }

        let layer_ptr =
            unsafe { sdl3::sys::metal::SDL_Metal_GetLayer(metal_view) } as *mut CAMetalLayer;
        let metal_layer =
            unsafe { Retained::retain(layer_ptr) }.expect("SDL_Metal_GetLayer returned null");

        (metal_view, metal_layer)
    };

    let window_struct = Box::new(Window {
        ctx,
        window,
        #[cfg(target_os = "macos")]
        metal_view,
        #[cfg(target_os = "macos")]
        metal_layer,
        update_callback: None,
        present_callback: None,
        clock,
    });

    // Also set up finalizable handle for cleanup
    instance.set_peer(window_struct);
    // instance.new_finalizable_handle(window_struct
}

#[cfg(target_os = "macos")]
impl Window {
    pub fn metal_layer(&self) -> &objc2_quartz_core::CAMetalLayer {
        &self.metal_layer
    }

    pub fn size(&self) -> (u32, u32) {
        self.window.size()
    }
}

#[cfg(target_os = "macos")]
impl Drop for Window {
    fn drop(&mut self) {
        // SDL requires destroying the Metal view before destroying the window.
        unsafe {
            if !self.metal_view.is_null() {
                sdl3::sys::metal::SDL_Metal_DestroyView(self.metal_view);
                self.metal_view = std::ptr::null_mut();
            }
        }
    }
}

#[native_func]
fn on_update(args: NativeArguments) {
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
fn on_present(args: NativeArguments) {
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

    if let Some(tick) = window.clock.next() {
        match tick {
            chron::Tick::Update => {
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
            chron::Tick::Render { interpolation } => {
                if let Some(ref present_cb) = window.present_callback {
                    let scope = Isolate::current().unwrap();
                    let interpolation_value = scope.new_double(interpolation as f64).unwrap();

                    let mut callback_args: [crate::dart_api::sys::Dart_Handle; 1] =
                        [interpolation_value.raw()];
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
    }
}
