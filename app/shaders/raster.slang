// vertex_color.slang

struct Vertex {
    float4 position;
    float4 color;
};

struct VertexOut {
    float4 position : SV_Position;
    float4 color    : COLOR;
};

[[vk::binding(0)]]
StructuredBuffer<Vertex> vertices;

[vk::binding(1)]
Texture2D<float4> colorTexture;

[shader("vertex")]
VertexOut vertexShader(uint vertexID : SV_VertexID) {
    Vertex v = vertices[vertexID];

    VertexOut out;
    // Mirroring your logic: mapping xy to NDC, z to 0, w to 1
    out.position = float4(v.position.xy, 0.0, 1.0);
    out.color = v.color;
    
    return out;
}

[shader("fragment")]
float4 fragmentShader(VertexOut input) : SV_Target {
    return input.color;
}