#include <metal_stdlib>
using namespace metal;

struct Vertex {
    float4 position;
    float4 color;
};

struct VertexOut {
    float4 position [[position]];
    float4 color;
};

struct Args {
    device Vertex* vertices [[id(0)]];
    device uint2* viewportSize [[id(1)]];
} [[argument_table]];

vertex VertexOut vertexShader(
    uint vertexID [[vertex_id]],
    Args args [[argument_table]]
) {
    Vertex in = args.vertices[vertexID];

    VertexOut out;
    
    // Positions are already in NDC-ish space from Dart.
    out.position = float4(in.position.xy, 0.0, 1.0);
    out.color = in.color;
    
    return out;
}

fragment float4 fragmentShader(VertexOut in [[stage_in]]) {
    return in.color;
}