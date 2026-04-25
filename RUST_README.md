# Drive (Rust)

A Rust reimplementation of the PufferLib Drive environment, replacing the C core with a PyO3-based extension module. The Rust version is a drop-in replacement for training -- same observation space, action space, and reward semantics.

## Prerequisites

- Python 3.8+
- Rust toolchain (rustc, cargo)
- maturin
- PufferLib installed (`pip install -e .`)
- Drive map binaries under `resources/drive/binaries/`

### Installing Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Installing maturin

```bash
uv pip install maturin
```

## Build

From the repo root:

```bash
cd pufferlib/ocean/drive_rust
maturin develop --release
cp target/release/libbinding.dylib binding.cpython-311-darwin.so   # macOS
# cp target/release/libbinding.so binding.cpython-311-x86_64-linux-gnu.so  # Linux
cd ../../..
```

`maturin develop` builds the wheel and installs it to site-packages, but because PufferLib is itself installed editable, Python resolves `pufferlib.ocean.drive_rust` to the source tree -- so we also copy the freshly-built dylib into the package directory under the CPython-tagged name. After this, `from pufferlib.ocean.drive_rust import binding` works immediately.

For a debug build (slower, but with better panic messages):

```bash
cd pufferlib/ocean/drive_rust
maturin develop
cp target/debug/libbinding.dylib binding.cpython-311-darwin.so
cd ../../..
```

If you ever change `src/lib.rs` and the build appears to finish in <1s with no `Compiling pufferlib-drive-rust` line, cargo's incremental cache got confused; force a clean rebuild with:

```bash
cargo clean && rm -f binding.cpython-311-darwin.so && maturin develop --release && cp target/release/libbinding.dylib binding.cpython-311-darwin.so
```

## Run

### Training

```bash
puffer train puffer_drive_rust
```

On Mac (no CUDA):

```bash
puffer train puffer_drive_rust --train.device cpu
```

### Parity test

Runs the C and Rust environments side-by-side with identical deterministic actions and compares observations, rewards, and terminals at every step:

```bash
python -m pufferlib.ocean.drive_rust.test_parity
```

### Using in Python directly

```python
from pufferlib.ocean.drive_rust.drive_rust import DriveRust

env = DriveRust(num_agents=64, num_maps=10)
obs, info = env.reset()

for _ in range(91):
    actions = env.single_action_space.sample()  # per-agent
    # ... expand to all agents ...
    obs, rewards, terminals, truncations, info = env.step(actions)

env.close()
```

## Project structure

```
pufferlib/ocean/drive_rust/
├── Cargo.toml          Rust crate config (pyo3, numpy, rand)
├── pyproject.toml      maturin build configuration
├── __init__.py         Python package marker
├── drive_rust.py       DriveRust(PufferEnv) -- Python wrapper
├── test_parity.py      C-vs-Rust golden-trace parity test
└── src/
    ├── lib.rs          PyO3 module: shared, env_init, vectorize, vec_*
    ├── types.rs        Constants, Entity, Log, Drive structs
    ├── map.rs          Binary map loader with Result error handling
    └── sim.rs          Grid, collision, dynamics, observations, step/reset
```

## Key improvements

- **No segfaults on missing files.** The C version silently returns NULL from `fopen` and crashes downstream. The Rust version returns a `Result` that surfaces as a Python exception with the file path and IO error message.

- **No Raylib dependency for training.** The Rust crate builds without linking Raylib, so headless training servers don't need graphics libraries. The existing C `drive` binary still handles interactive rendering and policy demos.

- **Automatic memory management.** Entity trajectories, grid cells, and neighbor caches are owned `Vec`s that are freed on drop. No manual `free()` calls, no double-free or use-after-free risks.

- **Explicit buffer boundaries.** The unsafe raw-pointer interface with NumPy is confined to `env_init` (pointer setup), `compute_observations`, and `c_step` (reward/terminal writes). All internal simulation logic is safe Rust.

- **Parallel-safe registration.** Registered as `puffer_drive_rust` alongside the original `puffer_drive`, so both can coexist. No changes to the C build or existing training configs.

- **Portable binary format.** The Rust map loader reads the same `map_XXX.bin` files produced by `drive.py`'s `save_map_binary`. No format conversion needed.
