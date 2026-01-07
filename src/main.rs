use std::process::{Child, Command, Stdio};

use clap::Parser;

use bigfish::dart::DartEngine;

#[derive(clap::Parser)]
struct Args {
    #[clap(long, default_value = if cfg!(debug_assertions) { "true" } else { "false" })]
    hmr: bool,
}

fn main() {
    let args = Args::parse();

    let mut engine = DartEngine::new(
        "./app/lib/main.dart",
        "./app/.dart_tool/package_config.json",
        args.hmr,
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

    // Basic tick loop (no window/event subsystem in this example binary).
    for _ in 0..600 {
        let mut isolate = engine.isolate().enter();
        isolate.invoke("tick", &mut []).unwrap();
        isolate.drain_microtask_queue();

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
