# PufferLib (Rust)

PufferLib's native code is now a single Rust crate at the repo root, built and
installed by [maturin](https://www.maturin.rs/). The crate is exposed to Python
as the package `pufferlib.bindings`, with one submodule per environment plus
shared kernels (e.g. the V-trace advantage kernel).

```python
from pufferlib.bindings import drive       # env: pufferlib/ocean/drive
from pufferlib.bindings import puffernet   # V-trace advantage kernel
```

## Prerequisites

- Python 3.9+
- Rust toolchain (`rustc`, `cargo`)
- Optional: `nvcc` and `CUDA_HOME` set to enable the GPU V-trace kernel
- Drive map binaries under `pufferlib/resources/drive/binaries/`

Installing Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Build

A regular editable install from the repo root builds the Rust extension in
place:

```bash
uv pip install -e .
# or
pip install -e .
```

This produces `pufferlib/bindings.cpython-<tag>.so` so
`from pufferlib.bindings import drive, puffernet` works immediately.

The CUDA V-trace kernel is compiled automatically when `CUDA_HOME` is set at
build time. Otherwise the CPU implementation is used (and `puffernet.has_cuda()`
returns `False`).

To rebuild only the Rust extension (after editing `src/*.rs` or
`cuda/*.cu`):

```bash
maturin develop --release
```

For a faster, debug build:

```bash
maturin develop
```

## Run

```bash
puffer train puffer_drive
# CPU-only host:
puffer train puffer_drive --train.device cpu
```

## Project structure

```
Cargo.toml             single PyO3 crate (lib name = "bindings")
build.rs               compiles cuda/ when CUDA_HOME is set
cuda/
  puff_advantage.cu    V-trace CUDA kernel
src/
  lib.rs               #[pymodule] bindings: registers env + kernel submodules
  envs/
    mod.rs             registers each env
    drive/
      mod.rs           PyO3 wrappers: shared, env_init, vec_*
      types.rs         Drive / Entity / Log
      map.rs           binary map loader
      sim.rs           grid / collision / dynamics / observations
  puffernet/
    mod.rs             PyO3 wrapper for compute_puff_advantage
    cpu.rs             safe Rust CPU implementation
pufferlib/
  bindings.<tag>.so    built by maturin, imported as pufferlib.bindings
  ocean/drive/drive.py thin PufferEnv wrapper around bindings.drive
  pufferl.py           uses pufferlib.bindings.puffernet for advantages
```

## Adding a new env

1. Create `src/envs/<name>/{mod.rs, types.rs, sim.rs, ...}` with the
   simulation code and PyO3 wrappers, plus a `pub fn register(py, parent)`.
2. Add `pub mod <name>;` and one `<name>::register(py, parent)?;` line to
   `src/envs/mod.rs`.
3. Create `pufferlib/ocean/<name>/<name>.py` with a `PufferEnv` subclass
   that does `from pufferlib.bindings import <name> as binding`.
4. Register the env in `pufferlib/ocean/environment.py`'s `MAKE_FUNCTIONS`
   and add a config under `pufferlib/config/ocean/<name>.ini`.

## Notes on the migration

- The previous C `drive` env, the C++ V-trace kernel
  (`pufferlib/extensions/`), the per-env `setup.py` plumbing, and the
  nested `pyproject.toml` under `drive_rust/` have all been removed. The
  Rust simulator and the Rust V-trace kernel replace them.
- `pufferlib._C` is gone. `pufferl.py` now imports
  `from pufferlib.bindings import puffernet` and dispatches via raw
  `tensor.data_ptr()`s, so there's no PyTorch C++ extension to compile.
- `pufferlib.ocean.drive` is the canonical Drive env (no `_rust` suffix).
  There is only one Drive now, and it is Rust.
- Rendering (Raylib) has not yet been ported to Rust; this is a follow-up
  step.
