[[vk::binding(0)]]
RWTexture2D<float4> outputTexture;

[numthreads(8, 8, 1)]
void computeShader(uint3 threadID : SV_DispatchThreadID) {
    outputTexture[threadID.xy] = float4(1.0, 0.0, 0.0, 1.0);
}