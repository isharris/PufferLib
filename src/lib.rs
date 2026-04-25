//! `pufferlib.bindings` — single PyO3 extension that exposes Rust simulators
//! (under `pufferlib.bindings.<env_name>`) and shared kernels
//! (currently `pufferlib.bindings.puffernet`).
//!
//! Adding a new env: drop a folder under `src/envs/<name>/` and add one
//! `<name>::register(py, parent)?;` line to [`envs::register`].

use pyo3::prelude::*;

mod envs;
mod puffernet;

#[pymodule]
fn bindings(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    envs::register(py, m)?;
    puffernet::register(py, m)?;
    Ok(())
}
