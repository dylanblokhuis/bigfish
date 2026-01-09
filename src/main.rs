use std::process::{Child, Command, Stdio};

use clap::Parser;

mod dart_api;
mod gpu;
mod window;
use dart_api::{Runtime, RuntimeConfig};

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

    isolate.enter().invoke("main", &mut []).unwrap();

    println!("Exiting...");

    // Clean up watcher when we exit.
    if let Some(mut child) = hot_reload_proc.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    std::process::exit(0);
}
