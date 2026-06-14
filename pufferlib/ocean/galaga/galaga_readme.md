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

6. **Mac / Apple Silicon** — `galaga.ini` sets **`device = mps`** and **14 workers × 2048 C envs = 28,672 agents** (one process per physical core on a 14-core M4 Pro, no oversubscription). Keep **`env.num_envs` high** (2048+) per worker — lowering the inner C loop count is what cut SPS before.

   ```bash
   puffer train puffer_galaga
   ```

   RunPod / CUDA (8 workers × 1024, no overwork):

   ```bash
   puffer train puffer_galaga --train.device cuda --vec.num-envs 8 --vec.num-workers 8 --env.num-envs 1024 --vec.overwork False
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

## How vectorized envs, workers, and training batch sizes fit together

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        galaga.ini config                                 │
│                                                                         │
│  [vec]                     [env]              [train]                    │
│  num_envs = 14             num_envs = 2048    batch_size = auto          │
│  num_workers = 14                             bptt_horizon = 64          │
│  overwork = False                             minibatch_size = 8192      │
│                                               update_epochs = 1          │
└─────────────────────────────────────────────────────────────────────────┘

                        ┌─────────────────┐
                        │  PuffeRL Trainer │
                        │    (pufferl.py)  │
                        └────────┬────────┘
                                 │ creates
                                 ▼
              ┌──────────────────────────────────────┐
              │     Multiprocessing Vectorizer        │
              │         (vector.py)                   │
              │                                      │
              │  vec.num_envs = 14 "logical slots"   │
              │  vec.num_workers = 14 OS processes    │
              │  envs_per_worker = num_envs/workers   │
              │                  = 14/14 = 1          │
              └──────────────────┬───────────────────┘
                                 │ spawns
                                 ▼
   ┌─────────────────────────────────────────────────────────────┐
   │                  14 Worker Processes                          │
   │                                                              │
   │  ┌──────────┐ ┌──────────┐ ┌──────────┐     ┌──────────┐   │
   │  │ Worker 0 │ │ Worker 1 │ │ Worker 2 │ ... │Worker 13 │   │
   │  └────┬─────┘ └────┬─────┘ └────┬─────┘     └────┬─────┘   │
   │       │             │             │                │          │
   └───────┼─────────────┼─────────────┼────────────────┼──────────┘
           │             │             │                │
           ▼             ▼             ▼                ▼
   ┌──────────────────────────────────────────────────────────────┐
   │        Each worker runs env.num_envs = 2048 C sims           │
   │                                                              │
   │  Worker 0:                                                   │
   │  ┌────┬────┬────┬────┬─────────────────────────────┬────┐   │
   │  │sim │sim │sim │sim │  ... sequential C loop ...  │sim │   │
   │  │ 0  │ 1  │ 2  │ 3  │       (vec_step)           │2047│   │
   │  └────┴────┴────┴────┴─────────────────────────────┴────┘   │
   │                                                              │
   │  Each sim = 1 agent (single-player Galaga)                   │
   │  2048 sims stepped sequentially in a tight C for-loop        │
   └──────────────────────────────────────────────────────────────┘


═══════════════════════════════════════════════════════════════════════════
                         SIZING MATH
═══════════════════════════════════════════════════════════════════════════

  total_agents = vec.num_envs × env.num_envs
               = 14 × 2048
               = 28,672

  batch_size   = total_agents × bptt_horizon      (when "auto")
               = 28,672 × 64
               = 1,835,008 timesteps per rollout

  segments     = batch_size / bptt_horizon
               = 1,835,008 / 64
               = 28,672                           (= total_agents)

  minibatches per update_epoch = batch_size / minibatch_size
                               = 1,835,008 / 8,192
                               = 224 gradient steps per epoch

═══════════════════════════════════════════════════════════════════════════
                    DATA FLOW (one training iteration)
═══════════════════════════════════════════════════════════════════════════

    ┌──────────────────────────────────────────────────────────────────┐
    │  1. ROLLOUT COLLECTION                                           │
    │                                                                  │
    │  For each of bptt_horizon=64 timesteps:                          │
    │    • All 28,672 agents step in parallel (across 14 workers)      │
    │    • Policy forward pass on batch of observations                │
    │    • Store: obs, actions, logprobs, values, rewards, terminals   │
    │                                                                  │
    │  Result: experience buffer of shape [segments, horizon]          │
    │          = [28,672, 64] = 1,835,008 transitions                  │
    └──────────────────────────────┬───────────────────────────────────┘
                                   │
                                   ▼
    ┌──────────────────────────────────────────────────────────────────┐
    │  2. ADVANTAGE COMPUTATION                                        │
    │                                                                  │
    │  GAE / V-trace over each segment (length=64 trajectory chunks)   │
    │  On-device (MPS/CUDA) via compute_puff_advantage                 │
    └──────────────────────────────┬───────────────────────────────────┘
                                   │
                                   ▼
    ┌──────────────────────────────────────────────────────────────────┐
    │  3. PPO UPDATE                                                   │
    │                                                                  │
    │  For update_epochs=1 epoch:                                      │
    │    For each of 224 minibatches (size 8,192 each):                │
    │      • Sample 128 segments × 64 horizon = 8,192 transitions      │
    │      • Forward pass → new logprobs, values, entropy              │
    │      • Compute PPO clipped loss + value loss                     │
    │      • Backward + clip grad norm + optimizer step                │
    │                                                                  │
    │  Total gradient steps per iteration = 224                        │
    └──────────────────────────────────────────────────────────────────┘


═══════════════════════════════════════════════════════════════════════════
                    DIVISIBILITY CONSTRAINTS
═══════════════════════════════════════════════════════════════════════════

  • total_agents must be ≤ segments (always true when batch_size=auto)
  • minibatch_size must divide batch_size evenly
  • minibatch_size must be divisible by bptt_horizon
      → minibatch_size / bptt_horizon = "minibatch_segments"
      → 8192 / 64 = 128 segments per minibatch ✓
  • vec.num_workers ≤ physical CPU cores (unless overwork=True)
  • vec.num_envs must be divisible by vec.num_workers


═══════════════════════════════════════════════════════════════════════════
                    PERFORMANCE KNOBS
═══════════════════════════════════════════════════════════════════════════

  ┌─────────────────────┬──────────────────────────────────────────────┐
  │ Setting             │ Effect                                        │
  ├─────────────────────┼──────────────────────────────────────────────┤
  │ env.num_envs ↑      │ More sims per worker → longer C loop per     │
  │                     │ step, but amortizes process overhead.         │
  │                     │ KEY driver of SPS for native C envs.          │
  ├─────────────────────┼──────────────────────────────────────────────┤
  │ vec.num_workers ↑   │ More OS processes → CPU parallelism.          │
  │                     │ Diminishing returns past physical cores.      │
  ├─────────────────────┼──────────────────────────────────────────────┤
  │ vec.num_envs ↑      │ More logical slots → more total_agents →      │
  │                     │ bigger batch → more GPU work per iteration.   │
  ├─────────────────────┼──────────────────────────────────────────────┤
  │ bptt_horizon ↑      │ Longer unroll for LSTM credit assignment.     │
  │                     │ Bigger batch if auto. More BPTT memory.       │
  ├─────────────────────┼──────────────────────────────────────────────┤
  │ minibatch_size ↑    │ Fewer gradient steps per epoch, each with     │
  │                     │ more data. Affects learning dynamics.          │
  ├─────────────────────┼──────────────────────────────────────────────┤
  │ update_epochs ↑     │ More passes over the same rollout data.       │
  │                     │ Can improve sample efficiency but adds cost.  │
  └─────────────────────┴──────────────────────────────────────────────┘


═══════════════════════════════════════════════════════════════════════════
                    EXAMPLE CONFIGS
═══════════════════════════════════════════════════════════════════════════

  Mac M4 Pro (14 cores, 48GB):
    vec.num_envs=14, vec.num_workers=14, env.num_envs=2048, overwork=False
    → 28,672 agents, batch=1,835,008, 224 minibatches

  RunPod / CUDA (8-core):
    vec.num_envs=8, vec.num_workers=8, env.num_envs=1024
    → 8,192 agents, batch=524,288, 64 minibatches

  Quick debug (single process):
    vec.num_envs=1, vec.num_workers=1, env.num_envs=2048
    → 2,048 agents, batch=131,072, 16 minibatches
```

## See also

- `scripts/build_ocean.sh` — full `emcc` flags and paths.
- `scripts/minshell.html` — HTML shell used for the WASM canvas.
- `galaga.c` / `galaga.h` — simulator and Raylib client (C demo + WASM).
- `galaga.py` — Python `PufferEnv` used for training.
- `pufferlib/config/ocean/galaga.ini` — default train/env config for `puffer_galaga`.
- `pufferlib/extensions/puffernet.h` — C neural net inference used by the demo.
- `pufferlib/resources/galaga/` — where `galaga_weights.bin` goes for emcc preload.
