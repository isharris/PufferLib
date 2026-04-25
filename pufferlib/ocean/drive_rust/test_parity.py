"""
Parity test: run the C Drive and Rust DriveRust side-by-side on the same
map with the same deterministic actions, then compare observations / rewards /
terminals after each step.

Usage (from repo root, after building both bindings):
    python -m pufferlib.ocean.drive_rust.test_parity

Requires:
  - pufferlib/resources/drive/binaries/map_000.bin  (at least one map)
  - Both C and Rust bindings installed
"""

import numpy as np
import sys
import os

import pufferlib

SEED = 42
NUM_AGENTS = 16
NUM_MAPS = 1
NUM_STEPS = 91


def make_c_env():
    from pufferlib.ocean.drive.drive import Drive
    return Drive(
        num_agents=NUM_AGENTS,
        num_maps=NUM_MAPS,
        seed=SEED,
        resample_frequency=0,
    )


def make_rust_env():
    from pufferlib.ocean.drive_rust.drive_rust import DriveRust
    return DriveRust(
        num_agents=NUM_AGENTS,
        num_maps=NUM_MAPS,
        seed=SEED,
        resample_frequency=0,
    )


def run_parity():
    sample_binary = os.path.join(
        pufferlib.__path__[0], "resources", "drive", "binaries", "map_000.bin"
    )
    if not os.path.exists(sample_binary):
        print(f"SKIP: map binaries not found at {sample_binary}")
        sys.exit(0)

    try:
        c_env = make_c_env()
    except Exception as e:
        print(f"SKIP: C env not available ({e})")
        sys.exit(0)

    try:
        rs_env = make_rust_env()
    except Exception as e:
        print(f"SKIP: Rust env not available ({e})")
        sys.exit(0)

    c_obs, _ = c_env.reset(seed=SEED)
    rs_obs, _ = rs_env.reset(seed=SEED)

    n = min(len(c_obs), len(rs_obs))
    rng = np.random.RandomState(SEED)

    max_obs_diff = 0.0
    max_rew_diff = 0.0
    term_mismatches = 0

    for step in range(NUM_STEPS):
        accel = rng.randint(0, 7, size=(n,))
        steer = rng.randint(0, 13, size=(n,))
        actions = np.stack([accel, steer], axis=-1).astype(np.int32)

        c_result = c_env.step(actions[:len(c_obs)])
        rs_result = rs_env.step(actions[:len(rs_obs)])

        c_obs_s, c_rew_s, c_term_s = c_result[0][:n], c_result[1][:n], c_result[2][:n]
        r_obs_s, r_rew_s, r_term_s = rs_result[0][:n], rs_result[1][:n], rs_result[2][:n]

        obs_diff = np.max(np.abs(c_obs_s.astype(np.float64) - r_obs_s.astype(np.float64)))
        rew_diff = np.max(np.abs(c_rew_s.astype(np.float64) - r_rew_s.astype(np.float64)))
        term_mismatch = int(np.sum(c_term_s != r_term_s))

        max_obs_diff = max(max_obs_diff, obs_diff)
        max_rew_diff = max(max_rew_diff, rew_diff)
        term_mismatches += term_mismatch

        if step % 10 == 0:
            print(
                f"  step {step:3d}  obs_diff={obs_diff:.6e}  "
                f"rew_diff={rew_diff:.6e}  term_mismatch={term_mismatch}"
            )

    c_env.close()
    rs_env.close()

    print()
    print(f"Max obs diff:      {max_obs_diff:.6e}")
    print(f"Max reward diff:   {max_rew_diff:.6e}")
    print(f"Terminal mismatches: {term_mismatches}")

    tol = 1e-4
    if max_obs_diff < tol and max_rew_diff < tol and term_mismatches == 0:
        print("\nPARITY OK")
    else:
        print(f"\nPARITY FAILED (tolerance={tol})")
        sys.exit(1)


if __name__ == "__main__":
    run_parity()
