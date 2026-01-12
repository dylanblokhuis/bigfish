import 'dart:async';
import 'dart:nativewrappers';

base class Window extends NativeFieldWrapperClass1 {
  Window({required int width, required int height, required String title}) {
    createWindow(width, height, title);
  }

  @pragma('vm:external-name', 'create_window')
  external void createWindow(int width, int height, String title);

  @pragma('vm:external-name', 'on_update')
  external void onUpdate(void Function() callback);

  @pragma('vm:external-name', 'on_present')
  external void onPresent(void Function(double interpolation) callback);

  @pragma('vm:external-name', 'poll')
  external bool poll();
}

base class Gpu extends NativeFieldWrapperClass1 {
  Gpu(Window window) {
    _initGpu(window);
  }

  @pragma('vm:external-name', 'init_gpu')
  external void _initGpu(Window window);

  @pragma('vm:external-name', 'begin_command_buffer')
  external CommandBuffer beginCommandBuffer();

  @pragma('vm:external-name', 'end_command_buffer')
  external void endCommandBuffer(CommandBuffer commandBuffer);

  @pragma('vm:external-name', 'gpu_draw')
  external void draw();
}

@pragma("vm:entry-point")
base class CommandBuffer extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  CommandBuffer();

  @pragma("vm:entry-point")
  external Gpu gpu;
}
