use bigfish_macros::native_impl;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTL4ArgumentTable, MTLBuffer, MTLComputePipelineState, MTLRenderPipelineState, MTLTexture,
};

use crate::dart_api::{Handle, NativeArguments, Result, Scope, TypedDataView};

type Id<T> = Retained<ProtocolObject<T>>;

pub struct Texture {
    pub texture: Id<dyn MTLTexture>,
}

pub struct Buffer {
    pub buffer: Id<dyn MTLBuffer>,
}

pub struct ArgumentTable {
    pub table: Id<dyn MTL4ArgumentTable>,
}

pub struct RenderPipeline {
    pub render_pipeline_state: Id<dyn MTLRenderPipelineState>,
}

pub struct ComputePipeline {
    pub compute_pipeline_state: Id<dyn MTLComputePipelineState>,
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
                    let list_obj = crate::dart_api::List::new(list).unwrap();
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

    fn set_label(_args: NativeArguments, _scope: Scope<'_>) {
        // Label methods may not be available on all MTLBuffer implementations
        // No-op for now - can be implemented if needed
    }
}
