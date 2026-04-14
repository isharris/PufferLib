#pragma once

#include "raylib.h"
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_PLAYER_ROCKETS 2
#define MAX_ENEMIES 10
#define MAX_ENEMY_ROCKETS 8
#define NUM_BEZIER_PATHS 4
#define NUM_QUARTETS 3
#define STAR_LAYERS 3

#define OBS_SIZE 67

#define ACTION_NOOP 0
#define ACTION_LEFT 1
#define ACTION_RIGHT 2
#define ACTION_SHOOT 3

#define PLAYER_SPEED 5.0f
#define PLAYER_ROCKET_SPEED 15.0f
#define ENEMY_ROCKET_SPEED_Y 7.0f
#define BEZIER_ADVANCE 0.012f
#define ENEMY_SHOOT_Y_THRESH 400.0f
#define SPAWN_INTERVAL 14
#define SHOOT_INTERVAL 30
#define ENEMIES_PER_WAVE 10
#define SCORE_PER_KILL 120
#define PLAYER_HALF_W 24
#define PLAYER_HALF_H 22
#define ENEMY_HALF_W 24
#define ENEMY_HALF_H 20
#define ROCKET_HALF_W 6
#define ROCKET_HALF_H 7
#define SHOOT_COOLDOWN 8

#define MAX_TICK 3600

#define MAX_STARS 40

static int galaga_render_flag = 0;
static int galaga_game_over_timer = 0;
static int galaga_game_over_started = 0;

typedef struct {
    float perf;
    float score;
    float episode_return;
    float episode_length;
    float n;
} Log;

typedef struct {
    float x, y;
} Vec2;

typedef struct {
    Vec2 p0, p1, p2, p3;
} BezierQuartet;

typedef struct {
    BezierQuartet quartets[NUM_QUARTETS];
} BezierPath;

typedef struct {
    float x, y;
    int active;
} PlayerRocket;

typedef struct {
    float x, y;
    int active;
    int type;
    float bezier_t;
    int path_id;
    float prev_x, prev_y;
} Enemy;

typedef struct {
    float x, y;
    float vx, vy;
    int active;
} EnemyRocket;

typedef struct {
    float x, y, speed;
} Star;

typedef struct {
    Log log;
    float *observations;
    int *actions;
    float *rewards;
    unsigned char *terminals;

    int width;
    int height;
    int frameskip;

    float player_x;
    PlayerRocket player_rockets[MAX_PLAYER_ROCKETS];
    int last_shot_tick;

    Enemy enemies[MAX_ENEMIES];
    int enemies_spawned;
    int enemies_killed;
    int wave;

    EnemyRocket enemy_rockets[MAX_ENEMY_ROCKETS];
    int enemy_rocket_idx;

    int spawn_timer;
    int shoot_timer;
    int tick;
    int score;
    float episode_return;

    Star stars[STAR_LAYERS * MAX_STARS];
} Galaga;

static const BezierPath PATHS[NUM_BEZIER_PATHS] = {
    {{ /* Path 0 (collection 1) */
        {{513, -15}, {700, 151}, {888, 650}, {501, 648}},
        {{501, 648}, {114, 646}, {208, 488}, {235, 343}},
        {{235, 343}, {262, 198}, {326, -181}, {513, -15}}
    }},
    {{ /* Path 1 (collection 2) */
        {{513, -15}, {430,  11}, {204, 659}, {516, 654}},
        {{516, 654}, {828, 649}, {420, 388}, {525, 375}},
        {{525, 375}, {630, 362}, {596, -41}, {513, -15}}
    }},
    {{ /* Path 2 (collection 3) */
        {{513, -15}, {365,  16}, {663, 556}, {516, 654}},
        {{516, 654}, {269, 652}, {476, 535}, {528, 393}},
        {{528, 393}, {480, 251}, {461,  14}, {513, -15}}
    }},
    {{ /* Path 3 (collection 4) */
        {{513, -15}, {330,  11}, {204, 659}, {516, 654}},
        {{516, 654}, {528, 649}, {220, 388}, {525, 375}},
        {{525, 375}, {530, 362}, {396, -41}, {513, -15}}
    }}
};

static Vec2 bezier_eval(const BezierQuartet *q, float t) {
    float u = 1.0f - t;
    float uu = u * u;
    float uuu = uu * u;
    float tt = t * t;
    float ttt = tt * t;
    Vec2 out;
    out.x = uuu * q->p0.x + 3.0f * uu * t * q->p1.x + 3.0f * u * tt * q->p2.x + ttt * q->p3.x;
    out.y = uuu * q->p0.y + 3.0f * uu * t * q->p1.y + 3.0f * u * tt * q->p2.y + ttt * q->p3.y;
    return out;
}

static Vec2 bezier_path_eval(int path_id, float bezier_t) {
    int qi = (int)bezier_t;
    if (qi >= NUM_QUARTETS) qi = NUM_QUARTETS - 1;
    float local_t = bezier_t - (float)qi;
    if (local_t < 0.0f) local_t = 0.0f;
    if (local_t > 1.0f) local_t = 1.0f;
    return bezier_eval(&PATHS[path_id].quartets[qi], local_t);
}

static int aabb_overlap(float ax, float ay, float ahw, float ahh,
                        float bx, float by, float bhw, float bhh) {
    return fabsf(ax - bx) < (ahw + bhw) && fabsf(ay - by) < (ahh + bhh);
}

static void init_stars(Galaga *env) {
    float speeds[STAR_LAYERS] = {1.0f, 4.0f, 8.0f};
    for (int layer = 0; layer < STAR_LAYERS; layer++) {
        for (int i = 0; i < MAX_STARS; i++) {
            int idx = layer * MAX_STARS + i;
            env->stars[idx].x = (float)(rand() % env->width);
            env->stars[idx].y = (float)(rand() % env->height);
            env->stars[idx].speed = speeds[layer];
        }
    }
}

static void update_stars(Galaga *env) {
    for (int i = 0; i < STAR_LAYERS * MAX_STARS; i++) {
        env->stars[i].y += env->stars[i].speed;
        if (env->stars[i].y > env->height) {
            env->stars[i].x = (float)(rand() % env->width);
            env->stars[i].y = (float)(-(rand() % 20) - 5);
        }
    }
}

static void move_player_rockets(Galaga *env) {
    for (int i = 0; i < MAX_PLAYER_ROCKETS; i++) {
        if (!env->player_rockets[i].active) continue;
        env->player_rockets[i].y -= PLAYER_ROCKET_SPEED;
        if (env->player_rockets[i].y < -ROCKET_HALF_H) {
            env->player_rockets[i].active = 0;
        }
    }
}

static void move_enemies(Galaga *env) {
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (!env->enemies[i].active) continue;
        Enemy *e = &env->enemies[i];
        e->prev_x = e->x;
        e->prev_y = e->y;
        e->bezier_t += BEZIER_ADVANCE;
        if (e->bezier_t >= (float)NUM_QUARTETS) {
            e->active = 0;
            continue;
        }
        Vec2 pos = bezier_path_eval(e->path_id, e->bezier_t);
        e->x = pos.x * (float)env->width / 1024.0f;
        e->y = pos.y * (float)env->height / 768.0f;
    }
}

static void move_enemy_rockets(Galaga *env) {
    for (int i = 0; i < MAX_ENEMY_ROCKETS; i++) {
        if (!env->enemy_rockets[i].active) continue;
        EnemyRocket *r = &env->enemy_rockets[i];
        r->x += r->vx;
        r->y += r->vy;
        if (r->y > env->height + ROCKET_HALF_H || r->y < -ROCKET_HALF_H ||
            r->x < -ROCKET_HALF_W || r->x > env->width + ROCKET_HALF_W) {
            r->active = 0;
        }
    }
}

static void try_spawn_enemies(Galaga *env) {
    if (env->enemies_spawned >= ENEMIES_PER_WAVE) return;
    env->spawn_timer++;
    if (env->spawn_timer < SPAWN_INTERVAL) return;
    env->spawn_timer = 0;

    int wave_mod = env->wave % 2;
    int path_a = wave_mod * 2;
    int path_b = wave_mod * 2 + 1;

    for (int pass = 0; pass < 2 && env->enemies_spawned < ENEMIES_PER_WAVE; pass++) {
        int pid = (pass == 0) ? path_a : path_b;
        for (int i = 0; i < MAX_ENEMIES; i++) {
            if (!env->enemies[i].active) {
                Enemy *e = &env->enemies[i];
                e->active = 1;
                e->type = env->wave % 3;
                e->bezier_t = 0.0f;
                e->path_id = pid;
                Vec2 start = bezier_path_eval(pid, 0.0f);
                e->x = start.x * (float)env->width / 1024.0f;
                e->y = start.y * (float)env->height / 768.0f;
                e->prev_x = e->x;
                e->prev_y = e->y;
                env->enemies_spawned++;
                break;
            }
        }
    }
}

static void try_enemy_shoot(Galaga *env) {
    env->shoot_timer++;
    if (env->shoot_timer < SHOOT_INTERVAL) return;
    env->shoot_timer = 0;

    int candidates[MAX_ENEMIES];
    int n = 0;
    float shoot_thresh = ENEMY_SHOOT_Y_THRESH * (float)env->height / 768.0f;
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (env->enemies[i].active && env->enemies[i].y < shoot_thresh) {
            candidates[n++] = i;
        }
    }
    if (n == 0) return;

    int ci = candidates[rand() % n];
    Enemy *e = &env->enemies[ci];
    float dy = ((float)env->height - 40.0f) - e->y;
    if (dy <= 0) return;
    float dx = env->player_x - e->x;
    float steps = dy / ENEMY_ROCKET_SPEED_Y;
    float vx = dx / steps;

    int slot = env->enemy_rocket_idx;
    env->enemy_rocket_idx = (env->enemy_rocket_idx + 1) % MAX_ENEMY_ROCKETS;
    EnemyRocket *r = &env->enemy_rockets[slot];
    r->active = 1;
    r->x = e->x;
    r->y = e->y;
    r->vx = vx;
    r->vy = ENEMY_ROCKET_SPEED_Y;
}

static void check_player_rocket_enemy_collision(Galaga *env) {
    for (int ri = 0; ri < MAX_PLAYER_ROCKETS; ri++) {
        if (!env->player_rockets[ri].active) continue;
        for (int ei = 0; ei < MAX_ENEMIES; ei++) {
            if (!env->enemies[ei].active) continue;
            if (aabb_overlap(env->player_rockets[ri].x, env->player_rockets[ri].y,
                             ROCKET_HALF_W, ROCKET_HALF_H,
                             env->enemies[ei].x, env->enemies[ei].y,
                             ENEMY_HALF_W, ENEMY_HALF_H)) {
                env->player_rockets[ri].active = 0;
                env->enemies[ei].active = 0;
                env->enemies_killed++;
                env->score += SCORE_PER_KILL;
                env->rewards[0] += 1.0f;
                break;
            }
        }
    }
}

static void check_enemy_rocket_player_collision(Galaga *env) {
    float py = (float)env->height - 40.0f;
    for (int i = 0; i < MAX_ENEMY_ROCKETS; i++) {
        if (!env->enemy_rockets[i].active) continue;
        if (aabb_overlap(env->enemy_rockets[i].x, env->enemy_rockets[i].y,
                         ROCKET_HALF_W, ROCKET_HALF_H,
                         env->player_x, py,
                         PLAYER_HALF_W, PLAYER_HALF_H)) {
            env->terminals[0] = 1;
            env->rewards[0] -= 5.0f;
            return;
        }
    }
}

static void check_enemy_player_collision(Galaga *env) {
    float py = (float)env->height - 40.0f;
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (!env->enemies[i].active) continue;
        if (aabb_overlap(env->enemies[i].x, env->enemies[i].y,
                         ENEMY_HALF_W, ENEMY_HALF_H,
                         env->player_x, py,
                         PLAYER_HALF_W, PLAYER_HALF_H)) {
            env->terminals[0] = 1;
            env->rewards[0] -= 5.0f;
            return;
        }
    }
}

static void check_wave_complete(Galaga *env) {
    if (env->enemies_spawned < ENEMIES_PER_WAVE) return;
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (env->enemies[i].active) return;
    }
    env->rewards[0] += 0.1f;
    env->wave++;
    env->enemies_spawned = 0;
    env->enemies_killed = 0;
    env->spawn_timer = 0;
}

static void compute_observations(Galaga *env) {
    int idx = 0;
    float inv_w = 1.0f / (float)env->width;
    float inv_h = 1.0f / (float)env->height;

    env->observations[idx++] = env->player_x * inv_w;

    for (int i = 0; i < MAX_PLAYER_ROCKETS; i++) {
        if (env->player_rockets[i].active) {
            env->observations[idx++] = env->player_rockets[i].x * inv_w;
            env->observations[idx++] = env->player_rockets[i].y * inv_h;
        } else {
            env->observations[idx++] = 0.0f;
            env->observations[idx++] = 0.0f;
        }
    }

    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (env->enemies[i].active) {
            env->observations[idx++] = env->enemies[i].x * inv_w;
            env->observations[idx++] = env->enemies[i].y * inv_h;
            env->observations[idx++] = 1.0f;
        } else {
            env->observations[idx++] = 0.0f;
            env->observations[idx++] = 0.0f;
            env->observations[idx++] = 0.0f;
        }
    }

    for (int i = 0; i < MAX_ENEMY_ROCKETS; i++) {
        if (env->enemy_rockets[i].active) {
            env->observations[idx++] = env->enemy_rockets[i].x * inv_w;
            env->observations[idx++] = env->enemy_rockets[i].y * inv_h;
            env->observations[idx++] = env->enemy_rockets[i].vx * inv_w;
            env->observations[idx++] = env->enemy_rockets[i].vy * inv_h;
        } else {
            env->observations[idx++] = 0.0f;
            env->observations[idx++] = 0.0f;
            env->observations[idx++] = 0.0f;
            env->observations[idx++] = 0.0f;
        }
    }
}

static void add_log(Galaga *env) {
    env->log.perf += env->score / 1000.0f;
    env->log.score += env->score;
    env->log.episode_length += env->tick;
    env->log.episode_return += env->episode_return;
    env->log.n++;
}

void c_reset(Galaga *env) {
    env->player_x = env->width / 2.0f;
    memset(env->player_rockets, 0, sizeof(env->player_rockets));
    env->last_shot_tick = -SHOOT_COOLDOWN;
    memset(env->enemies, 0, sizeof(env->enemies));
    env->enemies_spawned = 0;
    env->enemies_killed = 0;
    env->wave = 0;
    memset(env->enemy_rockets, 0, sizeof(env->enemy_rockets));
    env->enemy_rocket_idx = 0;
    env->spawn_timer = 0;
    env->shoot_timer = 0;
    env->tick = 0;
    env->score = 0;
    env->episode_return = 0.0f;
    init_stars(env);
    compute_observations(env);
}

static void step_frame(Galaga *env, int action) {
    if (action == ACTION_LEFT) {
        env->player_x -= PLAYER_SPEED;
        if (env->player_x < PLAYER_HALF_W) env->player_x = PLAYER_HALF_W;
    }
    if (action == ACTION_RIGHT) {
        env->player_x += PLAYER_SPEED;
        if (env->player_x > env->width - PLAYER_HALF_W)
            env->player_x = (float)(env->width - PLAYER_HALF_W);
    }
    if (action == ACTION_SHOOT) {
        int elapsed = env->tick - env->last_shot_tick;
        if (elapsed >= SHOOT_COOLDOWN) {
            int active_count = 0;
            for (int i = 0; i < MAX_PLAYER_ROCKETS; i++)
                active_count += env->player_rockets[i].active;
            if (active_count < MAX_PLAYER_ROCKETS) {
                for (int i = 0; i < MAX_PLAYER_ROCKETS; i++) {
                    if (!env->player_rockets[i].active) {
                        env->player_rockets[i].active = 1;
                        env->player_rockets[i].x = env->player_x;
                        env->player_rockets[i].y = (float)env->height - 40.0f - PLAYER_HALF_H;
                        env->last_shot_tick = env->tick;
                        break;
                    }
                }
            }
        }
    }

    move_player_rockets(env);
    try_spawn_enemies(env);
    move_enemies(env);
    try_enemy_shoot(env);
    move_enemy_rockets(env);
    update_stars(env);

    check_player_rocket_enemy_collision(env);
    check_enemy_rocket_player_collision(env);
    if (!env->terminals[0])
        check_enemy_player_collision(env);
    if (!env->terminals[0])
        check_wave_complete(env);
}

void c_step(Galaga *env) {
    env->rewards[0] = 0.0f;
    env->terminals[0] = 0;

    if (galaga_game_over_timer > 0)
        return;

    int action = env->actions[0];
    for (int i = 0; i < env->frameskip; i++) {
        env->tick++;
        step_frame(env, action);
        if (env->terminals[0]) break;
    }

    env->episode_return += env->rewards[0];
    if (env->terminals[0] || env->tick >= MAX_TICK) {
        env->terminals[0] = 1;
        add_log(env);
        c_reset(env);
        return;
    }
    compute_observations(env);
}

/* ---- Rendering (Raylib) ---- */

static const Color GAL_BG = {5, 5, 20, 255};
static const Color GAL_PLAYER = {0, 220, 255, 255};
static const Color GAL_PLAYER_ROCKET = {255, 255, 100, 255};
static const Color GAL_ENEMY_ROCKET = {255, 80, 80, 255};
static const Color GAL_WHITE = {230, 230, 230, 255};
static const Color GAL_STAR_DIM = {100, 100, 100, 255};
static const Color GAL_STAR_MID = {120, 120, 120, 255};
static const Color GAL_STAR_BRIGHT = {120, 120, 0, 255};

static const Color ENEMY_COLORS[3] = {
    {255, 50, 50, 255},
    {50, 255, 50, 255},
    {180, 50, 255, 255}
};

static void draw_stars(Galaga *env) {
    Color layer_colors[STAR_LAYERS] = {GAL_STAR_DIM, GAL_STAR_MID, GAL_STAR_BRIGHT};
    int layer_sizes[STAR_LAYERS] = {3, 2, 1};
    for (int layer = 0; layer < STAR_LAYERS; layer++) {
        for (int i = 0; i < MAX_STARS; i++) {
            int idx = layer * MAX_STARS + i;
            DrawRectangle((int)env->stars[idx].x, (int)env->stars[idx].y,
                          layer_sizes[layer], layer_sizes[layer], layer_colors[layer]);
        }
    }
}

static void draw_player(Galaga *env) {
    if (galaga_game_over_timer > 0) return;
    float px = env->player_x;
    float py = (float)env->height - 40.0f;
    Vector2 v0 = {px, py - PLAYER_HALF_H};
    Vector2 v1 = {px - PLAYER_HALF_W * 0.6f, py + PLAYER_HALF_H * 0.5f};
    Vector2 v2 = {px + PLAYER_HALF_W * 0.6f, py + PLAYER_HALF_H * 0.5f};
    DrawTriangle(v0, v2, v1, GAL_PLAYER);
    Vector2 w0 = {px - PLAYER_HALF_W, py + PLAYER_HALF_H};
    Vector2 w1 = {px - PLAYER_HALF_W * 0.4f, py};
    Vector2 w2 = {px + PLAYER_HALF_W * 0.4f, py};
    Vector2 w3 = {px + PLAYER_HALF_W, py + PLAYER_HALF_H};
    DrawTriangle(w0, w2, w1, GAL_PLAYER);
    DrawTriangle(w0, w3, w2, GAL_PLAYER);
}

static void draw_player_rockets(Galaga *env) {
    for (int i = 0; i < MAX_PLAYER_ROCKETS; i++) {
        if (!env->player_rockets[i].active) continue;
        float rx = env->player_rockets[i].x;
        float ry = env->player_rockets[i].y;
        DrawRectangle((int)(rx - 2), (int)(ry - ROCKET_HALF_H), 4, ROCKET_HALF_H * 2, GAL_PLAYER_ROCKET);
    }
}

static void draw_enemies(Galaga *env) {
    for (int i = 0; i < MAX_ENEMIES; i++) {
        if (!env->enemies[i].active) continue;
        Enemy *e = &env->enemies[i];
        Color c = ENEMY_COLORS[e->type % 3];
        float ex = e->x;
        float ey = e->y;
        float hw = ENEMY_HALF_W * 0.8f;
        float hh = ENEMY_HALF_H * 0.8f;
        Vector2 t0 = {ex, ey + hh};
        Vector2 t1 = {ex - hw, ey - hh};
        Vector2 t2 = {ex + hw, ey - hh};
        DrawTriangle(t0, t1, t2, c);
        DrawRectangle((int)(ex - hw * 0.5f), (int)(ey - hh * 0.3f),
                      (int)(hw), (int)(hh * 0.6f), c);
    }
}

static void draw_enemy_rockets(Galaga *env) {
    for (int i = 0; i < MAX_ENEMY_ROCKETS; i++) {
        if (!env->enemy_rockets[i].active) continue;
        float rx = env->enemy_rockets[i].x;
        float ry = env->enemy_rockets[i].y;
        DrawRectangle((int)(rx - 2), (int)(ry - ROCKET_HALF_H), 4, ROCKET_HALF_H * 2, GAL_ENEMY_ROCKET);
    }
}

void c_render(Galaga *env) {
    if (!IsWindowReady()) {
        InitWindow(env->width, env->height, "PufferLib Galaga");
        SetConfigFlags(FLAG_MSAA_4X_HINT);
        SetTargetFPS(60);
        galaga_render_flag = 1;
    }

    if (IsKeyDown(KEY_ESCAPE)) {
        exit(0);
    }

    if (env->terminals[0] == 1 && !galaga_game_over_started) {
        galaga_game_over_started = 1;
        galaga_game_over_timer = 120;
    }
    if (galaga_game_over_timer > 0) {
        galaga_game_over_timer--;
    } else {
        galaga_game_over_started = 0;
    }

    BeginDrawing();
    ClearBackground(GAL_BG);

    draw_stars(env);
    draw_player(env);
    draw_player_rockets(env);
    draw_enemies(env);
    draw_enemy_rockets(env);

    DrawText(TextFormat("Score: %d", env->score), 10, 10, 20, GAL_WHITE);
    DrawText(TextFormat("Wave: %d", env->wave + 1), env->width - 120, 10, 20, GAL_WHITE);

    if (galaga_game_over_timer > 0) {
        const char *txt = "GAME OVER";
        int tw = MeasureText(txt, 40);
        float alpha = (float)galaga_game_over_timer / 120.0f;
        Color c = {255, 50, 50, (unsigned char)(alpha * 255)};
        DrawText(txt, (env->width - tw) / 2, env->height / 2 - 20, 40, c);
    }

    EndDrawing();
}

void c_close(Galaga *env) {
    if (IsWindowReady()) {
        CloseWindow();
    }
}
