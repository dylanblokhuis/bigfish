use bigfish_macros::native_impl;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTL4ArgumentTableDescriptor, MTL4BlendState, MTL4CommandBuffer, MTL4CommandQueue,
    MTL4Compiler, MTL4CompilerDescriptor, MTL4ComputePipelineDescriptor, MTL4LibraryFunctionDescriptor, MTL4RenderPipelineDescriptor, MTLBlendFactor, MTLColorWriteMask, MTLCreateSystemDefaultDevice, MTLDevice, MTLEvent, MTLPixelFormat, MTLPrimitiveTopologyClass, MTLResidencySet, MTLResidencySetDescriptor, MTLSharedEvent, MTLTextureDescriptor, MTLTextureType, MTLTextureUsage
};
use std::process::{Command, Stdio};
use std::io::Write;
use std::str::FromStr;

use crate::dart_api::{from_dart, NativeArguments, Scope};
use crate::window::Window;

use super::{
    command_buffer::CommandBuffer,
    resources::{ArgumentTable, Buffer, ComputePipeline, RenderPipeline, Texture},
    types::{ComputePipelineDescriptor, RenderPipelineDescriptor},
};

type Id<T> = Retained<ProtocolObject<T>>;

pub struct Gpu {
    pub device: Id<dyn MTLDevice>,
    pub command_queue: Id<dyn MTL4CommandQueue>,
    pub command_buffer: Id<dyn MTL4CommandBuffer>,
    pub command_allocators: Vec<Id<dyn objc2_metal::MTL4CommandAllocator>>,
    pub compiler: Id<dyn MTL4Compiler>,
    pub residency_set: Id<dyn MTLResidencySet>,
    pub shared_event: Id<dyn MTLSharedEvent>,
    pub frame_number: u64,
    pub window_peer: *mut Window,
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
            layer.setMaximumDrawableCount(frames_in_flight);
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

        let vertex_spirv = {
            let child = Command::new("slangc")
                .arg("-stage")
                .arg("vertex")
                .arg("-target")
                .arg("spirv")
                .arg("-entry")
                .arg(descriptor.vertex_shader.entry_point.as_str())
                .arg(descriptor.vertex_shader.path.as_str())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let output = child.wait_with_output().unwrap();
            output.stdout
        };

        let fragment_spirv = {
            let child = Command::new("slangc")
                .arg("-stage")
                .arg("fragment")
                .arg("-target")
                .arg("spirv")
                .arg("-entry")
                .arg(descriptor.fragment_shader.entry_point.as_str())
                .arg(descriptor.fragment_shader.path.as_str())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let output = child.wait_with_output().unwrap();
            output.stdout
        };

        let vertex_metal = {
            let mut child = Command::new("spirv-cross")
                .arg("-")
                .arg("--msl")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&vertex_spirv).unwrap();
                stdin.flush().unwrap();
            }

            let output = child.wait_with_output().unwrap();
            String::from_utf8(output.stdout).unwrap()
        };

        let fragment_metal = {
            let mut child = Command::new("spirv-cross")
                .arg("-")
                .arg("--msl")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&fragment_spirv).unwrap();
                stdin.flush().unwrap();
            }

            let output = child.wait_with_output().unwrap();
            String::from_utf8(output.stdout).unwrap()
        };

        std::fs::write("target/shaders/vertex.metal", &vertex_metal).unwrap();
        std::fs::write("target/shaders/fragment.metal", &fragment_metal).unwrap();

        let vertex_library = gpu
            .device
            .newLibraryWithSource_options_error(
                &objc2_foundation::NSString::from_str(&vertex_metal),
                None,
            )
            .unwrap();
        let fragment_library = gpu
            .device
            .newLibraryWithSource_options_error(
                &objc2_foundation::NSString::from_str(&fragment_metal),
                None,
            )
            .unwrap();
        let vfd = MTL4LibraryFunctionDescriptor::new();
        vfd.setLibrary(Some(&vertex_library));
        vfd.setName(Some(&objc2_foundation::NSString::from_str("main0")));
        rp_desc.setVertexFunctionDescriptor(Some(&*vfd));

        let ffd = MTL4LibraryFunctionDescriptor::new();
        ffd.setLibrary(Some(&fragment_library));
        ffd.setName(Some(&objc2_foundation::NSString::from_str("main0")));
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

    fn compile_compute_pipeline(args: NativeArguments, scope: Scope<'_>) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let descriptor_instance = args.get_arg(1).unwrap();
        let descriptor = descriptor_instance
            .invoke(scope.new_string("toMap").unwrap(), &mut [])
            .unwrap();
        let descriptor = from_dart::<ComputePipelineDescriptor>(descriptor).unwrap();
        let compute_shader = descriptor.compute_shader;
        let compute_shader_spirv = {
            let child = Command::new("slangc")
                .arg("-stage")
                .arg("compute")
                .arg("-target")
                .arg("spirv")
                .arg("-entry")
                .arg(compute_shader.entry_point.as_str())
                .arg(compute_shader.path.as_str())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            let output = child.wait_with_output().unwrap();
            output.stdout
        };
        let compute_shader_metal = {
            let mut child = Command::new("spirv-cross")
                .arg("-")
                .arg("--msl")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&compute_shader_spirv).unwrap();
                stdin.flush().unwrap();
            }
            let output = child.wait_with_output().unwrap();
            String::from_utf8(output.stdout).unwrap()
        };
        std::fs::write("target/shaders/compute.metal", &compute_shader_metal).unwrap();
        let compute_shader_library = gpu
            .device
            .newLibraryWithSource_options_error(
                &objc2_foundation::NSString::from_str(&compute_shader_metal),
                None,
            )
            .unwrap();

        let cfd = MTL4LibraryFunctionDescriptor::new();
        cfd.setLibrary(Some(&compute_shader_library));
        cfd.setName(Some(&objc2_foundation::NSString::from_str("main0")));

        let desc = MTL4ComputePipelineDescriptor::new();
        desc.setComputeFunctionDescriptor(Some(&*cfd));

        let compute_pipeline_state = gpu
            .compiler
            .newComputePipelineStateWithDescriptor_compilerTaskOptions_error(&desc, None)
            .unwrap();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "ComputePipeline").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(ComputePipeline {
            compute_pipeline_state,
        }));
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

    fn add_texture_to_residency_set(args: NativeArguments) {
        let gpu_instance = args.get_arg(0).unwrap();
        let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
        let texture_instance = args.get_arg(1).unwrap();
        let texture = texture_instance.get_peer::<Texture>().unwrap();

        gpu.residency_set.addAllocation(texture.texture.as_ref());
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

        let texture = gpu.device.newTextureWithDescriptor(&descriptor).unwrap();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "Texture").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(Texture { texture }));
        args.set_return_value(class_instance);
    }

    fn metal_api_count(args: NativeArguments) {
        args.set_int_return_value(47);
    }
}
