import 'hot_reload_client.dart';
import 'logger.dart';

import 'dart:async';
import 'dart:io';

Future<void> main(List<String> args) async {
  if (args.isEmpty) {
    Logger.error('Missing directory to watch.');
    Logger.info('Usage: dart run cli <directory>');
    exitCode = 64; // EX_USAGE
    return;
  }

  final watchPath = args.first;
  final watchDir = Directory(watchPath);
  if (!watchDir.existsSync()) {
    Logger.error('Directory does not exist: $watchPath');
    exitCode = 66; // EX_NOINPUT
    return;
  }

  final client = HotReloadClient();

  StreamSubscription<FileSystemEvent>? sub;
  Timer? debounce;
  bool reloadInFlight = false;
  bool reloadQueued = false;

  final done = Completer<void>();
  bool shuttingDown = false;

  Future<void> cleanup() async {
    debounce?.cancel();
    debounce = null;

    // Directory.watch cancellation can occasionally hang; treat as best-effort.
    try {
      await sub?.cancel().timeout(const Duration(milliseconds: 250));
    } catch (_) {
      // ignore
    }
    sub = null;

    // vm_service dispose can hang if the websocket is in a bad state; best-effort.
    try {
      await client.disconnect().timeout(const Duration(milliseconds: 250));
    } catch (_) {
      // ignore
    }
  }

  void handleSignal(int code) {
    if (shuttingDown) return;
    shuttingDown = true;

    Logger.info('Stopping...');

    // Stop any in-flight/retrying connect attempt ASAP.
    client.cancel();

    // Ensure we always exit, even if cleanup hangs.
    Timer(const Duration(seconds: 1), () => exit(code));

    unawaited(
      cleanup().whenComplete(() {
        if (!done.isCompleted) done.complete();
        exit(code);
      }),
    );
  }

  ProcessSignal.sigint.watch().listen((_) => handleSignal(130));
  ProcessSignal.sigterm.watch().listen((_) => handleSignal(143));

  bool connected = await client.connect();
  if (!connected) {
    Logger.error('Failed to connect to Dart VM');
    return;
  }

  Logger.success('Connected to Dart VM');

  Logger.info('Watching: ${watchDir.absolute.path}');

  Future<void> triggerReload() async {
    if (reloadInFlight) {
      reloadQueued = true;
      return;
    }
    reloadInFlight = true;
    try {
      Logger.info('Reloading...');
      await client.reload();
    } finally {
      reloadInFlight = false;
      if (reloadQueued) {
        reloadQueued = false;
        // Coalesce multiple changes that happened during a reload into one more reload.
        unawaited(triggerReload());
      }
    }
  }

  sub = watchDir
      .watch(recursive: true)
      .listen(
        (a) {
          print('File changed: ${a.path}');
          debounce?.cancel();
          debounce = Timer(const Duration(milliseconds: 250), () {
            unawaited(triggerReload());
          });
        },
        onError: (Object e) {
          Logger.error('File watcher error: $e');
        },
      );

  // Keep the process alive while the watcher is running.
  await done.future;
}
