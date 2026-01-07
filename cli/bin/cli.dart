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

  bool connected = await client.connect();
  if (!connected) {
    Logger.error('Failed to connect to Dart VM');
    return;
  }

  Logger.success('Connected to Dart VM');

  Logger.info('Watching: ${watchDir.absolute.path}');

  StreamSubscription<FileSystemEvent>? sub;
  Timer? debounce;
  bool reloadInFlight = false;
  bool reloadQueued = false;

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

  Future<void> shutdown({int code = 0}) async {
    debounce?.cancel();
    await sub?.cancel();
    await client.disconnect();
    exit(code);
  }

  ProcessSignal.sigint.watch().listen((_) async {
    Logger.info('Stopping...');
    await shutdown(code: 0);
  });

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
  await Completer<void>().future;
}
