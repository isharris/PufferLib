use crate::map::load_map_binary;
use crate::types::*;

fn distance_2d(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    (dx * dx + dy * dy).sqrt()
}

fn check_line_intersection(p1: [f32; 2], p2: [f32; 2], q1: [f32; 2], q2: [f32; 2]) -> bool {
    if p1[0].max(p2[0]) < q1[0].min(q2[0])
        || p1[0].min(p2[0]) > q1[0].max(q2[0])
        || p1[1].max(p2[1]) < q1[1].min(q2[1])
        || p1[1].min(p2[1]) > q1[1].max(q2[1])
    {
        return false;
    }
    let dx1 = p2[0] - p1[0];
    let dy1 = p2[1] - p1[1];
    let dx2 = q2[0] - q1[0];
    let dy2 = q2[1] - q1[1];
    let cross = dx1 * dy2 - dy1 * dx2;
    if cross == 0.0 {
        return false;
    }
    let dx3 = p1[0] - q1[0];
    let dy3 = p1[1] - q1[1];
    let s = (dx1 * dy3 - dy1 * dx3) / cross;
    let t = (dx2 * dy3 - dy2 * dx3) / cross;
    s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0
}

fn check_obb_collision(car1: &Entity, car2: &Entity) -> bool {
    let cos1 = car1.heading_x;
    let sin1 = car1.heading_y;
    let cos2 = car2.heading_x;
    let sin2 = car2.heading_y;
    let hl1 = car1.length * 0.5;
    let hw1 = car1.width * 0.5;
    let hl2 = car2.length * 0.5;
    let hw2 = car2.width * 0.5;

    let c1 = [
        [car1.x + hl1 * cos1 - hw1 * sin1, car1.y + hl1 * sin1 + hw1 * cos1],
        [car1.x + hl1 * cos1 + hw1 * sin1, car1.y + hl1 * sin1 - hw1 * cos1],
        [car1.x - hl1 * cos1 - hw1 * sin1, car1.y - hl1 * sin1 + hw1 * cos1],
        [car1.x - hl1 * cos1 + hw1 * sin1, car1.y - hl1 * sin1 - hw1 * cos1],
    ];
    let c2 = [
        [car2.x + hl2 * cos2 - hw2 * sin2, car2.y + hl2 * sin2 + hw2 * cos2],
        [car2.x + hl2 * cos2 + hw2 * sin2, car2.y + hl2 * sin2 - hw2 * cos2],
        [car2.x - hl2 * cos2 - hw2 * sin2, car2.y - hl2 * sin2 + hw2 * cos2],
        [car2.x - hl2 * cos2 + hw2 * sin2, car2.y - hl2 * sin2 - hw2 * cos2],
    ];

    let axes = [
        [cos1, sin1],
        [-sin1, cos1],
        [cos2, sin2],
        [-sin2, cos2],
    ];
    for axis in &axes {
        let (mut min1, mut max1) = (f32::INFINITY, f32::NEG_INFINITY);
        let (mut min2, mut max2) = (f32::INFINITY, f32::NEG_INFINITY);
        for c in &c1 {
            let p = c[0] * axis[0] + c[1] * axis[1];
            min1 = min1.min(p);
            max1 = max1.max(p);
        }
        for c in &c2 {
            let p = c[0] * axis[0] + c[1] * axis[1];
            min2 = min2.min(p);
            max2 = max2.max(p);
        }
        if max1 < min2 || min1 > max2 {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Grid
// ---------------------------------------------------------------------------

impl Drive {
    pub fn get_grid_index(&self, x: f32, y: f32) -> i32 {
        if self.map_corners[0] >= self.map_corners[2]
            || self.map_corners[1] >= self.map_corners[3]
        {
            return -1;
        }
        let rel_x = x - self.map_corners[0];
        let rel_y = y - self.map_corners[1];
        let gx = (rel_x / GRID_CELL_SIZE) as i32;
        let gy = (rel_y / GRID_CELL_SIZE) as i32;
        if gx < 0 || gx >= self.grid_cols as i32 || gy < 0 || gy >= self.grid_rows as i32 {
            return -1;
        }
        gy * self.grid_cols as i32 + gx
    }

    fn add_entity_to_grid(&mut self, gi: i32, entity_idx: i32, geom_idx: i32) {
        if gi < 0 {
            return;
        }
        let base = gi as usize * SLOTS_PER_CELL;
        let count = self.grid_cells[base] as usize;
        if count >= MAX_ENTITIES_PER_CELL {
            return;
        }
        self.grid_cells[base + count * 2 + 1] = entity_idx;
        self.grid_cells[base + count * 2 + 2] = geom_idx;
        self.grid_cells[base] = (count + 1) as i32;
    }

    fn init_grid_map(&mut self) {
        let mut first = false;
        let (mut tlx, mut tly, mut brx, mut bry) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for e in &self.entities {
            if e.entity_type > 3 && e.entity_type < 7 {
                for j in 0..e.array_size {
                    if e.traj_x[j] == -10000.0 || e.traj_y[j] == -10000.0 {
                        continue;
                    }
                    if !first {
                        tlx = e.traj_x[j]; brx = e.traj_x[j];
                        tly = e.traj_y[j]; bry = e.traj_y[j];
                        first = true;
                        continue;
                    }
                    tlx = tlx.min(e.traj_x[j]); brx = brx.max(e.traj_x[j]);
                    tly = tly.min(e.traj_y[j]); bry = bry.max(e.traj_y[j]);
                }
            }
        }
        self.map_corners = [tlx, tly, brx, bry];
        self.grid_cols = ((brx - tlx) / GRID_CELL_SIZE).ceil() as usize;
        self.grid_rows = ((bry - tly) / GRID_CELL_SIZE).ceil() as usize;
        let cell_count = self.grid_cols * self.grid_rows;
        self.grid_cells = vec![0i32; cell_count * SLOTS_PER_CELL];

        let mut to_add = Vec::new();
        for i in 0..self.num_entities {
            let e = &self.entities[i];
            if e.entity_type > 3 && e.entity_type < 7 {
                for j in 0..e.array_size.saturating_sub(1) {
                    let xc = (e.traj_x[j] + e.traj_x[j + 1]) * 0.5;
                    let yc = (e.traj_y[j] + e.traj_y[j + 1]) * 0.5;
                    to_add.push((xc, yc, i as i32, j as i32));
                }
            }
        }
        for (x, y, ei, gi) in to_add {
            let idx = self.get_grid_index(x, y);
            self.add_entity_to_grid(idx, ei, gi);
        }
    }

    fn init_neighbor_offsets(&mut self) {
        let max = self.vision_range * self.vision_range;
        self.neighbor_offsets = vec![0i32; max * 2];
        let dx = [1i32, 0, -1, 0];
        let dy = [0i32, 1, 0, -1];
        let (mut x, mut y) = (0i32, 0i32);
        let mut dir = 0usize;
        let mut steps_to_take = 1;
        let mut steps_taken = 0;
        let mut segments = 0;
        let mut total = 0usize;
        let half = (self.vision_range / 2) as i32;
        let mut ci = 0usize;
        self.neighbor_offsets[ci] = 0;
        self.neighbor_offsets[ci + 1] = 0;
        ci += 2;
        total += 1;
        while total < max {
            x += dx[dir];
            y += dy[dir];
            if x.abs() <= half && y.abs() <= half {
                self.neighbor_offsets[ci] = x;
                self.neighbor_offsets[ci + 1] = y;
                ci += 2;
                total += 1;
            }
            steps_taken += 1;
            if steps_taken == steps_to_take {
                steps_taken = 0;
                dir = (dir + 1) % 4;
                segments += 1;
                if segments % 2 == 0 {
                    steps_to_take += 1;
                }
            }
        }
    }

    fn cache_neighbor_offsets(&mut self) {
        let cell_count = self.grid_cols * self.grid_rows;
        let vr2 = self.vision_range * self.vision_range;
        self.neighbor_cache_indices = vec![0i32; cell_count + 1];

        let mut count = 0i32;
        for i in 0..cell_count {
            let cx = (i % self.grid_cols) as i32;
            let cy = (i / self.grid_cols) as i32;
            self.neighbor_cache_indices[i] = count;
            for j in 0..vr2 {
                let nx = cx + self.neighbor_offsets[j * 2];
                let ny = cy + self.neighbor_offsets[j * 2 + 1];
                if nx < 0 || nx >= self.grid_cols as i32 || ny < 0 || ny >= self.grid_rows as i32 {
                    continue;
                }
                let gi = (self.grid_cols as i32 * ny + nx) as usize;
                count += self.grid_cells[gi * SLOTS_PER_CELL] * 2;
            }
        }
        self.neighbor_cache_indices[cell_count] = count;
        self.neighbor_cache_entities = vec![0i32; count as usize];

        for i in 0..cell_count {
            let cx = (i % self.grid_cols) as i32;
            let cy = (i / self.grid_cols) as i32;
            let mut ncb = 0usize;
            for j in 0..vr2 {
                let nx = cx + self.neighbor_offsets[j * 2];
                let ny = cy + self.neighbor_offsets[j * 2 + 1];
                if nx < 0 || nx >= self.grid_cols as i32 || ny < 0 || ny >= self.grid_rows as i32 {
                    continue;
                }
                let gi = (self.grid_cols as i32 * ny + nx) as usize;
                let gc = self.grid_cells[gi * SLOTS_PER_CELL] as usize;
                let base = self.neighbor_cache_indices[i] as usize;
                let src = gi * SLOTS_PER_CELL + 1;
                let dst = base + ncb;
                let len = gc * 2;
                self.neighbor_cache_entities[dst..dst + len]
                    .copy_from_slice(&self.grid_cells[src..src + len]);
                ncb += len;
            }
        }
    }

    fn get_neighbor_cache_entities(&self, cell_idx: i32, out: &mut [i32]) -> usize {
        if cell_idx < 0 || cell_idx >= (self.grid_cols * self.grid_rows) as i32 {
            return 0;
        }
        let ci = cell_idx as usize;
        let base = self.neighbor_cache_indices[ci] as usize;
        let end = self.neighbor_cache_indices[ci + 1] as usize;
        let pairs = ((end - base) / 2).min(out.len() / 2);
        let n = pairs * 2;
        out[..n].copy_from_slice(&self.neighbor_cache_entities[base..base + n]);
        pairs
    }

    fn check_neighbors(&self, x: f32, y: f32, out: &mut [i32], offsets: &[[i32; 2]]) -> usize {
        let index = self.get_grid_index(x, y);
        if index < 0 {
            return 0;
        }
        let gx = index % self.grid_cols as i32;
        let gy = index / self.grid_cols as i32;
        let mut cnt = 0usize;
        let max = out.len();
        for off in offsets {
            let nx = gx + off[0];
            let ny = gy + off[1];
            if nx < 0 || nx >= self.grid_cols as i32 || ny < 0 || ny >= self.grid_rows as i32 {
                continue;
            }
            let ni = (ny * self.grid_cols as i32 + nx) as usize * SLOTS_PER_CELL;
            let gc = self.grid_cells[ni] as usize;
            for j in 0..gc {
                if cnt + 1 >= max {
                    break;
                }
                out[cnt] = self.grid_cells[ni + 1 + j * 2];
                out[cnt + 1] = self.grid_cells[ni + 2 + j * 2];
                cnt += 2;
            }
        }
        cnt
    }
}

// ---------------------------------------------------------------------------
// Collision
// ---------------------------------------------------------------------------

impl Drive {
    pub fn collision_check(&mut self, agent_idx: usize) -> i32 {
        if self.entities[agent_idx].x == -10000.0 {
            return -1;
        }
        let half_l = self.entities[agent_idx].length * 0.5;
        let half_w = self.entities[agent_idx].width * 0.5;
        let cos_h = self.entities[agent_idx].heading.cos();
        let sin_h = self.entities[agent_idx].heading.sin();
        let ax = self.entities[agent_idx].x;
        let ay = self.entities[agent_idx].y;

        let mut corners = [[0.0f32; 2]; 4];
        for i in 0..4 {
            corners[i][0] =
                ax + CORNER_OFFSETS[i][0] * half_l * cos_h - CORNER_OFFSETS[i][1] * half_w * sin_h;
            corners[i][1] =
                ay + CORNER_OFFSETS[i][0] * half_l * sin_h + CORNER_OFFSETS[i][1] * half_w * cos_h;
        }

        let mut collided = 0i32;
        let mut car_collided_with = -1i32;

        // Offroad check
        let mut elist = [0i32; MAX_ENTITIES_PER_CELL * 2 * 25];
        let ls = self.check_neighbors(ax, ay, &mut elist, &COLLISION_OFFSETS);
        let mut k = 0;
        while k < ls {
            let eidx = elist[k];
            if eidx == -1 || eidx == agent_idx as i32 {
                k += 2;
                continue;
            }
            if self.entities[eidx as usize].entity_type != ROAD_EDGE {
                k += 2;
                continue;
            }
            let gi = elist[k + 1] as usize;
            let s = [
                self.entities[eidx as usize].traj_x[gi],
                self.entities[eidx as usize].traj_y[gi],
            ];
            let e = [
                self.entities[eidx as usize].traj_x[gi + 1],
                self.entities[eidx as usize].traj_y[gi + 1],
            ];
            let mut found = false;
            for c in 0..4 {
                if check_line_intersection(corners[c], corners[(c + 1) % 4], s, e) {
                    collided = OFFROAD;
                    found = true;
                    break;
                }
            }
            if found {
                break;
            }
            k += 2;
        }

        // Vehicle collision check
        for j in 0..MAX_CARS {
            let index: i32 = if j < self.active_agent_count {
                self.active_agent_indices[j]
            } else if j < self.num_cars {
                self.static_car_indices[j - self.active_agent_count]
            } else {
                continue;
            };
            if index == -1 || index == agent_idx as i32 {
                continue;
            }
            let dx = self.entities[index as usize].x - ax;
            let dy = self.entities[index as usize].y - ay;
            if dx * dx + dy * dy > 225.0 {
                continue;
            }
            if check_obb_collision(&self.entities[agent_idx], &self.entities[index as usize]) {
                collided = VEHICLE_COLLISION;
                car_collided_with = index;
                break;
            }
        }

        self.entities[agent_idx].collision_state = collided;

        let is_active = self.entities[agent_idx].active_agent;
        let respawned = self.entities[agent_idx].respawn_timestep != -1;
        if collided == VEHICLE_COLLISION && is_active == 1 && respawned {
            self.entities[agent_idx].collision_state = 0;
        }
        if collided == OFFROAD {
            return -1;
        }
        if car_collided_with == -1 {
            return -1;
        }
        if self.entities[car_collided_with as usize].respawn_timestep != -1 {
            self.entities[agent_idx].collision_state = 0;
        }
        car_collided_with
    }
}

// ---------------------------------------------------------------------------
// Dynamics
// ---------------------------------------------------------------------------

impl Drive {
    fn move_dynamics(&mut self, action_idx: usize, agent_idx: usize) {
        if self.dynamics_model != CLASSIC {
            return;
        }
        unsafe {
            let accel_i = (*self.actions.add(action_idx * 2)) as usize;
            let steer_i = (*self.actions.add(action_idx * 2 + 1)) as usize;
            let accel = ACCELERATION_VALUES[accel_i];
            let steer = STEERING_VALUES[steer_i];

            let a = &self.entities[agent_idx];
            let heading = a.heading;
            let mut speed = (a.vx * a.vx + a.vy * a.vy).sqrt();
            let dt = 0.1f32;
            speed += 0.5 * accel * dt;
            speed = speed.clamp(-MAX_SPEED, MAX_SPEED);
            let beta = (0.5 * steer.tan()).tanh();
            let yaw_rate = (speed * beta.cos() * steer.tan()) / a.length;
            let new_vx = speed * (heading + beta).cos();
            let new_vy = speed * (heading + beta).sin();

            let e = &mut self.entities[agent_idx];
            e.x += new_vx * dt;
            e.y += new_vy * dt;
            e.heading = heading + yaw_rate * dt;
            e.heading_x = e.heading.cos();
            e.heading_y = e.heading.sin();
            e.vx = new_vx;
            e.vy = new_vy;
        }
    }

    fn move_expert(&mut self, agent_idx: usize) {
        let t = self.timestep as usize;
        let e = &mut self.entities[agent_idx];
        e.x = e.traj_x[t];
        e.y = e.traj_y[t];
        e.z = e.traj_z[t];
        e.heading = e.traj_heading[t];
        e.heading_x = e.heading.cos();
        e.heading_y = e.heading.sin();
    }
}

// ---------------------------------------------------------------------------
// Observations
// ---------------------------------------------------------------------------

impl Drive {
    unsafe fn compute_observations(&mut self) {
        let obs_ptr = self.observations;
        std::ptr::write_bytes(obs_ptr, 0, self.active_agent_count * MAX_OBS);

        for i in 0..self.active_agent_count {
            let obs = obs_ptr.add(i * MAX_OBS);
            let ego_idx = self.active_agent_indices[i] as usize;
            let ego = &self.entities[ego_idx];
            if ego.entity_type > 3 {
                break;
            }
            if ego.respawn_timestep != -1 {
                *obs.add(6) = 1.0;
            }
            let cos_h = ego.heading_x;
            let sin_h = ego.heading_y;
            let ego_speed = (ego.vx * ego.vx + ego.vy * ego.vy).sqrt();
            let gx = ego.goal_position_x - ego.x;
            let gy = ego.goal_position_y - ego.y;
            let rgx = gx * cos_h + gy * sin_h;
            let rgy = -gx * sin_h + gy * cos_h;
            *obs.add(0) = rgx * 0.005;
            *obs.add(1) = rgy * 0.005;
            *obs.add(2) = ego_speed * 0.01;
            *obs.add(3) = ego.width / MAX_VEH_WIDTH;
            *obs.add(4) = ego.length / MAX_VEH_LEN;
            *obs.add(5) = if ego.collision_state > 0 { 1.0 } else { 0.0 };

            let mut obs_idx = 7usize;
            let mut cars_seen = 0usize;
            for j in 0..MAX_CARS {
                let index: i32 = if j < self.active_agent_count {
                    self.active_agent_indices[j]
                } else if j < self.num_cars {
                    self.static_car_indices[j - self.active_agent_count]
                } else {
                    continue;
                };
                if index == -1 {
                    continue;
                }
                if self.entities[index as usize].entity_type > 3 {
                    break;
                }
                if index == self.active_agent_indices[i] {
                    continue;
                }
                let other = &self.entities[index as usize];
                if ego.respawn_timestep != -1 || other.respawn_timestep != -1 {
                    continue;
                }
                let dx = other.x - ego.x;
                let dy = other.y - ego.y;
                if dx * dx + dy * dy > 2500.0 {
                    continue;
                }
                *obs.add(obs_idx) = (dx * cos_h + dy * sin_h) * 0.02;
                *obs.add(obs_idx + 1) = (-dx * sin_h + dy * cos_h) * 0.02;
                *obs.add(obs_idx + 2) = other.width / MAX_VEH_WIDTH;
                *obs.add(obs_idx + 3) = other.length / MAX_VEH_LEN;
                *obs.add(obs_idx + 4) =
                    other.heading_x * ego.heading_x + other.heading_y * ego.heading_y;
                *obs.add(obs_idx + 5) =
                    other.heading_y * ego.heading_x - other.heading_x * ego.heading_y;
                let os = (other.vx * other.vx + other.vy * other.vy).sqrt();
                *obs.add(obs_idx + 6) = os / MAX_SPEED;
                cars_seen += 1;
                obs_idx += 7;
            }
            let remaining = (MAX_CARS - 1 - cars_seen) * 7;
            std::ptr::write_bytes(obs.add(obs_idx) as *mut u8, 0, remaining * 4);
            obs_idx += remaining;

            // Road observations
            let mut elist = [0i32; MAX_ROAD_SEGMENT_OBSERVATIONS * 2];
            let grid_idx = self.get_grid_index(ego.x, ego.y);
            let ls = self.get_neighbor_cache_entities(grid_idx, &mut elist);
            for k in 0..ls {
                let eidx = elist[k * 2] as usize;
                let gidx = elist[k * 2 + 1] as usize;
                let ent = &self.entities[eidx];
                let sx = ent.traj_x[gidx];
                let sy = ent.traj_y[gidx];
                let ex = ent.traj_x[gidx + 1];
                let ey = ent.traj_y[gidx + 1];
                let mx = (sx + ex) * 0.5;
                let my = (sy + ey) * 0.5;
                let rx = mx - ego.x;
                let ry = my - ego.y;
                let xo = rx * cos_h + ry * sin_h;
                let yo = -rx * sin_h + ry * cos_h;
                let ddx = ex - mx;
                let ddy = ey - my;
                let length = ((mx - ex).powi(2) + (my - ey).powi(2)).sqrt();
                let hypot = (ddx * ddx + ddy * ddy).sqrt();
                let (dnx, dny) = if hypot > 0.0 {
                    (ddx / hypot, ddy / hypot)
                } else {
                    (ddx, ddy)
                };
                *obs.add(obs_idx) = xo * 0.02;
                *obs.add(obs_idx + 1) = yo * 0.02;
                *obs.add(obs_idx + 2) = length / MAX_ROAD_SEGMENT_LENGTH;
                *obs.add(obs_idx + 3) = 0.1 / MAX_ROAD_SCALE;
                *obs.add(obs_idx + 4) = dnx * cos_h + dny * sin_h;
                *obs.add(obs_idx + 5) = -dnx * sin_h + dny * cos_h;
                *obs.add(obs_idx + 6) = ent.entity_type as f32 - 4.0;
                obs_idx += 7;
            }
            let rem = (MAX_ROAD_SEGMENT_OBSERVATIONS - ls) * 7;
            std::ptr::write_bytes(obs.add(obs_idx) as *mut u8, 0, rem * 4);
        }
    }
}

// ---------------------------------------------------------------------------
// Init / Reset / Step
// ---------------------------------------------------------------------------

impl Drive {
    fn set_means(&mut self) {
        let mut mx = 0.0f32;
        let mut my = 0.0f32;
        let mut pc = 0i64;
        for e in &self.entities {
            if e.entity_type == VEHICLE {
                for j in 0..e.array_size {
                    if e.traj_valid[j] != 0 {
                        pc += 1;
                        mx += (e.traj_x[j] - mx) / pc as f32;
                        my += (e.traj_y[j] - my) / pc as f32;
                    }
                }
            } else if e.entity_type >= 4 {
                for j in 0..e.array_size {
                    pc += 1;
                    mx += (e.traj_x[j] - mx) / pc as f32;
                    my += (e.traj_y[j] - my) / pc as f32;
                }
            }
        }
        self.world_mean_x = mx;
        self.world_mean_y = my;
        for e in &mut self.entities {
            if e.entity_type == VEHICLE || e.entity_type >= 4 {
                for j in 0..e.array_size {
                    if e.traj_x[j] == -10000.0 {
                        continue;
                    }
                    e.traj_x[j] -= mx;
                    e.traj_y[j] -= my;
                }
                e.goal_position_x -= mx;
                e.goal_position_y -= my;
            }
        }
    }

    fn set_start_position(&mut self) {
        for i in 0..self.num_entities {
            let is_active = self
                .active_agent_indices
                .iter()
                .any(|&idx| idx == i as i32);
            let e = &mut self.entities[i];
            e.x = e.traj_x[0];
            e.y = e.traj_y[0];
            e.z = e.traj_z[0];
            if e.entity_type > 3 || e.entity_type == 0 {
                continue;
            }
            if !is_active {
                e.vx = 0.0;
                e.vy = 0.0;
                e.vz = 0.0;
                e.reached_goal = 0;
                e.collided_before_goal = 0;
            } else {
                e.vx = e.traj_vx[0];
                e.vy = e.traj_vy[0];
                e.vz = e.traj_vz[0];
            }
            e.heading = e.traj_heading[0];
            e.heading_x = e.heading.cos();
            e.heading_y = e.heading.sin();
            e.valid = e.traj_valid[0];
            e.collision_state = 0;
            e.respawn_timestep = -1;
        }
    }

    fn valid_active_agent(&mut self, idx: usize) -> f32 {
        let cos_h = self.entities[idx].traj_heading[0].cos();
        let sin_h = self.entities[idx].traj_heading[0].sin();
        let gx = self.entities[idx].goal_position_x - self.entities[idx].traj_x[0];
        let gy = self.entities[idx].goal_position_y - self.entities[idx].traj_y[0];
        let rgx = gx * cos_h + gy * sin_h;
        let rgy = -gx * sin_h + gy * cos_h;
        let dist = distance_2d(0.0, 0.0, rgx, rgy);
        self.entities[idx].width *= 0.7;
        self.entities[idx].length *= 0.7;
        if dist >= 2.0
            && self.entities[idx].mark_as_expert == 0
            && self.active_agent_count < self.num_agents as usize
        {
            return (dist as i32) as f32;
        }
        0.0
    }

    pub fn set_active_agents(&mut self) {
        self.active_agent_count = 0;
        self.static_car_count = 0;
        self.num_cars = 1;
        self.expert_static_car_count = 0;
        let mut active = Vec::with_capacity(MAX_CARS);
        let mut statc = Vec::with_capacity(MAX_CARS);
        let mut expert = Vec::with_capacity(MAX_CARS);

        if self.num_agents == 0 {
            self.num_agents = MAX_CARS as i32;
        }
        if self.num_objects == 0 {
            self.active_agent_count = 0;
            self.num_cars = 0;
            self.active_agent_indices = active;
            self.static_car_indices = statc;
            self.expert_static_car_indices = expert;
            return;
        }

        let first = self.num_objects - 1;
        let d = self.valid_active_agent(first);
        if d > 0.0 {
            self.active_agent_count = 1;
            active.push(first as i32);
            self.entities[first].active_agent = 1;
            self.num_cars = 1;
        } else {
            self.active_agent_count = 0;
            self.num_cars = 0;
        }

        for i in 0..self.num_objects - 1 {
            if self.num_cars >= MAX_CARS {
                break;
            }
            if self.entities[i].entity_type != 1 {
                continue;
            }
            if self.entities[i].traj_valid[0] != 1 {
                continue;
            }
            self.num_cars += 1;
            let d = self.valid_active_agent(i);
            if d > 0.0 {
                active.push(i as i32);
                self.active_agent_count += 1;
                self.entities[i].active_agent = 1;
            } else {
                statc.push(i as i32);
                self.static_car_count += 1;
                self.entities[i].active_agent = 0;
                if self.entities[i].mark_as_expert == 1
                    || (d >= 2.0 && self.active_agent_count == self.num_agents as usize)
                {
                    expert.push(i as i32);
                    self.expert_static_car_count += 1;
                    self.entities[i].mark_as_expert = 1;
                }
            }
        }
        self.active_agent_indices = active;
        self.static_car_indices = statc;
        self.expert_static_car_indices = expert;
    }

    fn remove_bad_trajectories(&mut self) {
        self.set_start_position();
        let n = self.active_agent_count;
        let mut collided_flags = vec![0i32; n];
        let mut collided_with = vec![-1i32; n];

        for _t in 0..TRAJECTORY_LENGTH {
            for i in 0..self.active_agent_count {
                self.move_expert(self.active_agent_indices[i] as usize);
            }
            for i in 0..self.expert_static_car_count {
                let idx = self.expert_static_car_indices[i] as usize;
                if self.entities[idx].x == -10000.0 {
                    continue;
                }
                self.move_expert(idx);
            }
            for i in 0..self.active_agent_count {
                let idx = self.active_agent_indices[i] as usize;
                self.entities[idx].collision_state = 0;
                let cw = self.collision_check(idx);
                if self.entities[idx].collision_state > 0 && collided_flags[i] == 0 {
                    collided_flags[i] = 1;
                    collided_with[i] = cw;
                }
            }
            self.timestep += 1;
        }

        for i in 0..n {
            if collided_with[i] == -1 {
                continue;
            }
            for j in 0..self.static_car_count {
                let si = self.static_car_indices[j] as usize;
                if si as i32 != collided_with[i] {
                    continue;
                }
                self.entities[si].traj_x[0] = -10000.0;
                self.entities[si].traj_y[0] = -10000.0;
            }
        }
        self.timestep = 0;
    }

    pub fn init(&mut self) -> Result<(), String> {
        self.human_agent_idx = 0;
        self.timestep = 0;
        let (entities, num_objects, num_roads) = load_map_binary(&self.map_name)
            .map_err(|e| format!("Failed to load map '{}': {}", self.map_name, e))?;
        self.entities = entities;
        self.num_objects = num_objects;
        self.num_roads = num_roads;
        self.num_entities = num_objects + num_roads;
        self.dynamics_model = CLASSIC;
        self.set_means();
        self.init_grid_map();
        self.vision_range = 21;
        self.init_neighbor_offsets();
        self.cache_neighbor_offsets();
        self.set_active_agents();
        self.remove_bad_trajectories();
        self.set_start_position();
        self.logs = vec![Log::default(); self.active_agent_count];
        Ok(())
    }

    fn add_log(&mut self) {
        for i in 0..self.active_agent_count {
            let eidx = self.active_agent_indices[i] as usize;
            if self.entities[eidx].reached_goal_this_episode != 0 {
                self.log.completion_rate += 1.0;
            }
            let offroad = self.logs[i].offroad_rate as i32;
            self.log.offroad_rate += offroad as f32;
            let collided = self.logs[i].collision_rate as i32;
            self.log.collision_rate += collided as f32;
            let clean = self.logs[i].clean_collision_rate as i32;
            self.log.clean_collision_rate += clean as f32;
            if self.entities[eidx].reached_goal_this_episode != 0
                && self.entities[eidx].collided_before_goal == 0
            {
                self.log.score += 1.0;
                self.log.perf += 1.0;
            }
            if offroad == 0 && collided == 0 && self.entities[eidx].reached_goal_this_episode == 0 {
                self.log.dnf_rate += 1.0;
            }
            self.log.episode_length += self.logs[i].episode_length;
            self.log.episode_return += self.logs[i].episode_return;
            self.log.n += 1.0;
        }
    }

    fn respawn_agent(&mut self, agent_idx: usize) {
        let e = &mut self.entities[agent_idx];
        e.x = e.traj_x[0];
        e.y = e.traj_y[0];
        e.heading = e.traj_heading[0];
        e.heading_x = e.heading.cos();
        e.heading_y = e.heading.sin();
        e.vx = e.traj_vx[0];
        e.vy = e.traj_vy[0];
        e.reached_goal = 0;
        e.respawn_timestep = self.timestep;
    }

    /// # Safety
    /// Caller must ensure observation/action/reward/terminal buffer pointers are valid.
    pub unsafe fn c_reset(&mut self) {
        self.timestep = 0;
        self.set_start_position();
        for x in 0..self.active_agent_count {
            self.logs[x] = Log::default();
            let idx = self.active_agent_indices[x] as usize;
            self.entities[idx].respawn_timestep = -1;
            self.entities[idx].reached_goal = 0;
            self.entities[idx].collided_before_goal = 0;
            self.entities[idx].reached_goal_this_episode = 0;
            self.collision_check(idx);
        }
        self.compute_observations();
    }

    /// # Safety
    /// Caller must ensure observation/action/reward/terminal buffer pointers are valid.
    pub unsafe fn c_step(&mut self) {
        let n = self.active_agent_count;
        std::ptr::write_bytes(self.rewards, 0, n);
        std::ptr::write_bytes(self.terminals, 0, n);
        self.timestep += 1;

        if self.timestep == TRAJECTORY_LENGTH as i32 {
            self.add_log();
            self.c_reset();
            return;
        }

        for i in 0..self.expert_static_car_count {
            let idx = self.expert_static_car_indices[i] as usize;
            if self.entities[idx].x == -10000.0 {
                continue;
            }
            self.move_expert(idx);
        }

        for i in 0..n {
            self.logs[i].score = 0.0;
            self.logs[i].episode_length += 1.0;
            let idx = self.active_agent_indices[i] as usize;
            self.entities[idx].collision_state = 0;
            self.move_dynamics(i, idx);
        }

        for i in 0..n {
            let idx = self.active_agent_indices[i] as usize;
            self.entities[idx].collision_state = 0;
            self.collision_check(idx);
            let cs = self.entities[idx].collision_state;

            if cs > 0 {
                if cs == VEHICLE_COLLISION && self.entities[idx].respawn_timestep == -1 {
                    // Matches C: inner if (respawn_timestep != -1) is always false here
                    *self.rewards.add(i) = self.reward_vehicle_collision;
                    self.logs[i].episode_return += self.reward_vehicle_collision;
                    self.logs[i].clean_collision_rate = 1.0;
                    self.logs[i].collision_rate = 1.0;
                } else if cs == OFFROAD {
                    *self.rewards.add(i) = self.reward_offroad_collision;
                    self.logs[i].offroad_rate = 1.0;
                    self.logs[i].episode_return += self.reward_offroad_collision;
                }
                if self.entities[idx].reached_goal_this_episode == 0 {
                    self.entities[idx].collided_before_goal = 1;
                }
            }

            let dist = distance_2d(
                self.entities[idx].x,
                self.entities[idx].y,
                self.entities[idx].goal_position_x,
                self.entities[idx].goal_position_y,
            );
            if dist < 2.0 {
                if self.entities[idx].respawn_timestep != -1 {
                    *self.rewards.add(i) += self.reward_goal_post_respawn;
                    self.logs[i].episode_return += self.reward_goal_post_respawn;
                } else {
                    *self.rewards.add(i) += 1.0;
                    self.logs[i].episode_return += 1.0;
                }
                self.entities[idx].reached_goal = 1;
                self.entities[idx].reached_goal_this_episode = 1;
            }
        }

        for i in 0..n {
            let idx = self.active_agent_indices[i] as usize;
            if self.entities[idx].reached_goal != 0 {
                self.respawn_agent(idx);
            }
        }
        self.compute_observations();
    }

    pub fn c_close(&mut self) {
        self.entities.clear();
        self.active_agent_indices.clear();
        self.logs.clear();
        self.grid_cells.clear();
        self.neighbor_offsets.clear();
        self.neighbor_cache_entities.clear();
        self.neighbor_cache_indices.clear();
        self.static_car_indices.clear();
        self.expert_static_car_indices.clear();
    }
}
