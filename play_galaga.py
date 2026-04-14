"""Interactive Galaga player using matplotlib event handling."""
import sys

import numpy as np
import matplotlib

# TkAgg: keys go to the Tk canvas widget. The macosx backend often never becomes
# the key window during a tight plt.pause() game loop, so arrows/WASD do nothing.
if sys.platform == 'darwin':
    try:
        matplotlib.use('TkAgg')
    except Exception:
        matplotlib.use('macosx')
else:
    try:
        matplotlib.use('TkAgg')
    except Exception:
        matplotlib.use('macosx')

matplotlib.rcParams['toolbar'] = 'none'
matplotlib.rcParams['keymap.back'] = []
matplotlib.rcParams['keymap.forward'] = []

import matplotlib.pyplot as plt

from pufferlib.ocean.galaga.galaga import Galaga

WIDTH, HEIGHT = 1024, 768
FPS = 30


class GalagaPlayer:
    def __init__(self):
        self.env = Galaga(num_envs=1, width=WIDTH, height=HEIGHT, frameskip=1)
        self.obs, _ = self.env.reset()

        # Matplotlib key_event fallback (non-Tk backends)
        self.action_map = {
            'left': 1,
            'right': 2,
            'a': 1,
            'd': 2,
            'w': 3,
            ' ': 3,
            'space': 3,
            'arrowleft': 1,
            'arrowright': 2,
        }
        self.current_action = 0
        self.running = True

        self._tk_input = False
        self._tk_held = set()
        self._tk_order = []

        self.fig, self.ax = plt.subplots(figsize=(10, 7.5), dpi=100)
        self.fig.patch.set_facecolor('black')
        self.enemy_colors = ['#ff3232', '#32ff32', '#b432ff']

        self._install_keyboard()

        self.fig.canvas.mpl_connect('close_event', self.on_close)

        plt.ion()
        self.fig.show()
        plt.show(block=False)
        self._raise_window()

        self.render_loop()

    def _install_keyboard(self):
        canvas = self.fig.canvas
        if not hasattr(canvas, 'get_tk_widget'):
            self.fig.canvas.mpl_connect('key_press_event', self.on_key_press)
            self.fig.canvas.mpl_connect('key_release_event', self.on_key_release)
            return

        self._tk_input = True
        tw = canvas.get_tk_widget()
        tw.configure(takefocus=True)
        tw.bind('<KeyPress>', self._tk_key_press)
        tw.bind('<KeyRelease>', self._tk_key_release)
        tw.bind('<Button-1>', lambda _e: self._tk_focus())
        tw.bind('<Enter>', lambda _e: self._tk_focus())
        self._tk_focus()
        root = tw.winfo_toplevel()
        root.after(50, self._tk_focus)
        root.after(250, self._tk_focus)

    def _tk_focus(self):
        if not self._tk_input:
            return
        tw = self.fig.canvas.get_tk_widget()
        tw.focus_set()
        try:
            tw.focus_force()
        except Exception:
            pass

    @staticmethod
    def _tk_normalize_sym(event):
        sym = (event.keysym or '').lower()
        if event.char == ' ':
            return 'space'
        if sym in ('ascii32', 'kp_space'):
            return 'space'
        return sym

    @staticmethod
    def _tk_sym_to_action(sym):
        if sym in ('left', 'a'):
            return 1
        if sym in ('right', 'd'):
            return 2
        if sym in ('space', 'w'):
            return 3
        return None

    def _tk_recompute_action(self):
        for sym in reversed(self._tk_order):
            if sym in self._tk_held:
                a = self._tk_sym_to_action(sym)
                if a is not None:
                    self.current_action = a
                    return
        self.current_action = 0

    def _tk_key_press(self, event):
        sym = self._tk_normalize_sym(event)
        if self._tk_sym_to_action(sym) is None:
            return
        self._tk_held.add(sym)
        self._tk_order = [s for s in self._tk_order if s != sym]
        self._tk_order.append(sym)
        self._tk_recompute_action()

    def _tk_key_release(self, event):
        sym = self._tk_normalize_sym(event)
        self._tk_held.discard(sym)
        self._tk_order = [s for s in self._tk_order if s != sym]
        self._tk_recompute_action()

    @staticmethod
    def _canon_key(key):
        if key is None:
            return None
        if len(key) == 1:
            return key.lower()
        return key.lower()

    def on_key_press(self, event):
        if self._tk_input:
            return
        key = self._canon_key(event.key)
        if key in self.action_map:
            self.current_action = self.action_map[key]

    def on_key_release(self, event):
        if self._tk_input:
            return
        key = self._canon_key(event.key)
        if key in self.action_map:
            self.current_action = 0

    def on_close(self, event):
        self.running = False

    def _raise_window(self):
        if self._tk_input:
            try:
                tw = self.fig.canvas.get_tk_widget()
                tw.winfo_toplevel().lift()
            except Exception:
                pass
            return
        try:
            win = self.fig.canvas.manager.window
            win.lift()
            win.attributes('-topmost', True)
            win.attributes('-topmost', False)
        except Exception:
            pass

    def render_loop(self):
        while self.running:
            action = np.array([self.current_action])
            self.obs, rewards, terminals, _, infos = self.env.step(action)
            o = self.obs[0].copy()

            self.ax.clear()
            self.ax.set_xlim(0, 1)
            self.ax.set_ylim(1, 0)
            self.ax.set_facecolor('#050514')
            self.ax.set_aspect(HEIGHT / WIDTH)
            self.ax.axis('off')

            self.ax.plot(o[0], 0.95, marker='v', color='#00dcff', markersize=14)

            for r in range(2):
                rx, ry = o[1 + r * 2], o[2 + r * 2]
                if ry > 0:
                    self.ax.plot(
                        rx,
                        ry,
                        marker='|',
                        color='#ffff64',
                        markersize=10,
                        markeredgewidth=2,
                    )

            for e in range(10):
                ex, ey, alive = o[5 + e * 3], o[6 + e * 3], o[7 + e * 3]
                if alive > 0.5:
                    self.ax.plot(
                        ex,
                        ey,
                        marker='s',
                        color=self.enemy_colors[e % 3],
                        markersize=10,
                    )

            for r in range(8):
                rx, ry = o[35 + r * 4], o[36 + r * 4]
                if ry > 0:
                    self.ax.plot(rx, ry, marker='.', color='#ff5050', markersize=7)

            hint = (
                "Click the plot once if keys do nothing. "
                "Arrows or A/D to move, Space or W to shoot. Close to quit."
            )
            self.ax.text(
                0.01,
                0.02,
                hint,
                color='#e6e6e6',
                fontsize=10,
                transform=self.ax.transAxes,
            )

            self.fig.canvas.draw()
            self.fig.canvas.flush_events()

            if bool(terminals[0]):
                print("Episode over — respawning")
                self.obs, _ = self.env.reset()

            plt.pause(1.0 / FPS)

        self.env.close()
        plt.close()


if __name__ == '__main__':
    WIDTH, HEIGHT = 1024, 768
    GalagaPlayer()
