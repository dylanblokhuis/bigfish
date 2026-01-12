import 'package:app/world.dart';

import 'native.dart';

void main() {
  final window = Window(width: 800, height: 600, title: 'Hello World');
  final gpu = Gpu(window);

  final world = World();
  window.onUpdate(() => update(world));
  window.onPresent((interpolation) => present(world, gpu, interpolation));

  // we poll here because if we use a long lived function, we cant hot reload anything
  while (window.poll()) {}
}

// update game logic at 60 ticks
void update(World world) {
  // print(time);
  world.spawn();

  // final child = world.spawn();
  // world.addChild(root, child);

  // print(world);
}

// we can render here, will loop as fast as possible, with the interpolation value being the amount of time that has passed since the last update
// can be used to interpolate values to not have janky movement
void present(World world, Gpu gpu, double interpolation) {
  // print("Present! $interpolation");
  final commandBuffer = gpu.beginCommandBuffer();
  gpu.endCommandBuffer(commandBuffer);
}
