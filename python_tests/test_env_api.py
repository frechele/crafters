import unittest


EXPECTED_ACTION_NAMES = [
    "noop",
    "move_left",
    "move_right",
    "move_up",
    "move_down",
    "do",
    "sleep",
    "place_stone",
    "place_table",
    "place_furnace",
    "place_plant",
    "make_wood_pickaxe",
    "make_stone_pickaxe",
    "make_iron_pickaxe",
    "make_wood_sword",
    "make_stone_sword",
    "make_iron_sword",
]


class CraftersApiTest(unittest.TestCase):
    def test_env_matches_crafter_style_surface(self) -> None:
        from crafters import Env

        env = Env(seed=0)
        import numpy as np

        self.assertIsNone(env.reward_range)
        self.assertIsNone(env.metadata)
        self.assertEqual(env.action_names, EXPECTED_ACTION_NAMES)
        self.assertEqual(env.action_space.n, len(EXPECTED_ACTION_NAMES))
        self.assertEqual(env.observation_space.shape, (64, 64, 3))
        self.assertIs(env.observation_space.dtype, np.uint8)

        obs = env.reset()
        self.assertIsInstance(obs, np.ndarray)
        self.assertEqual(obs.shape, (64, 64, 3))
        self.assertEqual(obs.dtype, np.uint8)

        step = env.step(env.action_names.index("noop"))
        self.assertEqual(len(step), 4)
        next_obs, reward, done, info = step
        self.assertIsInstance(next_obs, np.ndarray)
        self.assertEqual(next_obs.shape, (64, 64, 3))
        self.assertIsInstance(reward, float)
        self.assertIsInstance(done, bool)
        self.assertEqual(
            set(info),
            {"inventory", "achievements", "discount", "semantic", "player_pos", "reward"},
        )
        self.assertIsInstance(info["semantic"], np.ndarray)
        self.assertEqual(info["semantic"].shape, (64, 64))
        self.assertEqual(tuple(info["player_pos"]), (32, 32))

        render = env.render(size=(72, 96))
        self.assertEqual(render.shape, (96, 72, 3))
        self.assertEqual(render.dtype, np.uint8)

    def test_recorder_is_exported_and_delegates(self) -> None:
        from crafters import Env, Recorder

        recorder = Recorder(Env(seed=1), directory=None)
        obs = recorder.reset()
        self.assertEqual(obs.shape, (64, 64, 3))


if __name__ == "__main__":
    unittest.main()
