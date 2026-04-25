//! CPU port of the V-trace advantage kernel from the original
//! `pufferlib/extensions/pufferlib.cpp`.

/// V-trace advantage for one row (one trajectory of length `horizon`).
///
/// # Safety
/// All pointers must be valid for `horizon` `f32` reads / writes.
pub unsafe fn puff_advantage_row(
    values: *const f32,
    rewards: *const f32,
    dones: *const f32,
    importance: *const f32,
    advantages: *mut f32,
    gamma: f32,
    lambda: f32,
    rho_clip: f32,
    c_clip: f32,
    horizon: usize,
) {
    if horizon < 2 {
        return;
    }
    let mut lastpufferlam = 0.0f32;
    for t in (0..horizon - 1).rev() {
        let t_next = t + 1;
        let nextnonterminal = 1.0 - *dones.add(t_next);
        let imp_t = *importance.add(t);
        let rho_t = imp_t.min(rho_clip);
        let c_t = imp_t.min(c_clip);
        let delta = rho_t
            * (*rewards.add(t_next) + gamma * *values.add(t_next) * nextnonterminal
                - *values.add(t));
        lastpufferlam = delta + gamma * lambda * c_t * lastpufferlam * nextnonterminal;
        *advantages.add(t) = lastpufferlam;
    }
}

/// V-trace advantage for `[num_steps, horizon]`-shaped buffers in row-major
/// order. Each row is processed independently.
///
/// # Safety
/// All pointers must be valid for `num_steps * horizon` `f32` reads / writes.
pub unsafe fn puff_advantage(
    values: *const f32,
    rewards: *const f32,
    dones: *const f32,
    importance: *const f32,
    advantages: *mut f32,
    gamma: f32,
    lambda: f32,
    rho_clip: f32,
    c_clip: f32,
    num_steps: usize,
    horizon: usize,
) {
    let total = num_steps * horizon;
    let mut offset = 0usize;
    while offset < total {
        puff_advantage_row(
            values.add(offset),
            rewards.add(offset),
            dones.add(offset),
            importance.add(offset),
            advantages.add(offset),
            gamma,
            lambda,
            rho_clip,
            c_clip,
            horizon,
        );
        offset += horizon;
    }
}
