#include <metal_stdlib>
using namespace metal;

/**
 * A structure for the vertex data provided by the app.
 * This must match the layout of the `Vertex` struct in Swift.
 */
struct Vertex {
    float4 position [[attribute(0)]];
    float4 color [[attribute(1)]];
};

/**
 * A structure to define the output of the vertex shader
 * and the input for the fragment shader.
 * `position` is the required clip-space position.
 * `color` is the interpolated vertex color.
 */
struct VertexOut {
    float4 position [[position]];
    float4 color;
};

/**
 * The input buffer indices, matching the argument table in Swift.
 * The `[[buffer(0)]]` attribute corresponds to `setAddress(..., index: 0)` in Swift.
 * The `[[buffer(1)]]` attribute corresponds to `setAddress(..., index: 1)` in Swift.
 */
enum BufferIndex {
    VertexBuffer = 0,
    ViewportBuffer = 1
};

/**
 * Vertex Shader
 */
vertex VertexOut vertexShader(
    uint vertexID [[vertex_id]],
    constant Vertex *vertices [[buffer(VertexBuffer)]],
    constant uint2 *viewportSize [[buffer(ViewportBuffer)]]
) {
    // 1. Get the specific vertex for this vertex ID.
    Vertex in = vertices[vertexID];

    VertexOut out;
    
    // 2. Transform vertex position from normalized coordinates ([-1, 1])
    //    to clip space by applying the viewport aspect ratio.
    float2 pixelPos = in.position.xy * (float2(*viewportSize) / 2.0);
    float2 ndcPos = pixelPos / (float2(*viewportSize) / 2.0);

    out.position = float4(ndcPos.x, ndcPos.y, 0.0, 1.0);
    out.color = in.color;
    
    return out;
}

/**
 * Fragment Shader
 *
 * This function is called for every pixel within the triangle.
 * It receives the interpolated color from the vertex shader (`in.color`)
 * and returns it as the final color for that pixel.
 */
fragment float4 fragmentShader(VertexOut in [[stage_in]]) {
    return in.color;
}