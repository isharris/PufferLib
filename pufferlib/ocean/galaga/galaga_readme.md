# Galaga — web (WASM) build and run

The browser build is the **pure C Raylib** demo in `galaga.c` (same logic as the Python binding), compiled with **Emscripten** using `scripts/build_ocean.sh`. This matches how lightweight Ocean envs are built for the web elsewhere in PufferLib.

## Prerequisites

1. **Emscripten** — The `emcc` compiler must be on your `PATH`. Install the [Emscripten SDK](https://emscripten.org/docs/getting_started/downloads.html), then **activate** it in every shell where you build (the installer does not add `emcc` globally by default), for example:

   ```bash
   source "$HOME/emsdk/emsdk_env.sh"   # path may differ where you cloned emsdk
   ```

   Run `command -v emcc` to confirm before `build_ocean.sh … web`.

2. **Raylib WebAssembly** — From the **repository root** (parent of `scripts/`), you need a directory named **`raylib-5.5_webassembly`** with the usual Raylib Emscripten layout (`include/`, `lib/libraylib.a`, etc.). Obtain or build it from the [Raylib](https://www.raylib.com/) Emscripten instructions; the layout must match what `scripts/build_ocean.sh` expects for `web` mode.

3. **Resource paths** — The build preloads `pufferlib/resources/galaga` and `pufferlib/resources/shared`. Both must exist (they are in this repo; `galaga` may only contain a placeholder until you add assets).

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

- **Hold Left Shift** — human control (otherwise actions are random).
- With shift held: **A** or **Left** — move left; **D** or **Right** — move right; **Space** — shoot.

## Native build (optional)

For a desktop binary while iterating on C:

```bash
bash scripts/build_ocean.sh galaga local   # debug
bash scripts/build_ocean.sh galaga fast    # optimized
```

Requires **`raylib-5.5_macos`** or **`raylib-5.5_linux_amd64`** at the repo root, matching `build_ocean.sh` and your OS.

## See also

- `scripts/build_ocean.sh` — full `emcc` flags and paths.
- `scripts/minshell.html` — HTML shell used for the WASM canvas.
- `galaga.c` / `galaga.h` — simulator and Raylib client.
