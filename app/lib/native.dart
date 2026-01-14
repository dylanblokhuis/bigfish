import 'dart:nativewrappers';
import 'dart:typed_data';

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

@pragma("vm:entry-point")
base class Texture extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  Texture();

  @pragma('vm:external-name', 'Texture_width')
  external int width();

  @pragma('vm:external-name', 'Texture_height')
  external int height();

  @pragma('vm:external-name', 'Texture_pixel_format')
  external int pixelFormat();

  @pragma('vm:external-name', 'Gpu_replace_region')
  external void _replaceRegion(
    Texture texture,
    int regionX,
    int regionY,
    int regionZ,
    int regionWidth,
    int regionHeight,
    int regionDepth,
    int mipmapLevel,
    Uint8List bytes,
    int bytesPerRow, [
    int bytesPerImage = 0,
  ]);

  void replaceRegion({
    required Texture texture,
    required int regionX,
    required int regionY,
    required int regionZ,
    required int regionWidth,
    required int regionHeight,
    required int regionDepth,
    required int mipmapLevel,
    required Uint8List bytes,
    required int bytesPerRow,
    int bytesPerImage = 0,
  }) {
    _replaceRegion(
      texture,
      regionX,
      regionY,
      regionZ,
      regionWidth,
      regionHeight,
      regionDepth,
      mipmapLevel,
      bytes,
      bytesPerRow,
      bytesPerImage,
    );
  }
}

base class Gpu extends NativeFieldWrapperClass1 {
  Gpu(Window window) {
    _initGpu(window);
  }

  @pragma('vm:external-name', 'Gpu_init')
  external void _initGpu(Window window);

  @pragma('vm:external-name', 'Gpu_begin_command_buffer')
  external CommandBuffer beginCommandBuffer();

  @pragma('vm:external-name', 'Gpu_end_command_buffer')
  external void endCommandBuffer(CommandBuffer commandBuffer);

  @pragma('vm:external-name', 'Gpu_compile_render_pipeline')
  external RenderPipeline compileRenderPipeline(
    RenderPipelineDescriptor descriptor,
  );

  @pragma('vm:external-name', 'Gpu_compile_compute_pipeline')
  external ComputePipeline compileComputePipeline(
    ComputePipelineDescriptor descriptor,
  );

  @pragma('vm:external-name', 'Gpu_create_buffer')
  external Buffer createBuffer(int length, [int storageMode = 0]);

  @pragma('vm:external-name', 'Gpu_add_buffer_to_residency_set')
  external void addBufferToResidencySet(Buffer buffer);

  @pragma('vm:external-name', 'Gpu_add_texture_to_residency_set')
  external void addTextureToResidencySet(Texture texture);

  @pragma('vm:external-name', 'Gpu_commit_residency_set')
  external void commitResidencySet();

  @pragma('vm:external-name', 'Gpu_create_argument_table')
  external ArgumentTable _createArgumentTable(
    int maxBufferBindCount,
    int maxTextureBindCount,
    int maxSamplerStateBindCount,
  );

  ArgumentTable createArgumentTable({
    int maxBufferBindCount = 0,
    int maxTextureBindCount = 0,
    int maxSamplerStateBindCount = 0,
  }) {
    return _createArgumentTable(
      maxBufferBindCount,
      maxTextureBindCount,
      maxSamplerStateBindCount,
    );
  }

  @pragma('vm:external-name', 'Gpu_create_texture')
  external Texture createTexture(int width, int height, int pixelFormat);
}

@pragma("vm:entry-point")
base class ArgumentTable extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  ArgumentTable();

  @pragma('vm:external-name', 'ArgumentTable_set_buffer')
  external void setBuffer(Buffer buffer, int index, [int offset = 0]);

  @pragma('vm:external-name', 'ArgumentTable_set_texture')
  external void setTexture(Texture texture, int index);
}

@pragma("vm:entry-point")
base class CommandBuffer extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  CommandBuffer(this.gpu);

  @pragma("vm:entry-point")
  Gpu gpu;

  @pragma('vm:external-name', 'CommandBuffer_render_command_encoder')
  external RenderCommandEncoder _renderCommandEncoder(
    RenderPassDescriptor descriptor,
  );

  RenderCommandEncoder renderCommandEncoder(RenderPassDescriptor descriptor) {
    return _renderCommandEncoder(descriptor);
  }

  @pragma('vm:external-name', 'CommandBuffer_compute_command_encoder')
  external ComputeCommandEncoder computeCommandEncoder();

  @pragma('vm:external-name', 'CommandBuffer_drawable')
  external Texture drawable();
}

@pragma("vm:entry-point")
base class RenderPipeline extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  RenderPipeline();
}

@pragma("vm:entry-point")
base class ComputePipeline extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  ComputePipeline();
}

@pragma("vm:entry-point")
base class Buffer extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  Buffer();

  @pragma('vm:external-name', 'Buffer_length')
  external int length();

  @pragma('vm:external-name', 'Buffer_gpu_address')
  external int gpuAddress();

  @pragma('vm:external-name', 'Buffer_contents')
  external Uint8List contents();

  @pragma('vm:external-name', 'Buffer_set_contents')
  external void setContents(Uint8List data);

  @pragma('vm:external-name', 'Buffer_label')
  external String? label();

  @pragma('vm:external-name', 'Buffer_set_label')
  external void setLabel(String label);
}

@pragma("vm:entry-point")
base class RenderCommandEncoder extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  RenderCommandEncoder();

  @pragma('vm:external-name', 'RenderCommandEncoder_set_render_pipeline')
  external void setRenderPipeline(RenderPipeline renderPipeline);

  @pragma('vm:external-name', 'RenderCommandEncoder_set_viewport')
  external void _setViewport(double x, double y, double width, double height);

  void setViewport({
    required double width,
    required double height,
    double x = 0,
    double y = 0,
  }) {
    _setViewport(x, y, width, height);
  }

  @pragma('vm:external-name', 'RenderCommandEncoder_set_scissor_rect')
  external void _setScissorRect(int x, int y, int width, int height);

  void setScissorRect({
    required int width,
    required int height,
    int x = 0,
    int y = 0,
  }) {
    _setScissorRect(x, y, width, height);
  }

  @pragma('vm:external-name', 'RenderCommandEncoder_set_cull_mode')
  external void _setCullMode(int mode);

  void setCullMode(CullMode mode) => _setCullMode(mode.value);

  @pragma('vm:external-name', 'RenderCommandEncoder_draw_primitives')
  external void _drawPrimitives(
    int primitiveType,
    int vertexCount,
    int instanceCount,
    int baseVertex,
    int baseInstance,
  );

  void drawPrimitives({
    required PrimitiveType primitiveType,
    required int vertexCount,
    required int instanceCount,
    int baseVertex = 0,
    int baseInstance = 0,
  }) {
    _drawPrimitives(
      primitiveType.value,
      vertexCount,
      instanceCount,
      baseVertex,
      baseInstance,
    );
  }

  // @pragma("vm:external-name", "RenderCommandEncoder_draw_indexed_primitives")
  // external void _drawIndexedPrimitives(
  //   int primitiveType,
  //   int indexCount,
  //   int instanceCount,
  //   int baseVertex,
  //   int baseInstance,
  // );

  // void drawIndexedPrimitives({
  //   required PrimitiveType primitiveType,
  //   required int indexCount,
  //   required IndexType indexType,
  //   required int instanceCount,
  //   int baseVertex = 0,
  //   int baseInstance = 0,
  // }) {
  //   _drawIndexedPrimitives(
  //     primitiveType.value,
  //     indexCount,
  //     instanceCount,
  //     baseVertex,
  //     baseInstance,
  //   );
  // }

  @pragma('vm:external-name', 'RenderCommandEncoder_set_argument_table_object')
  external void setArgumentTableObject(ArgumentTable argumentTable);

  @pragma('vm:external-name', 'RenderCommandEncoder_end_encoding')
  external void endEncoding();
}

@pragma("vm:entry-point")
base class ComputeCommandEncoder extends NativeFieldWrapperClass1 {
  @pragma("vm:entry-point")
  ComputeCommandEncoder();

  @pragma('vm:external-name', 'ComputeCommandEncoder_end_encoding')
  external void endEncoding();

  @pragma('vm:external-name', 'ComputeCommandEncoder_set_compute_pipeline')
  external void setComputePipeline(ComputePipeline computePipeline);

  @pragma('vm:external-name', 'ComputeCommandEncoder_set_argument_table_object')
  external void setArgumentTableObject(ArgumentTable argumentTable);

  /// threadsPerGrid - [texture.width, texture.height, texture.depth]
  ///
  /// threadsPerThreadgroupY - (numthreadgroups) [8, 8, 1]
  ///
  /// This should handle partial edges automatically
  @pragma('vm:external-name', 'ComputeCommandEncoder_dispatch_threads')
  external void dispatchThreads(
    int threadsPerGridX,
    int threadsPerGridY,
    int threadsPerGridZ,
    int threadsPerThreadgroupX,
    int threadsPerThreadgroupY,
    int threadsPerThreadgroupZ,
  );

  @pragma('vm:external-name', 'ComputeCommandEncoder_dispatch_threads')
  external void dispatchThreadgroups(
    int threadgroupsPerGridX,
    int threadgroupsPerGridY,
    int threadgroupsPerGridZ,
    int threadsPerThreadgroupX,
    int threadsPerThreadgroupY,
    int threadsPerThreadgroupZ,
  );
}

enum CullMode {
  none(0),
  front(1),
  back(2);

  final int value;
  const CullMode(this.value);
}

enum IndexType {
  uint16(0),
  uint32(1);

  final int value;
  const IndexType(this.value);
}

enum PrimitiveType {
  point(0),
  line(1),
  lineStrip(2),
  triangle(3),
  triangleStrip(4);

  final int value;
  const PrimitiveType(this.value);
}

class Viewport {
  Viewport({
    required this.x,
    required this.y,
    required this.width,
    required this.height,
  });

  final double x;
  final double y;
  final double width;
  final double height;

  @pragma("vm:entry-point")
  Map<String, dynamic> toMap() {
    return {'x': x, 'y': y, 'width': width, 'height': height};
  }
}

class RenderPipelineDescriptor {
  String label = "Unnamed Render Pipeline Descriptor";
  List<RenderPipelineDescriptorColorAttachment> colorAttachments;
  PixelFormat depthAttachmentPixelFormat = PixelFormat.invalid;
  PixelFormat stencilAttachmentPixelFormat = PixelFormat.invalid;
  PrimitiveTopology primitiveTopology = PrimitiveTopology.unspecified;
  ShaderLibrary vertexShader;
  ShaderLibrary fragmentShader;

  RenderPipelineDescriptor({
    required this.colorAttachments,
    required this.vertexShader,
    required this.fragmentShader,
    this.depthAttachmentPixelFormat = PixelFormat.invalid,
    this.stencilAttachmentPixelFormat = PixelFormat.invalid,
    this.primitiveTopology = PrimitiveTopology.triangle,
    this.label = "Unnamed Render Pipeline Descriptor",
  });

  @pragma("vm:entry-point")
  Map<String, dynamic> toMap() {
    return {
      'label': label,
      'colorAttachments': colorAttachments.map((e) => e.toMap()).toList(),
      'depthAttachmentPixelFormat': depthAttachmentPixelFormat.value,
      'stencilAttachmentPixelFormat': stencilAttachmentPixelFormat.value,
      'primitiveTopology': primitiveTopology.value,
      'vertexShader': vertexShader.toMap(),
      'fragmentShader': fragmentShader.toMap(),
    };
  }
}

class ComputePipelineDescriptor {
  String label = "Unnamed Compute Pipeline Descriptor";
  ShaderLibrary computeShader;
  ComputePipelineDescriptor({required this.computeShader});
  @pragma("vm:entry-point")
  Map<String, dynamic> toMap() {
    return {'label': label, 'computeShader': computeShader.toMap()};
  }
}

class ShaderLibrary {
  String path;
  String entryPoint;
  ShaderLibrary({required this.path, required this.entryPoint});

  Map<String, dynamic> toMap() {
    return {'path': path, 'entryPoint': entryPoint};
  }
}

class RenderPipelineDescriptorColorAttachment {
  PixelFormat pixelFormat;
  ColorWriteMask writeMask = ColorWriteMask.all;
  bool blendEnabled = false;
  BlendOp rgbBlendOp = BlendOp.add;
  BlendOp alphaBlendOp = BlendOp.add;
  BlendFactor sourceAlphaBlendFactor = BlendFactor.one;
  BlendFactor destinationAlphaBlendFactor = BlendFactor.zero;
  BlendFactor sourceRgbBlendFactor = BlendFactor.one;
  BlendFactor destinationRgbBlendFactor = BlendFactor.zero;
  RenderPipelineDescriptorColorAttachment({required this.pixelFormat});

  Map<String, dynamic> toMap() {
    return {
      'pixelFormat': pixelFormat.value,
      'writeMask': writeMask.rawValue,
      'blendEnabled': blendEnabled,
      'rgbBlendOp': rgbBlendOp.index,
      'alphaBlendOp': alphaBlendOp.index,
      'sourceAlphaBlendFactor': sourceAlphaBlendFactor.index,
      'destinationAlphaBlendFactor': destinationAlphaBlendFactor.index,
      'sourceRgbBlendFactor': sourceRgbBlendFactor.index,
      'destinationRgbBlendFactor': destinationRgbBlendFactor.index,
    };
  }
}

enum BlendOp { add, subtract, reverseSubtract, min, max }

enum PixelFormat {
  invalid(0),
  a8Unorm(1),
  r8Unorm(10),
  r8UnormSrgb(11),
  r8Snorm(12),
  r8Uint(13),
  r8Sint(14),
  r16Unorm(20),
  r16Snorm(22),
  r16Uint(23),
  r16Sint(24),
  r16Float(25),
  rg8Unorm(30),
  rg8UnormSrgb(31),
  rg8Snorm(32),
  rg8Uint(33),
  rg8Sint(34),
  b5g6r5Unorm(40),
  a1bgr5Unorm(41),
  abgr4Unorm(42),
  bgr5a1Unorm(43),
  r32Uint(53),
  r32Sint(54),
  r32Float(55),
  rg16Unorm(60),
  rg16Snorm(62),
  rg16Uint(63),
  rg16Sint(64),
  rg16Float(65),
  rgba8Unorm(70),
  rgba8UnormSrgb(71),
  rgba8Snorm(72),
  rgba8Uint(73),
  rgba8Sint(74),
  bgra8Unorm(80),
  bgra8UnormSrgb(81),
  rgb10a2Unorm(90),
  rgb10a2Uint(91),
  rg11b10Float(92),
  rgb9e5Float(93),
  bgr10a2Unorm(94),
  bgr10Xr(554),
  bgr10XrSrgb(555),
  rg32Uint(103),
  rg32Sint(104),
  rg32Float(105),
  rgba16Unorm(110),
  rgba16Snorm(112),
  rgba16Uint(113),
  rgba16Sint(114),
  rgba16Float(115),
  bgra10Xr(552),
  bgra10XrSrgb(553),
  rgba32Uint(123),
  rgba32Sint(124),
  rgba32Float(125),
  bc1Rgba(130),
  bc1RgbaSrgb(131),
  bc2Rgba(132),
  bc2RgbaSrgb(133),
  bc3Rgba(134),
  bc3RgbaSrgb(135),
  bc4RUnorm(140),
  bc4RSnorm(141),
  bc5RgUnorm(142),
  bc5RgSnorm(143),
  bc6hRgbFloat(150),
  bc6hRgbUfloat(151),
  bc7RgbaUnorm(152),
  bc7RgbaUnormSrgb(153),

  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgb2bpp(160),
  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgb2bppSrgb(161),
  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgb4bpp(162),
  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgb4bppSrgb(163),
  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgba2bpp(164),
  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgba2bppSrgb(165),
  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgba4bpp(166),
  @Deprecated('Usage of ASTC/ETC2/BC formats is recommended instead.')
  pvrtcRgba4bppSrgb(167),

  eacR11Unorm(170),
  eacR11Snorm(172),
  eacRg11Unorm(174),
  eacRg11Snorm(176),
  eacRgba8(178),
  eacRgba8Srgb(179),
  etc2Rgb8(180),
  etc2Rgb8Srgb(181),
  etc2Rgb8a1(182),
  etc2Rgb8a1Srgb(183),
  astc4x4Srgb(186),
  astc5x4Srgb(187),
  astc5x5Srgb(188),
  astc6x5Srgb(189),
  astc6x6Srgb(190),
  astc8x5Srgb(192),
  astc8x6Srgb(193),
  astc8x8Srgb(194),
  astc10x5Srgb(195),
  astc10x6Srgb(196),
  astc10x8Srgb(197),
  astc10x10Srgb(198),
  astc12x10Srgb(199),
  astc12x12Srgb(200),
  astc4x4Ldr(204),
  astc5x4Ldr(205),
  astc5x5Ldr(206),
  astc6x5Ldr(207),
  astc6x6Ldr(208),
  astc8x5Ldr(210),
  astc8x6Ldr(211),
  astc8x8Ldr(212),
  astc10x5Ldr(213),
  astc10x6Ldr(214),
  astc10x8Ldr(215),
  astc10x10Ldr(216),
  astc12x10Ldr(217),
  astc12x12Ldr(218),
  astc4x4Hdr(222),
  astc5x4Hdr(223),
  astc5x5Hdr(224),
  astc6x5Hdr(225),
  astc6x6Hdr(226),
  astc8x5Hdr(228),
  astc8x6Hdr(229),
  astc8x8Hdr(230),
  astc10x5Hdr(231),
  astc10x6Hdr(232),
  astc10x8Hdr(233),
  astc10x10Hdr(234),
  astc12x10Hdr(235),
  astc12x12Hdr(236),
  gbgr422(240),
  bgrg422(241),
  depth16Unorm(250),
  depth32Float(252),
  stencil8(253),
  depth24UnormStencil8(255),
  depth32FloatStencil8(260),
  x32Stencil8(261),
  x24Stencil8(262),
  unspecialized(263);

  final int value;
  const PixelFormat(this.value);
}

extension type const ColorWriteMask(int rawValue) {
  static const ColorWriteMask none = ColorWriteMask(0);
  static const ColorWriteMask red = ColorWriteMask(0x1 << 3);
  static const ColorWriteMask green = ColorWriteMask(0x1 << 2);
  static const ColorWriteMask blue = ColorWriteMask(0x1 << 1);
  static const ColorWriteMask alpha = ColorWriteMask(0x1 << 0);

  static const ColorWriteMask all = ColorWriteMask(0xf);
  static const ColorWriteMask unspecialized = ColorWriteMask(0xFFFFFFFF);

  // Bitwise OR operator to combine masks
  ColorWriteMask operator |(ColorWriteMask other) {
    return ColorWriteMask(rawValue | other.rawValue);
  }

  // Bitwise AND operator to check for a mask
  ColorWriteMask operator &(ColorWriteMask other) {
    return ColorWriteMask(rawValue & other.rawValue);
  }

  bool has(ColorWriteMask mask) => (rawValue & mask.rawValue) != 0;
}

enum BlendFactor {
  zero(0),
  one(1),
  sourceColor(2),
  oneMinusSourceColor(3),
  sourceAlpha(4),
  oneMinusSourceAlpha(5),
  destinationColor(6),
  oneMinusDestinationColor(7),
  destinationAlpha(8),
  oneMinusDestinationAlpha(9),
  sourceAlphaSaturated(10),
  blendColor(11),
  oneMinusBlendColor(12),
  blendAlpha(13),
  oneMinusBlendAlpha(14),
  source1Color(15),
  oneMinusSource1Color(16),
  source1Alpha(17),
  oneMinusSource1Alpha(18);

  final int value;
  const BlendFactor(this.value);
}

enum PrimitiveTopology {
  unspecified(0),
  point(1),
  line(2),
  triangle(3);

  final int value;
  const PrimitiveTopology(this.value);
}

enum LoadAction {
  dontCare(0),
  load(1),
  clear(2);

  final int value;
  const LoadAction(this.value);
}

enum StoreAction {
  dontCare(0),
  store(1),
  multisampleResolve(2),
  storeAndMultisampleResolve(3),
  unknown(4),
  customSampleDepthStore(5);

  final int value;
  const StoreAction(this.value);
}

class RenderPassDescriptor {
  List<RenderPassDescriptorColorAttachment> colorAttachments;

  RenderPassDescriptor({required this.colorAttachments});

  @pragma("vm:entry-point")
  Map<String, dynamic> toMap() {
    return {
      'colorAttachments': colorAttachments.map((e) => e.toMap()).toList(),
    };
  }
}

class RenderPassDescriptorColorAttachment {
  Texture? texture;
  LoadAction loadAction = LoadAction.clear;
  StoreAction storeAction = StoreAction.store;
  List<double> clearColor = const [0.0, 0.0, 0.0, 1.0];

  RenderPassDescriptorColorAttachment({
    this.texture,
    this.loadAction = LoadAction.clear,
    this.storeAction = StoreAction.store,
    this.clearColor = const [0.0, 0.0, 0.0, 1.0],
  });

  Map<String, dynamic> toMap() {
    return {
      'texture': texture,
      'loadAction': loadAction.value,
      'storeAction': storeAction.value,
      'clearColor': clearColor,
    };
  }
}
