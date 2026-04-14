# Galaga — web (WASM) build, run, and training

The browser build is the **pure C Raylib** demo in `galaga.c` (same logic as the Python binding), compiled with **Emscripten** using `scripts/build_ocean.sh`. This matches how lightweight Ocean envs are built for the web elsewhere in PufferLib.

**Training** uses the **Python** `Galaga` env (`galaga.py` + native binding), driven by PufferLib’s `puffer` trainer — not the WASM build.

## Prerequisites

1. **Emscripten** — `emcc` available on your `PATH` (activate the Emscripten SDK environment before building).

2. **Raylib WebAssembly** — From the **repository root** (parent of `scripts/`), you need a directory named **`raylib-5.5_webassembly`** with the usual Raylib Emscripten layout (`include/`, `lib/libraylib.a`, etc.). Obtain or build it from the [Raylib](https://www.raylib.com/) Emscripten instructions; the layout must match what `scripts/build_ocean.sh` expects for `web` mode.

3. **Resource paths** — The build preloads `pufferlib/resources/galaga` and `pufferlib/resources/shared`. Both must exist. The `galaga` directory must contain `galaga_weights.bin` (see **Exporting weights** below).

## Build

From the **PufferLib repository root**:

```bash
bash scripts/build_ocean.sh galaga web
```

On success you get:

- `build_web/galaga/game.html`
- Associated `.js` / `.wasm` (and related) files Emscripten emits next to it.

## Run in the browser

Browsers often block `file://` for WASM or module loading. Prefer a **local HTTP server** in the **output** folder **`build_web/galaga/`** at the **repository root** — not in `pufferlib/ocean/galaga/` (that is only source code; `game.html` is not there, so you will get **404** if you start the server there).

From the repo root (the directory that contains `scripts/` and `build_web/`):

```bash
cd build_web/galaga
python3 -m http.server 8080
```

If you are unsure where you are, use an absolute path, for example:

```bash
cd /path/to/PufferLib/build_web/galaga
python3 -m http.server 8080
```

Then open [http://localhost:8080/game.html](http://localhost:8080/game.html).

Alternatively, copy the contents of `build_web/galaga/` to any static file host and open `game.html` from `https://…`.

## Controls (C demo)

- **Hold Left Shift** — human control.
- With shift held: **A** or **Left** — move left; **D** or **Right** — move right; **Space** — shoot.
- **Release Shift** — the trained agent plays (requires `galaga_weights.bin`; see below).

## Native build (optional)

For a desktop binary while iterating on C:

```bash
bash scripts/build_ocean.sh galaga local   # debug
bash scripts/build_ocean.sh galaga fast    # optimized
```

Requires **`raylib-5.5_macos`** or **`raylib-5.5_linux_amd64`** at the repo root, matching `build_ocean.sh` and your OS.

## Training the fighter (RL)

Training runs the **vectorized** `pufferlib.ocean.galaga.galaga.Galaga` environment through PufferLib’s PPO-style trainer (`pufferlib/pufferl.py`). You need a normal **Python install** of this repo with **PyTorch** and the **compiled native extensions** (including the Galaga binding). Optional extras: `pip install -e ".[galaga]"` from the repo root (see `pyproject.toml` `[galaga]`).

1. **Registered env name** — Use **`puffer_galaga`** (the `puffer_` prefix is required; see `env_creator` in `pufferlib/ocean/environment.py`).

2. **Start training** (from any directory, with the package on your `PYTHONPATH` or installed editable):

   ```bash
   puffer train puffer_galaga
   ```

   Equivalent:

   ```bash
   python -m pufferlib.pufferl train puffer_galaga
   ```

3. **Hyperparameters and env layout** — Defaults are merged from `pufferlib/config/default.ini` and **`pufferlib/config/ocean/galaga.ini`** (`[env]` for `num_envs`, resolution, `frameskip`; `[train]` for timesteps, learning rate, PPO-style settings).

4. **Overrides** — Any config key can be passed as a CLI flag (dotted names, underscores as hyphens). Examples:

   ```bash
   puffer train puffer_galaga --train.total-timesteps 10_000_000
   puffer train puffer_galaga --env.num-envs 512
   ```

   List everything for this env:

   ```bash
   puffer train puffer_galaga --help
   ```

5. **Checkpoints** — Training saves under `experiments/` (see `pufferlib/pufferl.py`). For rollouts with a saved policy, use **`puffer eval puffer_galaga`** and the `--load-model-path` / `--load-id` options (`puffer eval puffer_galaga --help`).

6. **Mac / Apple Silicon** — Pass `--train.device cpu` (or `mps` if your PyTorch build supports it) since there is no CUDA on Mac:

   ```bash
   puffer train puffer_galaga --train.device cpu
   ```

Full trainer options and multi-GPU patterns are documented on [puffer.ai](https://puffer.ai/).

## Exporting weights for the C / WASM demo

After training (or with an existing checkpoint), export the policy to a flat float32 binary that the C demo can load:

```bash
puffer export puffer_galaga --load-model-path experiments/<checkpoint>.pt --train.device cpu
```

This writes `puffer_galaga_weights.bin` in the current directory (141,445 floats / 565,780 bytes for the default `Default` + `LSTMWrapper` architecture with hidden_size=128, obs=67, actions=4).

Copy the file into the Emscripten preload tree:

```bash
cp puffer_galaga_weights.bin pufferlib/resources/galaga/galaga_weights.bin
```

Then rebuild the web (or native) binary so the new weights are bundled:

```bash
bash scripts/build_ocean.sh galaga web
```

The C demo (`galaga.c`) loads the weights at startup with Puffernet's `load_weights` / `make_linearlstm` and runs `forward_linearlstm` each frame when the user is not holding Shift. LSTM hidden state is zeroed on episode boundaries.

## See also

- `scripts/build_ocean.sh` — full `emcc` flags and paths.
- `scripts/minshell.html` — HTML shell used for the WASM canvas.
- `galaga.c` / `galaga.h` — simulator and Raylib client (C demo + WASM).
- `galaga.py` — Python `PufferEnv` used for training.
- `pufferlib/config/ocean/galaga.ini` — default train/env config for `puffer_galaga`.
- `pufferlib/extensions/puffernet.h` — C neural net inference used by the demo.
- `pufferlib/resources/galaga/` — where `galaga_weights.bin` goes for emcc preload.
