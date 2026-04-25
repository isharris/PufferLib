//! `pufferlib.bindings.puffernet` — V-trace advantage kernel.
//!
//! CPU implementation is a safe-ish Rust port of the original C++ kernel.
//! When the crate is built with `CUDA_HOME` set the CUDA kernel in
//! `cuda/puff_advantage.cu` is compiled and linked, and the `is_cuda` path
//! in [`compute_puff_advantage`] dispatches to it.

mod cpu;

use pyo3::prelude::*;

#[cfg(puffer_cuda)]
extern "C" {
    fn puffer_advantage_cuda_launch(
        values: *const f32,
        rewards: *const f32,
        dones: *const f32,
        importance: *const f32,
        advantages: *mut f32,
        gamma: f32,
        lambda: f32,
        rho_clip: f32,
        c_clip: f32,
        num_steps: i32,
        horizon: i32,
    ) -> i32;
}

#[allow(clippy::too_many_arguments)]
#[pyfunction]
#[pyo3(signature = (
    values_ptr, rewards_ptr, dones_ptr, importance_ptr, advantages_ptr,
    num_steps, horizon,
    gamma, lambda, rho_clip, c_clip,
    is_cuda,
))]
fn compute_puff_advantage(
    values_ptr: usize,
    rewards_ptr: usize,
    dones_ptr: usize,
    importance_ptr: usize,
    advantages_ptr: usize,
    num_steps: i32,
    horizon: i32,
    gamma: f32,
    lambda: f32,
    rho_clip: f32,
    c_clip: f32,
    is_cuda: bool,
) -> PyResult<()> {
    let values = values_ptr as *const f32;
    let rewards = rewards_ptr as *const f32;
    let dones = dones_ptr as *const f32;
    let importance = importance_ptr as *const f32;
    let advantages = advantages_ptr as *mut f32;

    if is_cuda {
        #[cfg(puffer_cuda)]
        unsafe {
            let err = puffer_advantage_cuda_launch(
                values, rewards, dones, importance, advantages,
                gamma, lambda, rho_clip, c_clip, num_steps, horizon,
            );
            if err != 0 {
                return Err(pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "CUDA error code {err}"
                )));
            }
            return Ok(());
        }
        #[cfg(not(puffer_cuda))]
        return Err(pyo3::exceptions::PyRuntimeError::new_err(
            "pufferlib.bindings was built without CUDA support \
             (set CUDA_HOME at build time and reinstall)",
        ));
    }

    unsafe {
        cpu::puff_advantage(
            values, rewards, dones, importance, advantages,
            gamma, lambda, rho_clip, c_clip,
            num_steps as usize, horizon as usize,
        );
    }
    Ok(())
}

#[pyfunction]
fn has_cuda() -> bool {
    cfg!(puffer_cuda)
}

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let m = PyModule::new_bound(py, "puffernet")?;
    m.add_function(wrap_pyfunction!(compute_puff_advantage, &m)?)?;
    m.add_function(wrap_pyfunction!(has_cuda, &m)?)?;

    parent.add_submodule(&m)?;
    let modules = py.import_bound("sys")?.getattr("modules")?;
    modules.set_item("pufferlib.bindings.puffernet", &m)?;
    Ok(())
}
