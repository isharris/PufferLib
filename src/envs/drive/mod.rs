//! Drive: PyO3 wrappers around the pure-Rust simulation in [`sim`].
//!
//! Exposed as `pufferlib.bindings.drive` with the same surface the previous
//! `binding.cpython-*.so` had: `shared`, `env_init`, `vectorize`,
//! `vec_reset`, `vec_step`, `vec_log`, `vec_render`, `vec_close`.

mod map;
mod sim;
mod types;

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use rand::Rng;

use self::types::{Drive, Log};

struct VecEnv {
    envs: Vec<*mut Drive>,
}

unsafe fn get_vec(handle: usize) -> &'static mut VecEnv {
    &mut *(handle as *mut VecEnv)
}

#[pyfunction]
#[pyo3(signature = (*, num_agents, num_maps, binaries_dir))]
fn shared(
    num_agents: i32,
    num_maps: i32,
    binaries_dir: &str,
) -> PyResult<(Vec<i64>, Vec<i64>, i64)> {
    let mut rng = rand::thread_rng();
    let mut total = 0i32;
    let mut env_count = 0usize;
    let max_envs = num_agents as usize;
    let mut offsets: Vec<i64> = Vec::new();
    let mut ids: Vec<i64> = Vec::new();

    while total < num_agents && env_count < max_envs {
        let map_id = rng.gen_range(0..num_maps);
        let path = format!("{}/map_{:03}.bin", binaries_dir, map_id);
        let mut env = Drive::new();
        env.map_name = path;
        let (entities, no, nr) = map::load_map_binary(&env.map_name).map_err(|e| {
            pyo3::exceptions::PyIOError::new_err(format!(
                "Failed to load map '{}': {}",
                env.map_name, e
            ))
        })?;
        env.entities = entities;
        env.num_objects = no;
        env.num_roads = nr;
        env.num_entities = no + nr;
        env.set_active_agents();

        ids.push(map_id as i64);
        offsets.push(total as i64);
        total += env.active_agent_count as i32;
        env_count += 1;
    }
    if total > num_agents {
        total = num_agents;
    }
    offsets.push(total as i64);
    Ok((offsets, ids, env_count as i64))
}

#[pyfunction]
#[pyo3(signature = (observations, actions, rewards, terminals, _truncations, _seed, **kwargs))]
fn env_init(
    observations: &Bound<'_, numpy::PyUntypedArray>,
    actions: &Bound<'_, numpy::PyUntypedArray>,
    rewards: &Bound<'_, numpy::PyUntypedArray>,
    terminals: &Bound<'_, numpy::PyUntypedArray>,
    _truncations: &Bound<'_, numpy::PyUntypedArray>,
    _seed: i32,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<usize> {
    use numpy::PyUntypedArrayMethods;

    let kw = kwargs.ok_or_else(|| {
        pyo3::exceptions::PyTypeError::new_err("env_init requires keyword arguments")
    })?;

    let get_f = |key: &str| -> PyResult<f32> {
        kw.get_item(key)?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_string()))?
            .extract::<f64>()
            .map(|v| v as f32)
    };
    let get_i = |key: &str| -> PyResult<i32> {
        kw.get_item(key)?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_string()))?
            .extract::<i32>()
    };
    let get_s = |key: &str| -> PyResult<String> {
        kw.get_item(key)?
            .ok_or_else(|| pyo3::exceptions::PyKeyError::new_err(key.to_string()))?
            .extract::<String>()
    };

    let mut env = Box::new(Drive::new());

    unsafe {
        env.observations = (*observations.as_array_ptr()).data as *mut f32;
        env.actions = (*actions.as_array_ptr()).data as *mut i32;
        env.rewards = (*rewards.as_array_ptr()).data as *mut f32;
        env.terminals = (*terminals.as_array_ptr()).data as *mut u8;
    }

    env.human_agent_idx = get_i("human_agent_idx")?;
    env.reward_vehicle_collision = get_f("reward_vehicle_collision")?;
    env.reward_offroad_collision = get_f("reward_offroad_collision")?;
    env.reward_goal_post_respawn = get_f("reward_goal_post_respawn")?;
    env.reward_vehicle_collision_post_respawn = get_f("reward_vehicle_collision_post_respawn")?;
    env.spawn_immunity_timer = get_i("spawn_immunity_timer")?;
    let map_id = get_i("map_id")?;
    let max_agents = get_i("max_agents")?;
    let binaries_dir = get_s("binaries_dir")?;

    env.map_name = format!("{}/map_{:03}.bin", binaries_dir, map_id);
    env.num_agents = max_agents;
    env.init()
        .map_err(pyo3::exceptions::PyRuntimeError::new_err)?;

    Ok(Box::into_raw(env) as usize)
}

#[pyfunction]
#[pyo3(signature = (*handles))]
fn vectorize(handles: &Bound<'_, PyTuple>) -> PyResult<usize> {
    let mut envs = Vec::with_capacity(handles.len());
    for i in 0..handles.len() {
        let h: usize = handles.get_item(i)?.extract()?;
        envs.push(h as *mut Drive);
    }
    Ok(Box::into_raw(Box::new(VecEnv { envs })) as usize)
}

#[pyfunction]
fn vec_reset(handle: usize, _seed: i32) -> PyResult<()> {
    unsafe {
        let v = get_vec(handle);
        for p in &v.envs {
            (**p).c_reset();
        }
    }
    Ok(())
}

#[pyfunction]
fn vec_step(handle: usize) -> PyResult<()> {
    unsafe {
        let v = get_vec(handle);
        for p in &v.envs {
            (**p).c_step();
        }
    }
    Ok(())
}

#[pyfunction]
fn vec_log(py: Python<'_>, handle: usize) -> PyResult<PyObject> {
    unsafe {
        let v = get_vec(handle);
        let mut agg = Log::default();
        for p in &v.envs {
            let env = &mut **p;
            agg.add_from(&env.log);
            env.log = Log::default();
        }
        let dict = PyDict::new_bound(py);
        if agg.n == 0.0 {
            return Ok(dict.into());
        }
        let n_raw = agg.n;
        agg.normalize();
        dict.set_item("perf", agg.perf)?;
        dict.set_item("score", agg.score)?;
        dict.set_item("episode_return", agg.episode_return)?;
        dict.set_item("episode_length", agg.episode_length)?;
        dict.set_item("offroad_rate", agg.offroad_rate)?;
        dict.set_item("collision_rate", agg.collision_rate)?;
        dict.set_item("dnf_rate", agg.dnf_rate)?;
        dict.set_item("completion_rate", agg.completion_rate)?;
        dict.set_item("clean_collision_rate", agg.clean_collision_rate)?;
        dict.set_item("n", n_raw)?;
        Ok(dict.into())
    }
}

#[pyfunction]
fn vec_render(_handle: usize, _env_id: usize) -> PyResult<()> {
    Ok(())
}

#[pyfunction]
fn vec_close(handle: usize) -> PyResult<()> {
    unsafe {
        let v = Box::from_raw(handle as *mut VecEnv);
        for p in v.envs {
            let mut env = Box::from_raw(p);
            env.c_close();
        }
    }
    Ok(())
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    eprintln!("======== RUST DRIVE LOADED (pufferlib.bindings.drive) ========");
    let m = PyModule::new_bound(py, "drive")?;
    m.add_function(wrap_pyfunction!(shared, &m)?)?;
    m.add_function(wrap_pyfunction!(env_init, &m)?)?;
    m.add_function(wrap_pyfunction!(vectorize, &m)?)?;
    m.add_function(wrap_pyfunction!(vec_reset, &m)?)?;
    m.add_function(wrap_pyfunction!(vec_step, &m)?)?;
    m.add_function(wrap_pyfunction!(vec_log, &m)?)?;
    m.add_function(wrap_pyfunction!(vec_render, &m)?)?;
    m.add_function(wrap_pyfunction!(vec_close, &m)?)?;

    parent.add_submodule(&m)?;
    // Make `from pufferlib.bindings import drive` and
    // `import pufferlib.bindings.drive` both work.
    let modules = py.import_bound("sys")?.getattr("modules")?;
    modules.set_item("pufferlib.bindings.drive", &m)?;
    Ok(())
}
