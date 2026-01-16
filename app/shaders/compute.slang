[[vk::binding(0)]]
RWTexture2D<float4> outputTexture;

[[vk::binding(1)]]
RaytracingAccelerationStructure scene;

[numthreads(8, 8, 1)]
void computeShader(uint3 threadID : SV_DispatchThreadID) {
    int2 dimensions;
    outputTexture.GetDimensions(dimensions.x, dimensions.y);
    if (threadID.x >= dimensions.x || threadID.y >= dimensions.y) {
        return;
    }

    float2 uv = (float2(threadID.xy) + 0.5) / float2(dimensions);
    float2 ndc = uv * 2.0 - 1.0;
    ndc.y = -ndc.y;

    RayDesc ray;
    ray.Origin = float3(0.0, 0.0, -2.0);
    ray.Direction = normalize(float3(ndc.x, ndc.y, 1.5));
    ray.TMin = 0.001;
    ray.TMax = 1000.0;

    RayQuery<RAY_FLAG_ACCEPT_FIRST_HIT_AND_END_SEARCH> query;
    query.TraceRayInline(scene, RAY_FLAG_ACCEPT_FIRST_HIT_AND_END_SEARCH, 0xFF, ray);
    while (query.Proceed()) {
    }

    float3 color = float3(0.05, 0.05, 0.08);
    if (query.CommittedStatus() == COMMITTED_TRIANGLE_HIT) {
        float2 bary = query.CommittedTriangleBarycentrics();
        float3 barycentric = float3(1.0 - bary.x - bary.y, bary.x, bary.y);
        color = barycentric;
    }

    outputTexture[threadID.xy] = float4(color, 1.0);
}