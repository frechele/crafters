# Crafter RS

Rust port of the Crafter environment with a simple local player runner.
The sprite assets in `assets/crafter/` are copied from the original
[`danijar/crafter`](https://github.com/danijar/crafter) repository under its MIT license.

## Run

```bash
cargo run --bin play
```

## Python API

Install the PyO3-backed package with:

```bash
python3 -m pip install -e .
```

The goal is to keep the surface close to the original Python package. In many
call sites, switching to the Rust-backed environment should be as small as:

```python
from crafters import Env

env = Env(seed=0)
obs = env.reset()
obs, reward, done, info = env.step(env.action_names.index("noop"))
```

## Controls

- Arrow keys: move
- Space or Enter: interact / mine / attack
- E: sleep
- 1: place stone
- 2: place table
- 3: place furnace
- 4: place plant
- Z/X/C: craft wood/stone/iron pickaxe
- A/S/D: craft wood/stone/iron sword
- N: wait one step
- R: reset episode
- Esc: quit
