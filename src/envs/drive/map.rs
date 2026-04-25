use std::fs::File;
use std::io::{self, BufReader, Read};

use super::types::Entity;

fn read_i32(r: &mut impl Read) -> io::Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_f32(r: &mut impl Read) -> io::Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_f32_vec(r: &mut impl Read, n: usize) -> io::Result<Vec<f32>> {
    let mut v = vec![0.0f32; n];
    let buf =
        unsafe { std::slice::from_raw_parts_mut(v.as_mut_ptr() as *mut u8, n * 4) };
    r.read_exact(buf)?;
    Ok(v)
}

fn read_i32_vec(r: &mut impl Read, n: usize) -> io::Result<Vec<i32>> {
    let mut v = vec![0i32; n];
    let buf =
        unsafe { std::slice::from_raw_parts_mut(v.as_mut_ptr() as *mut u8, n * 4) };
    r.read_exact(buf)?;
    Ok(v)
}

/// Load map binary file, returning (entities, num_objects, num_roads).
pub fn load_map_binary(filename: &str) -> io::Result<(Vec<Entity>, usize, usize)> {
    let file = File::open(filename)?;
    let mut r = BufReader::new(file);

    let num_objects = read_i32(&mut r)? as usize;
    let num_roads = read_i32(&mut r)? as usize;
    let num_entities = num_objects + num_roads;
    let mut entities = Vec::with_capacity(num_entities);

    for _ in 0..num_entities {
        let entity_type = read_i32(&mut r)?;
        let array_size = read_i32(&mut r)? as usize;

        let traj_x = read_f32_vec(&mut r, array_size)?;
        let traj_y = read_f32_vec(&mut r, array_size)?;
        let traj_z = read_f32_vec(&mut r, array_size)?;

        let (traj_vx, traj_vy, traj_vz, traj_heading, traj_valid) =
            if (1..=3).contains(&entity_type) {
                (
                    read_f32_vec(&mut r, array_size)?,
                    read_f32_vec(&mut r, array_size)?,
                    read_f32_vec(&mut r, array_size)?,
                    read_f32_vec(&mut r, array_size)?,
                    read_i32_vec(&mut r, array_size)?,
                )
            } else {
                (vec![], vec![], vec![], vec![], vec![])
            };

        let width = read_f32(&mut r)?;
        let length = read_f32(&mut r)?;
        let height = read_f32(&mut r)?;
        let goal_position_x = read_f32(&mut r)?;
        let goal_position_y = read_f32(&mut r)?;
        let goal_position_z = read_f32(&mut r)?;
        let mark_as_expert = read_i32(&mut r)?;

        entities.push(Entity {
            entity_type,
            array_size,
            traj_x, traj_y, traj_z,
            traj_vx, traj_vy, traj_vz,
            traj_heading, traj_valid,
            width, length, height,
            goal_position_x, goal_position_y, goal_position_z,
            mark_as_expert,
            ..Default::default()
        });
    }

    Ok((entities, num_objects, num_roads))
}
