/* Pure C Raylib demo — same simulator as the Python binding (galaga.h).
 * Local:  bash scripts/build_ocean.sh galaga local
 *         bash scripts/build_ocean.sh galaga fast
 * Web:    bash scripts/build_ocean.sh galaga web
 *         → build_web/galaga/game.html (needs Emscripten + raylib-5.5_webassembly
 *         in the repo root; see scripts/build_ocean.sh). Same pattern as squared.
 * Controls: hold Left Shift to take control — A/Left, D/Right, Space to shoot.
 *           Release Shift and the trained agent plays (if weights are present).
 */
#include "galaga.h"
#include "puffernet.h"

#define GALAGA_NUM_WEIGHTS 141445

int main() {
    Galaga env = {.width = 1024, .height = 768, .frameskip = 1};
    env.observations = (float *)calloc(OBS_SIZE, sizeof(float));
    env.actions = (int *)calloc(1, sizeof(int));
    env.rewards = (float *)calloc(1, sizeof(float));
    env.terminals = (unsigned char *)calloc(1, sizeof(unsigned char));

    Weights *weights = load_weights("resources/galaga/galaga_weights.bin",
                                    GALAGA_NUM_WEIGHTS);
    int logit_sizes[1] = {4};
    LinearLSTM *net = make_linearlstm(weights, 1, OBS_SIZE, logit_sizes, 1);

    c_reset(&env);
    c_render(&env);
    while (!WindowShouldClose()) {
        forward_linearlstm(net, env.observations, env.actions);

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
        }

        int was_terminal = env.terminals[0];
        c_step(&env);

        if (was_terminal) {
            memset(net->lstm->state_h, 0,
                   net->lstm->batch_size * net->lstm->hidden_size * sizeof(float));
            memset(net->lstm->state_c, 0,
                   net->lstm->batch_size * net->lstm->hidden_size * sizeof(float));
        }

        c_render(&env);
    }

    free_linearlstm(net);
    free(weights);
    free(env.observations);
    free(env.actions);
    free(env.rewards);
    free(env.terminals);
    c_close(&env);
}
