Place `galaga_weights.bin` here after running:

    puffer export puffer_galaga --load-model-path <checkpoint.pt> --train.device mps

The WASM build (`scripts/build_ocean.sh galaga web`) preloads this directory
into the Emscripten virtual FS at `resources/galaga/`, which is the path
`galaga.c` uses in `load_weights("resources/galaga/galaga_weights.bin", ...)`.

Expected size: 141445 floats = 565780 bytes (Default + LSTMWrapper, hidden=128,
obs=67, actions=4).
