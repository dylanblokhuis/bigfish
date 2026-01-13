use bigfish_macros::native_impl;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTL4ArgumentTable, MTL4ArgumentTableDescriptor, MTL4BlendState, MTL4CommandAllocator,
    MTL4CommandBuffer, MTL4CommandQueue, MTL4Compiler, MTL4CompilerDescriptor,
    MTL4RenderCommandEncoder, MTL4RenderPassDescriptor, MTL4RenderPipelineDescriptor,
    MTLBlendFactor, MTLColorWriteMask, MTLCreateSystemDefaultDevice, MTLDevice, MTLEvent,
    MTLLoadAction, MTLPixelFormat, MTLPrimitiveTopologyClass, MTLPrimitiveType,
    MTLRenderPipelineState, MTLRenderStages, MTLResidencySet, MTLResidencySetDescriptor,
    MTLSharedEvent, MTLStoreAction, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage, MTLViewport,
};
// Bring ObjC protocol traits into scope for method resolution.
use objc2_metal::{
    MTL4ArgumentTable as _, MTL4CommandEncoder as _, MTL4Compiler as _,
    MTL4RenderCommandEncoder as _, MTLBuffer as _, MTLDrawable as _, MTLTexture as _,
};
use objc2_quartz_core::CAMetalDrawable;
use serde::{Deserialize, Serialize};

use crate::dart_api::{from_dart, Handle, List, NativeArguments, Result, Scope, TypedDataView};
use crate::window::Window;

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TriangleData {
    vertices: [Vertex; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ViewportSize {
    size: [u32; 2],
}

type Id<T> = Retained<ProtocolObject<T>>;

struct Gpu {
    device: Id<dyn MTLDevice>,
    command_queue: Id<dyn MTL4CommandQueue>,
    command_buffer: Id<dyn MTL4CommandBuffer>,
    command_allocators: Vec<Id<dyn MTL4CommandAllocator>>,
    compiler: Id<dyn MTL4Compiler>,
    residency_set: Id<dyn MTLResidencySet>,
    shared_event: Id<dyn MTLSharedEvent>,
    frame_number: u64,
    window_peer: *mut Window,
}

struct CommandBuffer {
    drawable: Id<dyn CAMetalDrawable>,
}

struct Texture {
    texture: Id<dyn MTLTexture>,
}

#[native_impl]
impl Texture {
    fn replace_region(args: NativeArguments) {
        let texture_instance = args.get_arg(0).unwrap();
        let texture = texture_instance.get_peer::<Texture>().unwrap();

        let region_x = args.get_integer_arg(1).unwrap() as usize;
        let region_y = args.get_integer_arg(2).unwrap() as usize;
        let region_z = args.get_integer_arg(3).unwrap() as usize;
        let region_width = args.get_integer_arg(4).unwrap() as usize;
        let region_height = args.get_integer_arg(5).unwrap() as usize;
        let region_depth = args.get_integer_arg(6).unwrap() as usize;
        let mipmap_level = args.get_integer_arg(7).unwrap() as usize;

        let data_handle = args.get_arg(8).unwrap();
        let bytes_per_row = args.get_integer_arg(9).unwrap() as usize;
        let bytes_per_image = args.get_integer_arg(10).unwrap_or(0) as usize;

        let view = TypedDataView::acquire(data_handle).unwrap();
        let region = objc2_metal::MTLRegion {
            origin: objc2_metal::MTLOrigin {
                x: region_x,
                y: region_y,
                z: region_z,
            },
            size: objc2_metal::MTLSize {
                width: region_width,
                height: region_height,
                depth: region_depth,
            },
        };

        unsafe {
            use core::ptr::NonNull;
            let bytes_ptr = NonNull::new(view.data).unwrap().cast::<core::ffi::c_void>();
            texture
                .texture
                .replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage(
                    region,
                    mipmap_level,
                    0,
                    bytes_ptr,
                    bytes_per_row,
                    bytes_per_image,
                );
        }
        drop(view);
    }
}

struct ArgumentTable {
    table: Id<dyn MTL4ArgumentTable>,
}

#[native_impl]
impl ArgumentTable {
    fn set_buffer(args: NativeArguments) {
        let argument_table_instance = args.get_arg(0).unwrap();
        let argument_table = argument_table_instance.get_peer::<ArgumentTable>().unwrap();

        let buffer_instance = args.get_arg(1).unwrap();
        let buffer = buffer_instance.get_peer::<Buffer>().unwrap();

        let index = args.get_integer_arg(2).unwrap() as usize;
        let offset = args.get_integer_arg(3).unwrap_or(0) as usize;

        let buffer_address = buffer.buffer.gpuAddress() + offset as u64;
        unsafe {
            argument_table
                .table
                .setAddress_atIndex(buffer_address, index);
        }
    }

    fn set_texture(args: NativeArguments) {
        let argument_table_instance = args.get_arg(0).unwrap();
        let argument_table = argument_table_instance.get_peer::<ArgumentTable>().unwrap();

        let texture_instance = args.get_arg(1).unwrap();
        let texture = texture_instance.get_peer::<Texture>().unwrap();

        let index = args.get_integer_arg(2).unwrap() as usize;

        unsafe {
            let resource_id = texture.texture.gpuResourceID();
            argument_table.table.setTexture_atIndex(resource_id, index);
        }
    }
}

#[native_impl]
impl Texture {
    fn width(args: NativeArguments) {
        let texture_instance = args.get_arg(0).unwrap();
        let texture = texture_instance.get_peer::<Texture>().unwrap();
        let width = texture.texture.width() as i64;
        args.set_int_return_value(width);
    }

    fn height(args: NativeArguments) {
        let texture_instance = args.get_arg(0).unwrap();
        let texture = texture_instance.get_peer::<Texture>().unwrap();
        let height = texture.texture.height() as i64;
        args.set_int_return_value(height);
    }

    fn pixel_format(args: NativeArguments) {
        let texture_instance = args.get_arg(0).unwrap();
        let texture = texture_instance.get_peer::<Texture>().unwrap();
        let pixel_format = texture.texture.pixelFormat().0 as i64;
        args.set_int_return_value(pixel_format);
    }
}

#[native_impl]
impl CommandBuffer {
    fn render_command_encoder(args: NativeArguments, scope: Scope<'_>) {
        let command_buffer_instance = args.get_arg(0).unwrap();
        let command_buffer = command_buffer_instance.get_peer::<CommandBuffer>().unwrap();
        let gpu_handle = command_buffer_instance
            .get_field(scope.new_string("gpu").unwrap())
            .unwrap();
        let gpu = gpu_handle.get_peer::<Gpu>().unwrap();
        let descriptor_instance = args.get_arg(1).unwrap();
        let descriptor_map = descriptor_instance
            .invoke(scope.new_string("toMap").unwrap(), &mut [])
            .unwrap();

        let pass = MTL4RenderPassDescriptor::new();

        // TODO: clean up this mess
        let color_attachments_key = scope.new_string("colorAttachments").unwrap();
        if let Ok(color_attachments_list) = descriptor_map.map_get(&scope, color_attachments_key) {
            let list_obj = List::new(color_attachments_list).unwrap();
            if let Ok(len) = list_obj.len() {
                for i in 0..(len as usize) {
                    if let Ok(ca_map) = list_obj.get(&scope, i as isize) {
                        let ca = unsafe { pass.colorAttachments().objectAtIndexedSubscript(i) };

                        // Extract texture (optional - falls back to drawable if null)
                        let drawable_texture = command_buffer.drawable.texture();
                        let texture_key = scope.new_string("texture").unwrap();
                        let texture =
                            if let Ok(texture_handle) = ca_map.map_get(&scope, texture_key) {
                                if !texture_handle.is_null() {
                                    if let Ok(texture_peer) = texture_handle.get_peer::<Texture>() {
                                        Some(texture_peer.texture.as_ref())
                                    } else {
                                        // Invalid texture object, use drawable
                                        Some(drawable_texture.as_ref())
                                    }
                                } else {
                                    // Null texture, use drawable
                                    Some(drawable_texture.as_ref())
                                }
                            } else {
                                // No texture field, use drawable
                                Some(drawable_texture.as_ref())
                            };
                        ca.setTexture(texture);

                        // Extract load action
                        let load_action_key = scope.new_string("loadAction").unwrap();
                        if let Ok(load_action_handle) = ca_map.map_get(&scope, load_action_key) {
                            if let Ok(load_action_val) = load_action_handle.to_i64() {
                                ca.setLoadAction(MTLLoadAction(load_action_val as usize));
                            }
                        }

                        // Extract store action
                        let store_action_key = scope.new_string("storeAction").unwrap();
                        if let Ok(store_action_handle) = ca_map.map_get(&scope, store_action_key) {
                            if let Ok(store_action_val) = store_action_handle.to_i64() {
                                ca.setStoreAction(MTLStoreAction(store_action_val as usize));
                            }
                        }

                        // Extract clear color (optional)
                        let clear_color_key = scope.new_string("clearColor").unwrap();
                        if let Ok(clear_color_list) = ca_map.map_get(&scope, clear_color_key) {
                            let clear_color_list_obj = List::new(clear_color_list).unwrap();
                            if let Ok(clear_color_len) = clear_color_list_obj.len() {
                                if clear_color_len >= 4 {
                                    if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                                        clear_color_list_obj
                                            .get(&scope, 0)
                                            .and_then(|h| h.to_f64()),
                                        clear_color_list_obj
                                            .get(&scope, 1)
                                            .and_then(|h| h.to_f64()),
                                        clear_color_list_obj
                                            .get(&scope, 2)
                                            .and_then(|h| h.to_f64()),
                                        clear_color_list_obj
                                            .get(&scope, 3)
                                            .and_then(|h| h.to_f64()),
                                    ) {
                                        ca.setClearColor(objc2_metal::MTLClearColor {
                                            red: r,
                                            green: g,
                                            blue: b,
                                            alpha: a,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let render_command_encoder = gpu
            .command_buffer
            .renderCommandEncoderWithDescriptor(&pass)
            .unwrap();
        let render_command_encoder_instance = scope
            .new_object(
                scope
                    .get_class(
                        scope.library("package:app/native.dart").unwrap(),
                        "RenderCommandEncoder",
                    )
                    .unwrap(),
                scope.null_handle().unwrap(),
                &mut [],
            )
            .unwrap();
        render_command_encoder_instance
            .set_peer(Box::new(RenderCommandEncoder(render_command_encoder)));
        args.set_return_value(render_command_encoder_instance);
    }

    fn drawable(args: NativeArguments, scope: Scope<'_>) {
        let command_buffer_instance = args.get_arg(0).unwrap();
        let command_buffer = command_buffer_instance.get_peer::<CommandBuffer>().unwrap();
        let texture = command_buffer.drawable.texture();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "Texture").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(Texture { texture }));
        args.set_return_value(class_instance);
    }
}

struct RenderCommandEncoder(Id<dyn MTL4RenderCommandEncoder>);

#[native_impl]
impl RenderCommandEncoder {
    fn set_render_pipeline(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let render_pipeline = args.get_arg(1).unwrap();
        let render_pipeline = render_pipeline.get_peer::<RenderPipeline>().unwrap();

        render_command_encoder
            .0
            .setRenderPipelineState(&render_pipeline.render_pipeline_state);
    }

    fn set_viewport(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let x = args.get_double_arg(1).unwrap();
        let y = args.get_double_arg(2).unwrap();
        let width = args.get_double_arg(3).unwrap();
        let height = args.get_double_arg(4).unwrap();

        render_command_encoder.0.setViewport(MTLViewport {
            originX: x,
            originY: y,
            width: width,
            height: height,
            znear: 0.0,
            zfar: 1.0,
        });
    }

    fn set_scissor_rect(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let x = args.get_integer_arg(1).unwrap() as usize;
        let y = args.get_integer_arg(2).unwrap() as usize;
        let width = args.get_integer_arg(3).unwrap() as usize;
        let height = args.get_integer_arg(4).unwrap() as usize;

        render_command_encoder
            .0
            .setScissorRect(objc2_metal::MTLScissorRect {
                x,
                y,
                width,
                height,
            });
    }

    fn set_cull_mode(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let mode = args.get_integer_arg(1).unwrap() as usize;
        render_command_encoder
            .0
            .setCullMode(objc2_metal::MTLCullMode(mode));
    }

    fn draw_primitives(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();
        let primitive_type = args.get_integer_arg(1).unwrap();
        let vertex_count = args.get_integer_arg(2).unwrap();
        let instance_count = args.get_integer_arg(3).unwrap();
        let vertex_start = args.get_integer_arg(4).unwrap();
        let base_instance = args.get_integer_arg(5).unwrap();

        unsafe {
            render_command_encoder
                .0
                .drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance(
                    MTLPrimitiveType(primitive_type as usize),
                    vertex_start as usize,
                    vertex_count as usize,
                    instance_count as usize,
                    base_instance as usize,
                );
        }
    }

    // fn draw_indexed_primitives(args: NativeArguments) {
    //     let render_command_encoder_instance = args.get_arg(0).unwrap();
    //     let render_command_encoder = render_command_encoder_instance
    //         .get_peer::<RenderCommandEncoder>()
    //         .unwrap();
    //     let primitive_type = args.get_integer_arg(1).unwrap();
    //     let index_count = args.get_integer_arg(2).unwrap();
    //     let instance_count = args.get_integer_arg(3).unwrap();
    //     let base_vertex = args.get_integer_arg(4).unwrap();
    //     let base_instance = args.get_integer_arg(5).unwrap();

    //     unsafe {
    //         render_command_encoder
    //             .0
    //             .drawIndexedPrimitives_indexCount_indexType_indexBuffer_indexBufferLength_instanceCount_baseVertex_baseInstance(
    //                 MTLPrimitiveType(primitive_type as usize),
    //                 index_count as usize,
    //                 instance_count as usize,
    //                 base_vertex as usize,
    //                 base_instance as usize,
    //             );
    //     }
    // }

    fn set_argument_table_object(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let argument_table_instance = args.get_arg(1).unwrap();
        let argument_table = argument_table_instance.get_peer::<ArgumentTable>().unwrap();

        unsafe {
            render_command_encoder.0.setArgumentTable_atStages(
                argument_table.table.as_ref(),
                MTLRenderStages::Vertex | MTLRenderStages::Fragment,
            );
        }
    }

    fn end_encoding(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();
        render_command_encoder.0.endEncoding();
    }
}

#[native_impl]
impl Gpu {
    fn init(args: NativeArguments) {
        let instance = args.get_arg(0).unwrap();
        let window_handle = args.get_arg(1).unwrap();
        let window_peer = window_handle.get_peer::<Window>().unwrap() as *mut Window;

        let frames_in_flight = 3;
        let device = MTLCreateSystemDefaultDevice().unwrap();
        let command_queue = device.newMTL4CommandQueue().unwrap();
        let command_buffer = device.newCommandBuffer().unwrap();
        let mut command_allocators = Vec::with_capacity(frames_in_flight);
        for _ in 0..frames_in_flight {
            let command_allocator = device.newCommandAllocator().unwrap();
            command_allocators.push(command_allocator);
        }
        let desc = MTLResidencySetDescriptor::new();
        let residency_set = device.newResidencySetWithDescriptor_error(&desc).unwrap();

        // Bind the SDL-created CAMetalLayer to this device and configure basics.
        #[cfg(target_os = "macos")]
        {
            use objc2_quartz_core::CAMetalLayer;
            let window = unsafe { &*window_peer };
            let layer: &CAMetalLayer = window.metal_layer();
            layer.setDevice(Some(device.as_ref()));
            layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
            layer.setMaximumDrawableCount(frames_in_flight as usize);
        }

        command_queue.addResidencySet(&residency_set);

        #[cfg(target_os = "macos")]
        {
            let window = unsafe { &*window_peer };
            command_queue.addResidencySet(&window.metal_layer().residencySet());
        }

        let shared_event = device.newSharedEvent().unwrap();
        shared_event.setSignaledValue(0);

        let compiler_desc = MTL4CompilerDescriptor::new();
        let compiler = device
            .newCompilerWithDescriptor_error(&compiler_desc)
            .unwrap();

        instance.set_peer(Box::new(Gpu {
            device,
            command_queue,
            command_buffer,
            command_allocators,
            residency_set,
            compiler,
            shared_event,
            frame_number: 0,
            window_peer,
        }));
    }

    fn create_argument_table(args: NativeArguments, scope: Scope<'_>) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();

        let max_buffer_bind_count = args.get_integer_arg(1).unwrap() as usize;
        let max_texture_bind_count = args.get_integer_arg(2).unwrap() as usize;
        let max_sampler_state_bind_count = args.get_integer_arg(3).unwrap() as usize;
        let table_desc = MTL4ArgumentTableDescriptor::new();
        if max_buffer_bind_count > 0 {
            table_desc.setMaxBufferBindCount(max_buffer_bind_count);
        }
        if max_texture_bind_count > 0 {
            table_desc.setMaxTextureBindCount(max_texture_bind_count);
        }
        if max_sampler_state_bind_count > 0 {
            table_desc.setMaxSamplerStateBindCount(max_sampler_state_bind_count);
        }
        let table = gpu
            .device
            .newArgumentTableWithDescriptor_error(&table_desc)
            .unwrap();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "ArgumentTable").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(ArgumentTable { table }));
        args.set_return_value(class_instance);
    }

    fn render_pipeline_descriptor(args: NativeArguments) {}

    fn begin_command_buffer(args: NativeArguments, scope: Scope<'_>) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu: &mut Gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let window = unsafe { &*gpu.window_peer };

        let drawable = match window.metal_layer().nextDrawable() {
            Some(d) => d,
            None => return,
        };

        gpu.frame_number += 1;
        let frame_index = (gpu.frame_number as usize) % gpu.command_allocators.len();

        if gpu.frame_number > gpu.command_allocators.len() as u64 {
            let earlier = gpu.frame_number - gpu.command_allocators.len() as u64;
            let _timed_out = gpu
                .shared_event
                .waitUntilSignaledValue_timeoutMS(earlier, 10);
        }

        let allocator = &gpu.command_allocators[frame_index];
        allocator.reset();

        gpu.command_buffer
            .beginCommandBufferWithAllocator(allocator);
        gpu.command_buffer.useResidencySet(&gpu.residency_set);

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "CommandBuffer").unwrap();
        // let constructor_name = scope.new_string("CommandBuffer").unwrap();
        let class_instance = scope
            .new_object(
                class_type,
                scope.null_handle().unwrap(),
                &mut [gpu_instance.raw()],
            )
            .unwrap();
        class_instance.set_peer(Box::new(CommandBuffer { drawable }));
        // class_instance.set_field(scope.new_string("gpu").unwrap(), &gpu_instance);
        args.set_return_value(class_instance);
    }

    fn end_command_buffer(args: NativeArguments) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let command_buffer_instance = args.get_arg(1).unwrap();
        let command_buffer = command_buffer_instance.get_peer::<CommandBuffer>().unwrap();

        gpu.command_buffer.endCommandBuffer();

        // Submit + present (Metal 4 queue semantics).
        let drawable_mtl: &ProtocolObject<dyn objc2_metal::MTLDrawable> =
            command_buffer.drawable.as_ref();
        gpu.command_queue.waitForDrawable(drawable_mtl);

        let buf_ptr = core::ptr::NonNull::from(&*gpu.command_buffer);
        let mut bufs = [buf_ptr];
        unsafe {
            gpu.command_queue
                .commit_count(core::ptr::NonNull::new(bufs.as_mut_ptr()).unwrap(), 1);
        }

        gpu.command_queue.signalDrawable(drawable_mtl);
        command_buffer.drawable.present();

        let event: &ProtocolObject<dyn MTLEvent> = gpu.shared_event.as_ref();
        gpu.command_queue.signalEvent_value(event, gpu.frame_number);
    }

    fn compile_render_pipeline(args: NativeArguments, scope: Scope<'_>) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let descriptor_instance = args.get_arg(1).unwrap();
        let descriptor = descriptor_instance
            .invoke(scope.new_string("toMap").unwrap(), &mut [])
            .unwrap();
        let descriptor = from_dart::<RenderPipelineDescriptor>(descriptor).unwrap();
        let rp_desc = MTL4RenderPipelineDescriptor::new();
        for i in 0..descriptor.color_attachments.len() {
            let color_attachment = &descriptor.color_attachments[i];
            let ca = unsafe { rp_desc.colorAttachments().objectAtIndexedSubscript(i) };
            ca.setPixelFormat(MTLPixelFormat(color_attachment.pixel_format.0));
            ca.setWriteMask(MTLColorWriteMask(color_attachment.write_mask.0));
            ca.setBlendingState(if color_attachment.blend_enabled {
                MTL4BlendState::Enabled
            } else {
                MTL4BlendState::Disabled
            });
            ca.setSourceRGBBlendFactor(MTLBlendFactor(color_attachment.source_rgb_blend_factor.0));
            ca.setDestinationRGBBlendFactor(MTLBlendFactor(
                color_attachment.destination_rgb_blend_factor.0,
            ));
            ca.setSourceAlphaBlendFactor(MTLBlendFactor(
                color_attachment.source_alpha_blend_factor.0,
            ));
            ca.setDestinationAlphaBlendFactor(MTLBlendFactor(
                color_attachment.destination_alpha_blend_factor.0,
            ));
        }

        rp_desc
            .setInputPrimitiveTopology(MTLPrimitiveTopologyClass(descriptor.primitive_topology.0));

        let vertex_library = gpu
            .device
            .newLibraryWithSource_options_error(
                &objc2_foundation::NSString::from_str(&descriptor.vertex_shader.source),
                None,
            )
            .unwrap();
        let fragment_library = gpu
            .device
            .newLibraryWithSource_options_error(
                &objc2_foundation::NSString::from_str(&descriptor.fragment_shader.source),
                None,
            )
            .unwrap();
        let vfd = objc2_metal::MTL4LibraryFunctionDescriptor::new();
        vfd.setLibrary(Some(&vertex_library));
        vfd.setName(Some(&objc2_foundation::NSString::from_str(
            &descriptor.vertex_shader.entry_point,
        )));
        rp_desc.setVertexFunctionDescriptor(Some(&*vfd));

        let ffd = objc2_metal::MTL4LibraryFunctionDescriptor::new();
        ffd.setLibrary(Some(&fragment_library));
        ffd.setName(Some(&objc2_foundation::NSString::from_str(
            &descriptor.fragment_shader.entry_point,
        )));
        rp_desc.setFragmentFunctionDescriptor(Some(&*ffd));

        let render_pipeline_state = gpu
            .compiler
            .newRenderPipelineStateWithDescriptor_compilerTaskOptions_error(&rp_desc, None)
            .unwrap();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "RenderPipeline").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(RenderPipeline {
            render_pipeline_state,
        }));
        class_instance.set_field(scope.new_string("gpu").unwrap(), &gpu_instance);
        args.set_return_value(class_instance);
    }

    fn create_buffer(args: NativeArguments, scope: Scope<'_>) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let length = args.get_integer_arg(1).unwrap() as usize;
        // For now, always use StorageModeShared
        // TODO: Support other storage modes if needed
        let options = objc2_metal::MTLResourceOptions::StorageModeShared;
        let buffer = gpu
            .device
            .newBufferWithLength_options(length, options)
            .unwrap();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "Buffer").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(Buffer { buffer }));
        args.set_return_value(class_instance);
    }

    fn add_buffer_to_residency_set(args: NativeArguments) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let buffer_instance = args.get_arg(1).unwrap();
        let buffer = buffer_instance.get_peer::<Buffer>().unwrap();

        gpu.residency_set.addAllocation(buffer.buffer.as_ref());
    }

    fn commit_residency_set(args: NativeArguments) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();

        gpu.residency_set.commit();
    }

    fn create_texture(args: NativeArguments, scope: Scope<'_>) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let width = args.get_integer_arg(1).unwrap() as usize;
        let height = args.get_integer_arg(2).unwrap() as usize;
        let pixel_format_value = args.get_integer_arg(3).unwrap() as usize;
        let pixel_format = MTLPixelFormat(pixel_format_value);

        let descriptor = MTLTextureDescriptor::new();
        unsafe {
            descriptor.setTextureType(MTLTextureType::Type2D);
            descriptor.setWidth(width);
            descriptor.setHeight(height);
            descriptor.setPixelFormat(pixel_format);
            descriptor.setStorageMode(objc2_metal::MTLStorageMode::Managed);
            descriptor.setUsage(
                MTLTextureUsage::ShaderRead
                    | MTLTextureUsage::ShaderWrite
                    | MTLTextureUsage::RenderTarget,
            );
            descriptor.setMipmapLevelCount(1);
        }

        let texture = unsafe { gpu.device.newTextureWithDescriptor(&descriptor) }.unwrap();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "Texture").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(Texture { texture }));
        args.set_return_value(class_instance);
    }
}

struct RenderPipeline {
    render_pipeline_state: Id<dyn MTLRenderPipelineState>,
}

struct Buffer {
    buffer: Id<dyn objc2_metal::MTLBuffer>,
}

#[native_impl]
impl Buffer {
    fn length(args: NativeArguments) {
        let buffer_instance = args.get_arg(0).unwrap();
        let buffer = buffer_instance.get_peer::<Buffer>().unwrap();
        let length = buffer.buffer.length() as i64;
        args.set_int_return_value(length);
    }

    fn gpu_address(args: NativeArguments) {
        let buffer_instance = args.get_arg(0).unwrap();
        let buffer = buffer_instance.get_peer::<Buffer>().unwrap();
        let addr = buffer.buffer.gpuAddress() as i64;
        args.set_int_return_value(addr);
    }

    fn contents(args: NativeArguments, scope: Scope<'_>) {
        let buffer_instance = args.get_arg(0).unwrap();
        let buffer = buffer_instance.get_peer::<Buffer>().unwrap();
        let length = buffer.buffer.length();
        let contents_ptr = buffer.buffer.contents().as_ptr() as *const u8;

        // Try to create Uint8List via dart:typed_data
        // If that fails, fall back to creating a regular List
        let uint8_list = (|| -> Result<Handle> {
            let library = scope.library("dart:typed_data")?;
            let uint8_list_class = scope.get_class(library, "Uint8List")?;
            let length_handle = scope.new_integer(length as i64)?;
            let list_instance = scope.new_object(
                uint8_list_class,
                scope.null_handle()?,
                &mut [length_handle.raw()],
            )?;

            // Copy data into the list using TypedDataView
            let view = TypedDataView::acquire(list_instance)?;
            unsafe {
                let slice = core::slice::from_raw_parts(contents_ptr, length);
                core::ptr::copy_nonoverlapping(slice.as_ptr(), view.data, length);
            }
            Ok(list_instance)
        })();

        match uint8_list {
            Ok(list) => args.set_return_value(list),
            Err(_) => {
                // Fallback: create a regular List<int>
                let list_handle = unsafe { crate::dart_api::sys::Dart_NewList(length as isize) };
                let list = Handle::from_raw(list_handle);
                if !list.is_null() {
                    let list_obj = List::new(list).unwrap();
                    unsafe {
                        let slice = core::slice::from_raw_parts(contents_ptr, length);
                        for (i, &byte) in slice.iter().enumerate() {
                            let byte_handle = scope.new_integer(byte as i64).unwrap();
                            list_obj.set(i as isize, byte_handle).unwrap();
                        }
                    }
                    args.set_return_value(list);
                } else {
                    args.set_return_value(scope.null_handle().unwrap());
                }
            }
        }
    }

    fn set_contents(args: NativeArguments) {
        let buffer_instance = args.get_arg(0).unwrap();
        let buffer = buffer_instance.get_peer::<Buffer>().unwrap();
        let data_handle = args.get_arg(1).unwrap();

        let length = buffer.buffer.length();
        let contents_ptr = buffer.buffer.contents().as_ptr() as *mut u8;

        // Use TypedDataView for efficient access
        let view = TypedDataView::acquire(data_handle).unwrap();
        let copy_length = core::cmp::min(length, view.len as usize);
        unsafe {
            core::ptr::copy_nonoverlapping(view.data, contents_ptr, copy_length);
        }
        drop(view);
    }

    fn label(args: NativeArguments, scope: Scope<'_>) {
        // Label methods may not be available on all MTLBuffer implementations
        // Return null for now - can be implemented if needed
        args.set_return_value(scope.null_handle().unwrap());
    }

    fn set_label(args: NativeArguments, scope: Scope<'_>) {
        // Label methods may not be available on all MTLBuffer implementations
        // No-op for now - can be implemented if needed
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RenderPipelineDescriptor {
    label: String,
    color_attachments: Vec<RenderPipelineDescriptorColorAttachment>,
    depth_attachment_pixel_format: PixelFormat,
    stencil_attachment_pixel_format: PixelFormat,
    primitive_topology: PrimitiveTopology,
    vertex_shader: ShaderLibrary,
    fragment_shader: ShaderLibrary,
}

#[derive(Deserialize, Serialize, Debug)]
struct PrimitiveTopology(usize);

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ShaderLibrary {
    source: String,
    entry_point: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RenderPipelineDescriptorColorAttachment {
    pixel_format: PixelFormat,
    write_mask: ColorWriteMask,
    blend_enabled: bool,
    rgb_blend_op: BlendOp,
    alpha_blend_op: BlendOp,
    source_alpha_blend_factor: BlendFactor,
    destination_alpha_blend_factor: BlendFactor,
    source_rgb_blend_factor: BlendFactor,
    destination_rgb_blend_factor: BlendFactor,
}

#[derive(Deserialize, Serialize, Debug)]
struct ColorWriteMask(usize);

#[derive(Deserialize, Serialize, Debug)]
struct BlendOp(usize);

#[derive(Deserialize, Serialize, Debug)]
struct BlendFactor(usize);

#[derive(Deserialize, Serialize, Debug)]
struct PixelFormat(usize);
