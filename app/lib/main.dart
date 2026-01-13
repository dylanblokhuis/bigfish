import 'dart:io';

import 'package:app/world.dart';

import 'native.dart';

class SimpleRaster {
  late RenderPipeline renderPipeline;

  SimpleRaster(Gpu gpu) {
    final metalShader = File("./app/shaders/Shaders.metal").readAsStringSync();
    final descriptor = RenderPipelineDescriptor(
      colorAttachments: [
        RenderPipelineDescriptorColorAttachment(
          pixelFormat: PixelFormat.bgra8Unorm,
        ),
      ],
      vertexShader: ShaderLibrary(
        source: metalShader,
        entryPoint: 'vertexShader',
      ),
      fragmentShader: ShaderLibrary(
        source: metalShader,
        entryPoint: 'fragmentShader',
      ),
    );
    renderPipeline = gpu.compileRenderPipeline(descriptor);
  }
}

void main() {
  final window = Window(width: 800, height: 600, title: 'Hello World');
  final gpu = Gpu(window);

  final world = World();
  world.insertResource(SimpleRaster(gpu));
  window.onUpdate(() => update(world));
  window.onPresent((interpolation) => present(world, gpu, interpolation));

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
  final simpleRaster = world.getResource<SimpleRaster>();
  // print("Present! $interpolation");
  final commandBuffer = gpu.beginCommandBuffer();
  final renderCommandEncoder = commandBuffer.renderCommandEncoder();
  renderCommandEncoder.setRenderPipeline(simpleRaster.renderPipeline);
  renderCommandEncoder.setViewport(
    Viewport(x: 0, y: 0, width: 800, height: 600),
  );
  renderCommandEncoder.endEncoding();
  gpu.endCommandBuffer(commandBuffer);
}
