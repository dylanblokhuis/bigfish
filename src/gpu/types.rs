use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TriangleData {
    pub vertices: [Vertex; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ViewportSize {
    pub size: [u32; 2],
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RenderPipelineDescriptor {
    pub label: String,
    pub color_attachments: Vec<RenderPipelineDescriptorColorAttachment>,
    pub depth_attachment_pixel_format: PixelFormat,
    pub stencil_attachment_pixel_format: PixelFormat,
    pub primitive_topology: PrimitiveTopology,
    pub vertex_shader: ShaderLibrary,
    pub fragment_shader: ShaderLibrary,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComputePipelineDescriptor {
    pub label: String,
    pub compute_shader: ShaderLibrary,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PrimitiveTopology(pub usize);

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ShaderLibrary {
    pub path: String,
    pub entry_point: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RenderPipelineDescriptorColorAttachment {
    pub pixel_format: PixelFormat,
    pub write_mask: ColorWriteMask,
    pub blend_enabled: bool,
    pub rgb_blend_op: BlendOp,
    pub alpha_blend_op: BlendOp,
    pub source_alpha_blend_factor: BlendFactor,
    pub destination_alpha_blend_factor: BlendFactor,
    pub source_rgb_blend_factor: BlendFactor,
    pub destination_rgb_blend_factor: BlendFactor,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ColorWriteMask(pub usize);

#[derive(Deserialize, Serialize, Debug)]
pub struct BlendOp(pub usize);

#[derive(Deserialize, Serialize, Debug)]
pub struct BlendFactor(pub usize);

#[derive(Deserialize, Serialize, Debug)]
pub struct PixelFormat(pub usize);
