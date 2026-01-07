use std::ffi::CString;
use std::process::{Child, Command, Stdio};

use clap::Parser;

use crate::bindings::{
    Dart_EnterIsolate, Dart_EnterScope, Dart_ExitIsolate, Dart_ExitScope, Dart_Invoke,
    Dart_IsError, Dart_NewStringFromCString, Dart_RootLibrary, Dart_ShutdownIsolate,
    DartDll_DrainMicrotaskQueue, DartDll_Initialize, DartDll_LoadScript, DartDll_Shutdown,
    DartDllConfig,
};

mod bindings;

#[derive(clap::Parser)]
struct Args {
    #[clap(long, short, default_value = if cfg!(debug_assertions) { "true" } else { "false" })]
    hmr: bool,
}

fn main() {
    let args = Args::parse();
    let sdl3 = sdl3::init().unwrap();
    let video_subsystem = sdl3.video().unwrap();
    let _window = video_subsystem.window("Dart", 800, 600).build().unwrap();

    let config = DartDllConfig {
        service_port: 5858,
        start_service_isolate: args.hmr,
    };

    let isolate = unsafe {
        DartDll_Initialize(&config);

        let package_config = CString::new("./app/.dart_tool/package_config.json").unwrap();
        let script_path = CString::new("./app/lib/main.dart").unwrap();

        let isolate = DartDll_LoadScript(
            script_path.as_ptr(),
            package_config.as_ptr(),
            std::ptr::null_mut(),
        );

        if isolate.is_null() {
            DartDll_Shutdown();
            panic!("Failed to load script");
        }

        Dart_EnterIsolate(isolate);
        Dart_EnterScope();

        let library = Dart_RootLibrary();
        if Dart_IsError(library) {
            Dart_ExitScope();
            Dart_ShutdownIsolate();
            DartDll_Shutdown();
            panic!("Failed to get root library");
        }

        let result = Dart_Invoke(
            library,
            Dart_NewStringFromCString(c"main".as_ptr()),
            0,
            std::ptr::null_mut(),
        );

        if Dart_IsError(result) {
            Dart_ExitScope();
            Dart_ShutdownIsolate();
            DartDll_Shutdown();
            panic!("Failed to invoke main");
        }

        DartDll_DrainMicrotaskQueue();
        Dart_ExitScope();
        Dart_ExitIsolate();

        isolate
    };

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

    'main_loop: loop {
        for event in sdl3.event_pump().unwrap().poll_iter() {
            if let sdl3::event::Event::Quit { .. } = event {
                break 'main_loop;
            }
        }

        unsafe {
            Dart_EnterIsolate(isolate);
            Dart_EnterScope();
            let library = Dart_RootLibrary();

            Dart_Invoke(
                library,
                Dart_NewStringFromCString(c"tick".as_ptr()),
                0,
                std::ptr::null_mut(),
            );
            DartDll_DrainMicrotaskQueue();
            Dart_ExitScope();
            Dart_ExitIsolate();
        }

        // 60 ticks
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    // Clean up watcher when we exit.
    if let Some(mut child) = hot_reload_proc.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}
