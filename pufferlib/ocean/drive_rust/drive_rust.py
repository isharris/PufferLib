import numpy as np
import gymnasium
import os

import pufferlib
from pufferlib.ocean.drive_rust import binding


class DriveRust(pufferlib.PufferEnv):
    def __init__(
        self,
        render_mode=None,
        report_interval=1,
        width=1280,
        height=1024,
        human_agent_idx=0,
        reward_vehicle_collision=-0.1,
        reward_offroad_collision=-0.1,
        reward_goal_post_respawn=0.5,
        reward_vehicle_collision_post_respawn=-0.25,
        spawn_immunity_timer=30,
        resample_frequency=91,
        num_maps=100,
        num_agents=512,
        buf=None,
        seed=1,
    ):
        self.render_mode = render_mode
        self.num_maps = num_maps
        self.report_interval = report_interval
        self.reward_vehicle_collision = reward_vehicle_collision
        self.reward_offroad_collision = reward_offroad_collision
        self.reward_goal_post_respawn = reward_goal_post_respawn
        self.reward_vehicle_collision_post_respawn = reward_vehicle_collision_post_respawn
        self.spawn_immunity_timer = spawn_immunity_timer
        self.human_agent_idx = human_agent_idx
        self.resample_frequency = resample_frequency
        self.num_obs = 7 + 63 * 7 + 200 * 7
        self.single_observation_space = gymnasium.spaces.Box(
            low=-1, high=1, shape=(self.num_obs,), dtype=np.float32
        )
        self.single_action_space = gymnasium.spaces.MultiDiscrete([7, 13])

        binary_path = "resources/drive/binaries/map_000.bin"
        if not os.path.exists(binary_path):
            raise FileNotFoundError(
                f"Required file {binary_path} not found. "
                "Please ensure the Drive maps are downloaded and installed correctly per docs."
            )

        agent_offsets, map_ids, num_envs = binding.shared(
            num_agents=num_agents, num_maps=num_maps
        )
        self.num_agents = num_agents
        self.agent_offsets = agent_offsets
        self.map_ids = map_ids
        self.num_envs = num_envs
        super().__init__(buf=buf)

        env_ids = []
        for i in range(num_envs):
            cur = agent_offsets[i]
            nxt = agent_offsets[i + 1]
            env_id = binding.env_init(
                self.observations[cur:nxt],
                self.actions[cur:nxt],
                self.rewards[cur:nxt],
                self.terminals[cur:nxt],
                self.truncations[cur:nxt],
                seed,
                human_agent_idx=human_agent_idx,
                reward_vehicle_collision=reward_vehicle_collision,
                reward_offroad_collision=reward_offroad_collision,
                reward_goal_post_respawn=reward_goal_post_respawn,
                reward_vehicle_collision_post_respawn=reward_vehicle_collision_post_respawn,
                spawn_immunity_timer=spawn_immunity_timer,
                map_id=map_ids[i],
                max_agents=nxt - cur,
            )
            env_ids.append(env_id)

        self.c_envs = binding.vectorize(*env_ids)

    def reset(self, seed=0):
        binding.vec_reset(self.c_envs, seed)
        self.tick = 0
        return self.observations, []

    def step(self, actions):
        self.terminals[:] = 0
        self.actions[:] = actions
        binding.vec_step(self.c_envs)
        self.tick += 1
        info = []
        if self.tick % self.report_interval == 0:
            log = binding.vec_log(self.c_envs)
            if log:
                info.append(log)
        if (
            self.tick > 0
            and self.resample_frequency > 0
            and self.tick % self.resample_frequency == 0
        ):
            self.tick = 0
            binding.vec_close(self.c_envs)
            agent_offsets, map_ids, num_envs = binding.shared(
                num_agents=self.num_agents, num_maps=self.num_maps
            )
            env_ids = []
            seed = np.random.randint(0, 2**32 - 1)
            for i in range(num_envs):
                cur = agent_offsets[i]
                nxt = agent_offsets[i + 1]
                env_id = binding.env_init(
                    self.observations[cur:nxt],
                    self.actions[cur:nxt],
                    self.rewards[cur:nxt],
                    self.terminals[cur:nxt],
                    self.truncations[cur:nxt],
                    seed,
                    human_agent_idx=self.human_agent_idx,
                    reward_vehicle_collision=self.reward_vehicle_collision,
                    reward_offroad_collision=self.reward_offroad_collision,
                    reward_goal_post_respawn=self.reward_goal_post_respawn,
                    reward_vehicle_collision_post_respawn=self.reward_vehicle_collision_post_respawn,
                    spawn_immunity_timer=self.spawn_immunity_timer,
                    map_id=map_ids[i],
                    max_agents=nxt - cur,
                )
                env_ids.append(env_id)
            self.c_envs = binding.vectorize(*env_ids)
            binding.vec_reset(self.c_envs, seed)
            self.terminals[:] = 1
        return (self.observations, self.rewards, self.terminals, self.truncations, info)

    def render(self):
        binding.vec_render(self.c_envs, 0)

    def close(self):
        binding.vec_close(self.c_envs)
