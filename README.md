# Crafter RS

`crafters` is a Rust-backed port of the original
[`danijar/crafter`](https://github.com/danijar/crafter) environment.
It is built to feel familiar to existing Crafter users while giving you a
faster native core and a lightweight local runner.

The sprite assets in `assets/crafter/` are copied from the original repository
under its MIT license.

## Why use it

- Keep a Python-facing API that is close to the original `crafter` package.
- Run the environment on a Rust core instead of the original Python engine.
- Preserve the action set, reward structure, semantic observations, and common
  training loop shape.
- Switch trained agents and evaluation code with minimal changes.

Recent parity work focused on matching the original environment more closely,
including world generation, entity population, and long-horizon balancing. If
you trained agents on the original Crafter environment, `crafters` is intended
to be a practical replacement rather than a different benchmark.

## Install

Install the Python package with:

```bash
python3 -m pip install -e .
```

If you also want Gym registration helpers:

```bash
python3 -m pip install -e '.[gym]'
```

## Quick start

```python
from crafters import Env

env = Env(seed=0)
obs = env.reset()

done = False
while not done:
    action = env.action_names.index("noop")
    obs, reward, done, info = env.step(action)
```

The standard step contract is the familiar:

```python
obs, reward, done, info = env.step(action)
```

where:

- `obs` is an RGB `numpy.ndarray`
- `reward` is a float
- `done` is a boolean
- `info` contains `inventory`, `achievements`, `discount`, `semantic`,
  `player_pos`, and `reward`

## Migration guide for `crafter` users

If your code already uses the original Python Crafter package, the smallest
possible migration is usually:

```python
# before
from crafter import Env

# after
from crafters import Env
```

and keep the rest of the control loop unchanged.

### What stays the same

- `Env(...)` accepts the same high-level arguments: `area`, `view`, `size`,
  `reward`, `length`, and `seed`
- `reset()`, `step(action)`, and `render(size=None)` follow the same shape
- `action_names` is available and uses the same action strings
- `action_space` and `observation_space` are exposed
- Gym IDs `CrafterReward-v1` and `CrafterNoReward-v1` are registered when Gym
  is installed
- `info["semantic"]` is available for policy/debugging code that uses semantic
  observations
- `Recorder` is available for episode, stats, and video dumps

### What to change

- Change imports from `crafter` to `crafters`
- Install with `pip install -e .` instead of installing the original package
- If you depend on original private engine internals, re-check those call sites
  because `crafters` intentionally exposes a smaller Python surface over a Rust
  core

### Practical migration tips

- If your code already indexes actions by name, keep doing that:

  ```python
  action = env.action_names.index("make_iron_pickaxe")
  ```

- If you use Gym wrappers, prefer `gym.make("CrafterReward-v1")` or
  `gym.make("CrafterNoReward-v1")` after installing the optional Gym extra.

- If you evaluate agents trained on original Crafter, do not rewrite reward
  logic or achievement handling first. Try the same evaluation harness and only
  change imports.

- If you compare transfer results against original Crafter, compare on the same
  seeds and episode lengths. `crafters` is designed for parity, so seed-matched
  comparisons are the most useful sanity check.

- If you used `info["semantic"]` or `info["player_pos"]` for debugging or
  auxiliary losses, those fields are still there.

- If your old code reached deep into Python world objects for custom
  manipulation, expect to adapt those parts. The public training loop is close;
  private engine internals are not a 1:1 Python object graph.

## Python examples

### Action-by-name loop

```python
from crafters import Env

env = Env(seed=0, reward=True)
obs = env.reset()

for _ in range(100):
    action = env.action_names.index("noop")
    obs, reward, done, info = env.step(action)
    if done:
        break
```

### Gym usage

```python
import gym
import crafters

env = gym.make("CrafterReward-v1")
obs = env.reset()
```

### Recorder usage

```python
from crafters import Env, Recorder

env = Recorder(Env(seed=0), directory="logs/crafter-run")
obs = env.reset()

done = False
while not done:
    obs, reward, done, info = env.step(env.action_names.index("noop"))
```

## Local runner

Run the native desktop runner with:

```bash
cargo run --bin play
```

## Controls

- Arrow keys: move
- Space or Enter: interact / mine / attack
- `E`: sleep
- `1`: place stone
- `2`: place table
- `3`: place furnace
- `4`: place plant
- `Z/X/C`: craft wood/stone/iron pickaxe
- `A/S/D`: craft wood/stone/iron sword
- `N`: wait one step
- `R`: reset episode
- `Esc`: quit
