// Optimized Fullscreen Triangle Blit

struct VertexOut {
    float4 position : SV_Position;
    float2 uv : UV;
};

Texture2D<float4> sourceTexture;
SamplerState linearSampler;

[shader("vertex")]
VertexOut vertexShader(uint vertexID : SV_VertexID) {
    VertexOut out;
    
    // Efficiently generates:
    // ID 0: (-1, -1)
    // ID 1: ( 3, -1)
    // ID 2: (-1,  3)
    float2 uv = float2((vertexID << 1) & 2, vertexID & 2);
    out.position = float4(uv * float2(2.0, -2.0) + float2(-1.0, 1.0), 0.0, 1.0);
    out.uv = uv;
    
    return out;
}

[shader("fragment")]
float4 fragmentShader(VertexOut input) : SV_Target {    
    return sourceTexture.Sample(linearSampler, input.uv);
}
