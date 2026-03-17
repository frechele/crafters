#!/usr/bin/env python3

import hashlib
import json
import pathlib
import sys

import numpy as np


ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = pathlib.Path("/tmp/codex-crafter-src/crafter")
SOURCE = pathlib.Path(
    sys.argv[2] if len(sys.argv) > 2 else DEFAULT_SOURCE
).resolve()

if str(SOURCE) not in sys.path:
    sys.path.insert(0, str(SOURCE))

import crafter
from crafter import objects


ITEM_NAMES = [
    "health",
    "food",
    "drink",
    "energy",
    "sapling",
    "wood",
    "stone",
    "coal",
    "iron",
    "diamond",
    "wood_pickaxe",
    "stone_pickaxe",
    "iron_pickaxe",
    "wood_sword",
    "stone_sword",
    "iron_sword",
]

ACHIEVEMENT_NAMES = [
    "collect_coal",
    "collect_diamond",
    "collect_drink",
    "collect_iron",
    "collect_sapling",
    "collect_stone",
    "collect_wood",
    "defeat_skeleton",
    "defeat_zombie",
    "eat_cow",
    "eat_plant",
    "make_iron_pickaxe",
    "make_iron_sword",
    "make_stone_pickaxe",
    "make_stone_sword",
    "make_wood_pickaxe",
    "make_wood_sword",
    "place_furnace",
    "place_plant",
    "place_stone",
    "place_table",
    "wake_up",
]


def main() -> None:
    output = pathlib.Path(
        sys.argv[1]
        if len(sys.argv) > 1
        else ROOT / "tests" / "fixtures" / "crafter_reference.json"
    )
    output.parent.mkdir(parents=True, exist_ok=True)
    fixture = {
        "action_names": list(crafter.Env(seed=0).action_names),
        "daylight_timeline": build_daylight_timeline(),
        "render_scenarios": build_render_scenarios(),
        "night_render_scenarios": build_night_render_scenarios(),
        "step_scenarios": build_step_scenarios(),
    }
    output.write_text(json.dumps(fixture, indent=2, sort_keys=True) + "\n")


def build_daylight_timeline():
    env = crafter.Env(seed=0)
    env.reset()
    timeline = [{"label": "reset", "daylight": float(env._world.daylight)}]
    for action in ["noop", "noop", "noop"]:
        env.step(env.action_names.index(action))
        timeline.append({"action": action, "daylight": float(env._world.daylight)})
    return timeline


def build_render_scenarios():
    scenarios = []
    for name, sleeping in [("showcase_day", False), ("showcase_sleep", True)]:
        setup = showcase_setup(daylight=1.0, sleeping=sleeping)
        env = build_env(setup)
        scenarios.append(
            {
                "name": name,
                "setup": setup,
                "snapshot": render_snapshot(env, setup["render_size"]),
            }
        )
    return scenarios


def build_night_render_scenarios():
    setup = showcase_setup(daylight=0.0, sleeping=False)
    env = build_env(setup)
    first = env.render(size=tuple(setup["render_size"]))
    second = env.render(size=tuple(setup["render_size"]))
    metrics = {
        "edge_tile": [0, 0],
        "inner_tile": [3, 2],
        "unit": 16,
        "edge_luma": average_tile_luma(first, 16, 0, 0),
        "inner_luma": average_tile_luma(first, 16, 3, 2),
        "frames_differ": sha256_frame(first) != sha256_frame(second),
    }
    return [{"name": "showcase_night", "setup": setup, "metrics": metrics}]


def build_step_scenarios():
    return [
        step_scenario(
            "crafting_chain",
            {
                "config": default_config(),
                "world_fill": "grass",
                "materials": [{"pos": [33, 32], "material": "tree"}],
                "objects": [],
                "player": {
                    "pos": [32, 32],
                    "facing": "right",
                    "inventory": {"wood": 2},
                },
            },
            ["do", "place_table", "make_wood_sword"],
        ),
        step_scenario(
            "stone_without_pickaxe",
            {
                "config": default_config(),
                "world_fill": "grass",
                "materials": [{"pos": [33, 32], "material": "stone"}],
                "objects": [],
                "player": {
                    "pos": [32, 32],
                    "facing": "right",
                },
            },
            ["do"],
        ),
        step_scenario(
            "sleep_wake",
            {
                "config": default_config(),
                "world_fill": "grass",
                "materials": [],
                "objects": [],
                "player": {
                    "pos": [32, 32],
                    "facing": "right",
                    "inventory": {"energy": 8},
                    "fatigue": -11,
                },
            },
            ["sleep", "noop"],
        ),
        step_scenario(
            "lava_death",
            {
                "config": default_config(),
                "world_fill": "grass",
                "materials": [{"pos": [33, 32], "material": "lava"}],
                "objects": [],
                "player": {
                    "pos": [32, 32],
                    "facing": "right",
                },
            },
            ["move_right"],
        ),
    ]


def step_scenario(name, setup, actions):
    env = build_env(setup)
    snapshots = []
    for action in actions:
        obs, reward, done, info = env.step(env.action_names.index(action))
        snapshots.append(step_snapshot(env, obs, reward, done, info))
    return {
        "name": name,
        "setup": setup,
        "actions": actions,
        "snapshots": snapshots,
    }


def showcase_setup(daylight, sleeping):
    return {
        "config": default_config(),
        "render_size": [144, 144],
        "world_fill": "grass",
        "materials": [
            {"pos": [31, 31], "material": "sand"},
            {"pos": [32, 31], "material": "path"},
            {"pos": [33, 31], "material": "tree"},
            {"pos": [34, 31], "material": "stone"},
            {"pos": [35, 31], "material": "coal"},
            {"pos": [36, 31], "material": "iron"},
            {"pos": [37, 31], "material": "diamond"},
            {"pos": [31, 32], "material": "water"},
            {"pos": [30, 32], "material": "lava"},
            {"pos": [33, 33], "material": "table"},
            {"pos": [34, 33], "material": "furnace"},
        ],
        "objects": [
            {"kind": "cow", "pos": [31, 33]},
            {"kind": "zombie", "pos": [33, 32]},
            {"kind": "skeleton", "pos": [34, 32]},
            {"kind": "plant", "pos": [35, 32]},
            {"kind": "arrow", "pos": [36, 32], "facing": "left"},
        ],
        "daylight": daylight,
        "player": {
            "pos": [32, 32],
            "facing": "left",
            "sleeping": sleeping,
            "inventory": {
                "wood": 3,
                "stone": 4,
                "coal": 5,
                "iron": 6,
                "diamond": 1,
                "wood_pickaxe": 1,
                "stone_sword": 1,
            },
        },
    }


def default_config():
    return {
        "area": [64, 64],
        "view": [9, 9],
        "size": [64, 64],
        "reward": True,
        "length": 10000,
        "seed": 0,
    }


def build_env(setup):
    config = setup["config"]
    env = crafter.Env(
        area=tuple(config["area"]),
        view=tuple(config["view"]),
        size=tuple(config["size"]),
        reward=config["reward"],
        length=config["length"],
        seed=config["seed"],
    )
    env.reset()
    clear_non_player_objects(env)
    fill_world(env, setup["world_fill"])

    player = env._player
    move_player(env, np.array(setup["player"].get("pos", player.pos)))
    player.facing = direction_tuple(setup["player"].get("facing", "down"))
    player.sleeping = bool(setup["player"].get("sleeping", False))
    for name, value in setup["player"].get("inventory", {}).items():
        player.inventory[name] = int(value)
    if "hunger" in setup["player"]:
        player._hunger = float(setup["player"]["hunger"])
    if "thirst" in setup["player"]:
        player._thirst = float(setup["player"]["thirst"])
    if "fatigue" in setup["player"]:
        player._fatigue = int(setup["player"]["fatigue"])
    if "recover" in setup["player"]:
        player._recover = float(setup["player"]["recover"])
    if "last_health" in setup["player"]:
        player._last_health = int(setup["player"]["last_health"])

    for patch in setup.get("materials", []):
        env._world[tuple(patch["pos"])] = patch["material"]

    for obj in setup.get("objects", []):
        spawn_object(env, obj)

    if "daylight" in setup:
        env._world.daylight = float(setup["daylight"])
    return env


def clear_non_player_objects(env):
    for obj in list(env._world.objects):
        if obj is not env._player:
            env._world.remove(obj)


def fill_world(env, material):
    for x in range(env._world.area[0]):
        for y in range(env._world.area[1]):
            env._world[(x, y)] = material


def move_player(env, pos):
    if np.array_equal(env._player.pos, pos):
        return
    env._world.move(env._player, pos)


def spawn_object(env, spec):
    pos = tuple(spec["pos"])
    kind = spec["kind"]
    if kind == "cow":
        obj = objects.Cow(env._world, pos)
    elif kind == "zombie":
        obj = objects.Zombie(env._world, pos, env._player)
    elif kind == "skeleton":
        obj = objects.Skeleton(env._world, pos, env._player)
    elif kind == "plant":
        obj = objects.Plant(env._world, pos)
    elif kind == "arrow":
        obj = objects.Arrow(env._world, pos, direction_tuple(spec["facing"]))
    else:
        raise ValueError(f"unsupported object kind: {kind}")
    env._world.add(obj)


def direction_tuple(name):
    return {
        "left": (-1, 0),
        "right": (1, 0),
        "up": (0, -1),
        "down": (0, 1),
    }[name]


def render_snapshot(env, render_size):
    frame = env.render(size=tuple(render_size))
    semantic = env._sem_view()
    return snapshot_payload(
        frame=frame,
        semantic=semantic,
        player=env._player,
        daylight=env._world.daylight,
    )


def step_snapshot(env, obs, reward, done, info):
    return snapshot_payload(
        frame=obs,
        semantic=info["semantic"],
        player=env._player,
        daylight=env._world.daylight,
        reward=reward,
        done=done,
        discount=info["discount"],
    )


def snapshot_payload(frame, semantic, player, daylight, reward=None, done=None, discount=None):
    payload = {
        "frame": {
            "width": int(frame.shape[1]),
            "height": int(frame.shape[0]),
            "channels": int(frame.shape[2]),
            "sha256": sha256_frame(frame),
        },
        "semantic": {
            "width": int(semantic.shape[0]),
            "height": int(semantic.shape[1]),
            "cells": [int(value) for value in semantic.flatten().tolist()],
        },
        "player_pos": [int(player.pos[0]), int(player.pos[1])],
        "sleeping": bool(player.sleeping),
        "daylight": float(daylight),
        "inventory": {name: int(player.inventory[name]) for name in ITEM_NAMES},
        "achievements": {
            name: int(player.achievements[name]) for name in ACHIEVEMENT_NAMES
        },
    }
    if reward is not None:
        payload["reward"] = float(reward)
    if done is not None:
        payload["done"] = bool(done)
    if discount is not None:
        payload["discount"] = float(discount)
    return payload


def sha256_frame(frame):
    return hashlib.sha256(np.asarray(frame, dtype=np.uint8).tobytes()).hexdigest()


def average_tile_luma(frame, unit, tile_x, tile_y):
    start_x = tile_x * unit
    start_y = tile_y * unit
    tile = frame[start_y : start_y + unit, start_x : start_x + unit]
    luma = 0.299 * tile[..., 0] + 0.587 * tile[..., 1] + 0.114 * tile[..., 2]
    return float(luma.mean())


if __name__ == "__main__":
    main()
