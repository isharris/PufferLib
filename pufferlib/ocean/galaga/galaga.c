/* Pure C Raylib demo — same simulator as the Python binding (galaga.h).
 * Local:  bash scripts/build_ocean.sh galaga local
 *         bash scripts/build_ocean.sh galaga fast
 * Web:    bash scripts/build_ocean.sh galaga web
 *         → build_web/galaga/game.html (needs Emscripten + raylib-5.5_webassembly
 *         in the repo root; see scripts/build_ocean.sh). Same pattern as squared.
 * Controls: hold Left Shift — A/Left, D/Right, Space to shoot; else random actions.
 */
#include "galaga.h"

int main() {
    Galaga env = {.width = 1024, .height = 768, .frameskip = 1};
    env.observations = (float *)calloc(OBS_SIZE, sizeof(float));
    env.actions = (int *)calloc(1, sizeof(int));
    env.rewards = (float *)calloc(1, sizeof(float));
    env.terminals = (unsigned char *)calloc(1, sizeof(unsigned char));

    c_reset(&env);
    c_render(&env);
    while (!WindowShouldClose()) {
        if (IsKeyDown(KEY_LEFT_SHIFT)) {
            if (IsKeyDown(KEY_A) || IsKeyDown(KEY_LEFT)) {
                env.actions[0] = ACTION_LEFT;
            } else if (IsKeyDown(KEY_D) || IsKeyDown(KEY_RIGHT)) {
                env.actions[0] = ACTION_RIGHT;
            } else if (IsKeyDown(KEY_SPACE)) {
                env.actions[0] = ACTION_SHOOT;
            } else {
                env.actions[0] = ACTION_NOOP;
            }
        } else {
            env.actions[0] = rand() % 4;
        }
        c_step(&env);
        c_render(&env);
    }
    free(env.observations);
    free(env.actions);
    free(env.rewards);
    free(env.terminals);
    c_close(&env);
}
