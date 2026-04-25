// V-trace advantage kernel. Mirrors the original CUDA kernel in
// pufferlib/extensions/cuda/pufferlib.cu.

#include <cuda_runtime.h>

__device__ void puff_advantage_row_cuda(
    const float* values, const float* rewards, const float* dones,
    const float* importance, float* advantages,
    float gamma, float lambda, float rho_clip, float c_clip, int horizon)
{
    float lastpufferlam = 0.0f;
    for (int t = horizon - 2; t >= 0; t--) {
        int t_next = t + 1;
        float nextnonterminal = 1.0f - dones[t_next];
        float rho_t = fminf(importance[t], rho_clip);
        float c_t = fminf(importance[t], c_clip);
        float delta = rho_t * (rewards[t_next] + gamma * values[t_next] * nextnonterminal - values[t]);
        lastpufferlam = delta + gamma * lambda * c_t * lastpufferlam * nextnonterminal;
        advantages[t] = lastpufferlam;
    }
}

__global__ void puff_advantage_kernel(
    const float* values, const float* rewards, const float* dones,
    const float* importance, float* advantages,
    float gamma, float lambda, float rho_clip, float c_clip,
    int num_steps, int horizon)
{
    int row = blockIdx.x * blockDim.x + threadIdx.x;
    if (row >= num_steps) return;
    int offset = row * horizon;
    puff_advantage_row_cuda(
        values + offset, rewards + offset, dones + offset,
        importance + offset, advantages + offset,
        gamma, lambda, rho_clip, c_clip, horizon);
}

extern "C" int puffer_advantage_cuda_launch(
    const float* values, const float* rewards, const float* dones,
    const float* importance, float* advantages,
    float gamma, float lambda, float rho_clip, float c_clip,
    int num_steps, int horizon)
{
    int threads = 256;
    int blocks = (num_steps + threads - 1) / threads;
    puff_advantage_kernel<<<blocks, threads>>>(
        values, rewards, dones, importance, advantages,
        gamma, lambda, rho_clip, c_clip, num_steps, horizon);
    return (int)cudaGetLastError();
}
