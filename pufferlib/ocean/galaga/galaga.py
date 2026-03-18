import gymnasium
import numpy as np

import pufferlib
from pufferlib.ocean.galaga import binding


class Galaga(pufferlib.PufferEnv):
    def __init__(self, num_envs=1, render_mode=None, buf=None, seed=0,
                 width=1024, height=768, frameskip=1):
        obs_size = 67  # 1 + 2*2 + 10*3 + 8*4
        self.single_observation_space = gymnasium.spaces.Box(
            low=0, high=1, shape=(obs_size,), dtype=np.float32)
        self.single_action_space = gymnasium.spaces.Discrete(4)
        self.render_mode = render_mode
        self.num_agents = num_envs

        super().__init__(buf)
        self.c_envs = binding.vec_init(
            self.observations, self.actions, self.rewards,
            self.terminals, self.truncations, num_envs, seed,
            width=width, height=height, frameskip=frameskip)

    def reset(self, seed=0):
        binding.vec_reset(self.c_envs, seed)
        return self.observations, []

    def step(self, actions):
        self.actions[:] = actions
        binding.vec_step(self.c_envs)
        info = [binding.vec_log(self.c_envs)]
        return (self.observations, self.rewards,
                self.terminals, self.truncations, info)

    def render(self):
        binding.vec_render(self.c_envs, 0)

    def close(self):
        binding.vec_close(self.c_envs)


if __name__ == '__main__':
    N = 4096
    env = Galaga(num_envs=N)
    env.reset()
    steps = 0

    CACHE = 1024
    actions = np.random.randint(0, 4, (CACHE, N))

    import time
    start = time.time()
    while time.time() - start < 10:
        env.step(actions[steps % CACHE])
        steps += 1

    sps = int(env.num_agents * steps / (time.time() - start))
    print(f'Galaga SPS: {sps:,}')
