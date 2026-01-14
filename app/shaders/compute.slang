[[vk::binding(0)]]
RWTexture2D<float4> outputTexture;

[numthreads(8, 8, 1)]
void computeShader(uint3 threadID : SV_DispatchThreadID) {
    int2 dimensions;
    outputTexture.GetDimensions(dimensions.x, dimensions.y);
    // if outside of grid, return
    if (threadID.x >= dimensions.x || threadID.y >= dimensions.y) {
        return;
    }

    outputTexture[threadID.xy] = float4(1.0, 1.0, 0.0, 1.0);
}