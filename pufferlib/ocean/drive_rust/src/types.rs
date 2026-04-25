pub const VEHICLE: i32 = 1;
pub const ROAD_EDGE: i32 = 6;

pub const TRAJECTORY_LENGTH: usize = 91;
pub const CLASSIC: i32 = 0;

pub const VEHICLE_COLLISION: i32 = 1;
pub const OFFROAD: i32 = 2;

pub const GRID_CELL_SIZE: f32 = 5.0;
pub const MAX_ENTITIES_PER_CELL: usize = 10;
pub const SLOTS_PER_CELL: usize = MAX_ENTITIES_PER_CELL * 2 + 1;

pub const MAX_ROAD_SEGMENT_OBSERVATIONS: usize = 200;
pub const MAX_CARS: usize = 64;
pub const MAX_SPEED: f32 = 100.0;
pub const MAX_VEH_LEN: f32 = 30.0;
pub const MAX_VEH_WIDTH: f32 = 15.0;
pub const MAX_ROAD_SEGMENT_LENGTH: f32 = 100.0;
pub const MAX_ROAD_SCALE: f32 = 100.0;

pub const MAX_OBS: usize = 7 + 7 * (MAX_CARS - 1) + 7 * MAX_ROAD_SEGMENT_OBSERVATIONS;

pub const ACCELERATION_VALUES: [f32; 7] =
    [-4.0000, -2.6670, -1.3330, -0.0000, 1.3330, 2.6670, 4.0000];
pub const STEERING_VALUES: [f32; 13] = [
    -1.000, -0.833, -0.667, -0.500, -0.333, -0.167, 0.000, 0.167, 0.333, 0.500, 0.667, 0.833,
    1.000,
];

pub const CORNER_OFFSETS: [[f32; 2]; 4] = [[-1.0, 1.0], [1.0, 1.0], [1.0, -1.0], [-1.0, -1.0]];

pub const COLLISION_OFFSETS: [[i32; 2]; 25] = [
    [-2, -2], [-1, -2], [0, -2], [1, -2], [2, -2],
    [-2, -1], [-1, -1], [0, -1], [1, -1], [2, -1],
    [-2, 0],  [-1, 0],  [0, 0],  [1, 0],  [2, 0],
    [-2, 1],  [-1, 1],  [0, 1],  [1, 1],  [2, 1],
    [-2, 2],  [-1, 2],  [0, 2],  [1, 2],  [2, 2],
];

#[derive(Clone)]
pub struct Entity {
    pub entity_type: i32,
    pub array_size: usize,
    pub traj_x: Vec<f32>,
    pub traj_y: Vec<f32>,
    pub traj_z: Vec<f32>,
    pub traj_vx: Vec<f32>,
    pub traj_vy: Vec<f32>,
    pub traj_vz: Vec<f32>,
    pub traj_heading: Vec<f32>,
    pub traj_valid: Vec<i32>,
    pub width: f32,
    pub length: f32,
    pub height: f32,
    pub goal_position_x: f32,
    pub goal_position_y: f32,
    pub goal_position_z: f32,
    pub mark_as_expert: i32,
    pub collision_state: i32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub heading: f32,
    pub heading_x: f32,
    pub heading_y: f32,
    pub valid: i32,
    pub reached_goal: i32,
    pub respawn_timestep: i32,
    pub collided_before_goal: i32,
    pub reached_goal_this_episode: i32,
    pub active_agent: i32,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            entity_type: 0, array_size: 0,
            traj_x: Vec::new(), traj_y: Vec::new(), traj_z: Vec::new(),
            traj_vx: Vec::new(), traj_vy: Vec::new(), traj_vz: Vec::new(),
            traj_heading: Vec::new(), traj_valid: Vec::new(),
            width: 0.0, length: 0.0, height: 0.0,
            goal_position_x: 0.0, goal_position_y: 0.0, goal_position_z: 0.0,
            mark_as_expert: 0, collision_state: 0,
            x: 0.0, y: 0.0, z: 0.0,
            vx: 0.0, vy: 0.0, vz: 0.0,
            heading: 0.0, heading_x: 0.0, heading_y: 0.0,
            valid: 0, reached_goal: 0, respawn_timestep: 0,
            collided_before_goal: 0, reached_goal_this_episode: 0, active_agent: 0,
        }
    }
}

#[derive(Clone, Default)]
pub struct Log {
    pub episode_return: f32,
    pub episode_length: f32,
    pub perf: f32,
    pub score: f32,
    pub offroad_rate: f32,
    pub collision_rate: f32,
    pub clean_collision_rate: f32,
    pub completion_rate: f32,
    pub dnf_rate: f32,
    pub n: f32,
}

impl Log {
    pub fn add_from(&mut self, other: &Log) {
        self.episode_return += other.episode_return;
        self.episode_length += other.episode_length;
        self.perf += other.perf;
        self.score += other.score;
        self.offroad_rate += other.offroad_rate;
        self.collision_rate += other.collision_rate;
        self.clean_collision_rate += other.clean_collision_rate;
        self.completion_rate += other.completion_rate;
        self.dnf_rate += other.dnf_rate;
        self.n += other.n;
    }

    pub fn normalize(&mut self) {
        if self.n > 0.0 {
            let n = self.n;
            self.episode_return /= n;
            self.episode_length /= n;
            self.perf /= n;
            self.score /= n;
            self.offroad_rate /= n;
            self.collision_rate /= n;
            self.clean_collision_rate /= n;
            self.completion_rate /= n;
            self.dnf_rate /= n;
            self.n /= n;
        }
    }
}

pub struct Drive {
    pub observations: *mut f32,
    pub actions: *mut i32,
    pub rewards: *mut f32,
    pub terminals: *mut u8,

    pub log: Log,
    pub logs: Vec<Log>,
    pub num_agents: i32,
    pub active_agent_count: usize,
    pub active_agent_indices: Vec<i32>,
    pub human_agent_idx: i32,
    pub entities: Vec<Entity>,
    pub num_entities: usize,
    pub num_cars: usize,
    pub num_objects: usize,
    pub num_roads: usize,
    pub static_car_count: usize,
    pub static_car_indices: Vec<i32>,
    pub expert_static_car_count: usize,
    pub expert_static_car_indices: Vec<i32>,
    pub timestep: i32,
    pub dynamics_model: i32,
    pub map_corners: [f32; 4],
    pub grid_cells: Vec<i32>,
    pub grid_cols: usize,
    pub grid_rows: usize,
    pub vision_range: usize,
    pub neighbor_offsets: Vec<i32>,
    pub neighbor_cache_entities: Vec<i32>,
    pub neighbor_cache_indices: Vec<i32>,
    pub reward_vehicle_collision: f32,
    pub reward_offroad_collision: f32,
    pub map_name: String,
    pub world_mean_x: f32,
    pub world_mean_y: f32,
    pub spawn_immunity_timer: i32,
    pub reward_goal_post_respawn: f32,
    pub reward_vehicle_collision_post_respawn: f32,
}

impl Drive {
    pub fn new() -> Self {
        Self {
            observations: std::ptr::null_mut(),
            actions: std::ptr::null_mut(),
            rewards: std::ptr::null_mut(),
            terminals: std::ptr::null_mut(),
            log: Log::default(),
            logs: Vec::new(),
            num_agents: 0,
            active_agent_count: 0,
            active_agent_indices: Vec::new(),
            human_agent_idx: 0,
            entities: Vec::new(),
            num_entities: 0,
            num_cars: 0,
            num_objects: 0,
            num_roads: 0,
            static_car_count: 0,
            static_car_indices: Vec::new(),
            expert_static_car_count: 0,
            expert_static_car_indices: Vec::new(),
            timestep: 0,
            dynamics_model: CLASSIC,
            map_corners: [0.0; 4],
            grid_cells: Vec::new(),
            grid_cols: 0,
            grid_rows: 0,
            vision_range: 0,
            neighbor_offsets: Vec::new(),
            neighbor_cache_entities: Vec::new(),
            neighbor_cache_indices: Vec::new(),
            reward_vehicle_collision: 0.0,
            reward_offroad_collision: 0.0,
            map_name: String::new(),
            world_mean_x: 0.0,
            world_mean_y: 0.0,
            spawn_immunity_timer: 0,
            reward_goal_post_respawn: 0.0,
            reward_vehicle_collision_post_respawn: 0.0,
        }
    }
}
