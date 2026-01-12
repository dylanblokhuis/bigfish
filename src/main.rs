use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use clap::Parser;

mod dart_api;
mod gpu;
mod window;
use dart_api::{Runtime, RuntimeConfig};

use crate::dart_api::native_resolver;

#[derive(clap::Parser)]
struct Args {
    #[clap(long, default_value = if cfg!(debug_assertions) { "true" } else { "false" })]
    hmr: bool,
}

fn main() {
    let args = Args::parse();

    // If we spawn the Dart hot-reload watcher, ensure Ctrl+C always kills it.
    // Without this, interrupting the Rust process can leave the Dart process running.
    let hot_reload_proc: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    {
        let hot_reload_proc = Arc::clone(&hot_reload_proc);
        ctrlc::set_handler(move || {
            eprintln!("\nInterrupted. Shutting down...");
            if let Ok(mut guard) = hot_reload_proc.lock() {
                if let Some(mut child) = guard.take() {
                    let _ = child.kill();
                }
            }
            std::process::exit(130);
        })
        .expect("failed to set Ctrl+C handler");
    }

    let engine = Runtime::initialize(RuntimeConfig {
        service_port: 5858,
        start_service_isolate: args.hmr,
    })
    .unwrap();

    let mut isolate = engine
        .load_script(
            c"./app/lib/main.dart",
            c"./app/.dart_tool/package_config.json",
        )
        .unwrap();

    // Start the Dart hot-reload watcher CLI (best-effort).
    // Requested command: `dart run cli/bin/cli.dart app/lib`
    if args.hmr {
        let child = Command::new("dart")
            .args(["run", "cli/bin/cli.dart", "app/lib"])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("failed to spawn hot-reload watcher");

        *hot_reload_proc.lock().expect("poisoned watcher lock") = Some(child);
    }

    {
        let mut scope = isolate.enter();
        let library = scope.library("package:app/native.dart").unwrap();
        scope.set_native_resolver(library, Some(native_resolver));

        let root_library = scope.library("package:app/main.dart").unwrap();
        scope.invoke(root_library, "main", &mut []).unwrap();
    }

    println!("Exiting...");

    // Clean up watcher when we exit.
    if let Some(mut child) = hot_reload_proc
        .lock()
        .expect("poisoned watcher lock")
        .take()
    {
        let _ = child.kill();
        let _ = child.wait();
    }
    std::process::exit(0);
}
