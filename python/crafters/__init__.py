from .env import Env
from .recorder import Recorder

__all__ = ["Env", "Recorder"]

try:
    import gym

    gym.register(
        id="CrafterReward-v1",
        entry_point="crafters:Env",
        max_episode_steps=10000,
        kwargs={"reward": True},
    )
    gym.register(
        id="CrafterNoReward-v1",
        entry_point="crafters:Env",
        max_episode_steps=10000,
        kwargs={"reward": False},
    )
except ImportError:
    pass
except Exception:
    pass
