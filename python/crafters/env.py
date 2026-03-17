import collections

import numpy as np

from ._core import RustEnv as _RustEnv


try:
    import gym

    DiscreteSpace = gym.spaces.Discrete
    BoxSpace = gym.spaces.Box
    DictSpace = gym.spaces.Dict
    BaseClass = gym.Env
except ImportError:
    DiscreteSpace = collections.namedtuple("DiscreteSpace", "n")
    BoxSpace = collections.namedtuple("BoxSpace", "low, high, shape, dtype")
    DictSpace = collections.namedtuple("DictSpace", "spaces")
    BaseClass = object


class _WorldProxy:
    def __init__(self, core, area):
        self._core = core
        self.area = tuple(int(value) for value in area)

    @property
    def daylight(self):
        return float(self._core.daylight)


class _PlayerProxy:
    def __init__(self, core):
        self._core = core

    @property
    def pos(self):
        return self._core.player_pos

    @property
    def inventory(self):
        return self._core.player_inventory()

    @property
    def achievements(self):
        return self._core.player_achievements()

    @property
    def sleeping(self):
        return bool(self._core.player_sleeping)

    @property
    def health(self):
        return int(self._core.player_health)


class Env(BaseClass):
    def __init__(
        self,
        area=(64, 64),
        view=(9, 9),
        size=(64, 64),
        reward=True,
        length=10000,
        seed=None,
    ):
        view = np.array(view if hasattr(view, "__len__") else (view, view))
        size = np.array(size if hasattr(size, "__len__") else (size, size))
        area = tuple(int(value) for value in area)
        seed = int(np.random.randint(0, 2**31 - 1)) if seed is None else int(seed)
        core_length = None if length in (None, 0) else int(length)
        self._area = area
        self._view = view
        self._size = size
        self._reward = reward
        self._length = length
        self._seed = seed
        self._episode = 0
        self._step = None
        self._core = _RustEnv(
            area=area,
            view=tuple(int(value) for value in view),
            size=tuple(int(value) for value in size),
            reward=bool(reward),
            length=core_length,
            seed=seed,
        )
        self._action_names = list(self._core.action_names)
        self._world = _WorldProxy(self._core, area)
        self._player = _PlayerProxy(self._core)
        self._last_health = None
        self._unlocked = None
        self.reward_range = None
        self.metadata = None

    @property
    def observation_space(self):
        shape = tuple(int(value) for value in self._size) + (3,)
        return BoxSpace(0, 255, shape, np.uint8)

    @property
    def action_space(self):
        return DiscreteSpace(len(self._action_names))

    @property
    def action_names(self):
        return list(self._action_names)

    def reset(self):
        self._episode += 1
        self._step = 0
        self._last_health = self._player.health
        self._unlocked = set()
        return self._core.reset()

    def step(self, action):
        action = int(action)
        if action < 0:
            action += len(self._action_names)
        obs, reward, done, info = self._core.step(action)
        self._step = 1 if self._step is None else self._step + 1
        self._last_health = info["inventory"]["health"]
        self._unlocked = {
            name for name, count in info["achievements"].items() if count > 0
        }
        return obs, reward, done, info

    def render(self, size=None):
        if size is None:
            normalized = None
        elif hasattr(size, "__len__"):
            normalized = tuple(int(value) for value in size)
        else:
            normalized = (int(size), int(size))
        return self._core.render(normalized)
