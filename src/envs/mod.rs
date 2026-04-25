//! Registry of Rust-backed environments. Each env lives in its own
//! submodule and exposes a `register(py, parent)` entry point that
//! attaches its PyO3 functions as `pufferlib.bindings.<env_name>`.

use pyo3::prelude::*;

pub mod drive;

pub fn register(py: Python<'_>, parent: &Bound<'_, PyModule>) -> PyResult<()> {
    drive::register(py, parent)?;
    Ok(())
}
