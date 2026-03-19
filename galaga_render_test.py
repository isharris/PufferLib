"""Save a Galaga replay as MP4 from observation data. No display required."""
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
import imageio

from pufferlib.ocean.galaga.galaga import Galaga

WIDTH, HEIGHT = 1024, 768
NUM_FRAMES = 600  # 20 seconds at 30fps

env = Galaga(num_envs=1, width=WIDTH, height=HEIGHT, frameskip=1)
obs, _ = env.reset()

observations = []
for i in range(NUM_FRAMES):
    actions = np.random.randint(0, 4, size=1)
    obs, rewards, terminals, truncations, infos = env.step(actions)
    observations.append(obs[0].copy())
env.close()

print(f'Rendering {NUM_FRAMES} frames...')
fig, ax = plt.subplots(figsize=(8, 6), dpi=100)
fig.patch.set_facecolor('black')

enemy_colors = ['#ff3232', '#32ff32', '#b432ff']
frames = []

for i, o in enumerate(observations):
    ax.clear()
    ax.set_xlim(0, 1)
    ax.set_ylim(1, 0)
    ax.set_facecolor('#050514')
    ax.set_aspect(HEIGHT / WIDTH)
    ax.axis('off')

    # Player
    ax.plot(o[0], 0.95, marker='v', color='#00dcff', markersize=14)

    # Player rockets (idx 1-4)
    for r in range(2):
        rx, ry = o[1 + r*2], o[2 + r*2]
        if ry > 0:
            ax.plot(rx, ry, marker='|', color='#ffff64', markersize=10, markeredgewidth=2)

    # Enemies (idx 5-34: 10 x (x,y,alive))
    for e in range(10):
        ex, ey, alive = o[5 + e*3], o[6 + e*3], o[7 + e*3]
        if alive > 0.5:
            ax.plot(ex, ey, marker='s', color=enemy_colors[e % 3], markersize=10)

    # Enemy rockets (idx 35-66: 8 x (x,y,vx,vy))
    for r in range(8):
        rx, ry = o[35 + r*4], o[36 + r*4]
        if ry > 0:
            ax.plot(rx, ry, marker='.', color='#ff5050', markersize=7)

    ax.text(0.01, 0.02, f'Frame {i}', color='#e6e6e6', fontsize=9,
            transform=ax.transAxes)

    fig.canvas.draw()
    frame = np.array(fig.canvas.buffer_rgba())[:, :, :3].copy()
    frames.append(frame)

    if (i + 1) % 100 == 0:
        print(f'  {i + 1}/{NUM_FRAMES}')

plt.close()

imageio.mimsave('galaga_test.mp4', frames, fps=30)
print('Saved galaga_test.mp4')
