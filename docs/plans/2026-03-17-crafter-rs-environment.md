# Crafter Rust Environment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a standalone Rust crate that reproduces the Crafter environment core: reset/step loop, procedural world generation, entity behaviors, rewards, done conditions, semantic state, and RGB rendering.

**Architecture:** Keep the port as a pure Rust library with a small public `Env` API. Split static game rules from runtime world state so tests can exercise mechanics without depending on rendering. Reproduce the Python logic closely, but use typed enums/structs instead of dynamic dictionaries where possible.

**Tech Stack:** Rust 2024, `cargo test`, `rand`, `opensimplex_noise_rs` or equivalent simplex crate, `image` for PNG decoding and resize, `serde` for optional structured outputs.

---

### Task 1: Crate Skeleton And Rules Model

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Create: `src/config.rs`
- Create: `src/types.rs`
- Test: `tests/env_basics.rs`

**Step 1: Write the failing test**

Write a reset test that expects:
- fixed action count `17`
- default RGB observation shape `64x64x3`
- player inventory initialized to Crafter defaults

**Step 2: Run test to verify it fails**

Run: `cargo test env_reset_initializes_default_spaces --test env_basics -- --exact`
Expected: FAIL because `Env` and related types do not exist yet.

**Step 3: Write minimal implementation**

Add typed enums and structs for actions, materials, items, and basic environment configuration. Expose a placeholder `Env::new`, `Env::reset`, and `Env::action_names`.

**Step 4: Run test to verify it passes**

Run: `cargo test env_reset_initializes_default_spaces --test env_basics -- --exact`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/config.rs src/types.rs tests/env_basics.rs
git commit -m "feat: scaffold crafter rust environment"
```

### Task 2: World State And Deterministic Reset

**Files:**
- Create: `src/world.rs`
- Create: `src/entities.rs`
- Modify: `src/lib.rs`
- Test: `tests/env_basics.rs`

**Step 1: Write the failing test**

Add tests that assert:
- `reset()` increments episode seed deterministically
- player starts in world center
- semantic map size equals configured area
- world materials are fully populated after reset

**Step 2: Run test to verify it fails**

Run: `cargo test env_reset_builds_world_state --test env_basics -- --exact`
Expected: FAIL because runtime world generation and player placement are missing.

**Step 3: Write minimal implementation**

Implement `World`, chunk bookkeeping, material map, object slots, and player spawn/reset wiring.

**Step 4: Run test to verify it passes**

Run: `cargo test env_reset_builds_world_state --test env_basics -- --exact`
Expected: PASS

**Step 5: Commit**

```bash
git add src/world.rs src/entities.rs src/lib.rs tests/env_basics.rs
git commit -m "feat: add world state and reset flow"
```

### Task 3: Terrain Generation

**Files:**
- Create: `src/worldgen.rs`
- Modify: `src/world.rs`
- Modify: `src/entities.rs`
- Test: `tests/worldgen.rs`

**Step 1: Write the failing test**

Add tests that assert:
- generated maps contain only valid Crafter materials
- spawn area near the player is safe walkable terrain
- same seed and episode yield identical terrain

**Step 2: Run test to verify it fails**

Run: `cargo test --test worldgen`
Expected: FAIL because terrain generation is stubbed or incomplete.

**Step 3: Write minimal implementation**

Port the simplex-based terrain and spawn-object generation from Python, including tunnels and initial cows/zombies/skeletons.

**Step 4: Run test to verify it passes**

Run: `cargo test --test worldgen`
Expected: PASS

**Step 5: Commit**

```bash
git add src/worldgen.rs src/world.rs src/entities.rs tests/worldgen.rs
git commit -m "feat: port crafter terrain generation"
```

### Task 4: Player Actions, Survival, And Crafting Rules

**Files:**
- Modify: `src/entities.rs`
- Modify: `src/world.rs`
- Modify: `src/lib.rs`
- Test: `tests/gameplay.rs`

**Step 1: Write the failing test**

Add focused tests for:
- movement and facing
- collecting trees/stone/water with tool requirements
- placing stone/table/furnace/plant
- crafting pickaxes and swords
- hunger/thirst/energy updates and health degeneration/regeneration

**Step 2: Run test to verify it fails**

Run: `cargo test --test gameplay`
Expected: FAIL because action handling is incomplete.

**Step 3: Write minimal implementation**

Port `Player::update` behavior and related helpers as directly as possible while keeping Rust APIs typed and testable.

**Step 4: Run test to verify it passes**

Run: `cargo test --test gameplay`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entities.rs src/world.rs src/lib.rs tests/gameplay.rs
git commit -m "feat: add crafter player mechanics"
```

### Task 5: Hostile Entities And Rewards

**Files:**
- Modify: `src/entities.rs`
- Modify: `src/lib.rs`
- Test: `tests/entities.rs`

**Step 1: Write the failing test**

Add tests for:
- zombie melee damage and cooldown
- skeleton movement and arrow spawning
- arrow collisions with objects/materials
- reward calculation from health delta and newly unlocked achievements
- done conditions from death and episode length

**Step 2: Run test to verify it fails**

Run: `cargo test --test entities`
Expected: FAIL because NPC update logic and reward accounting are incomplete.

**Step 3: Write minimal implementation**

Port cow, zombie, skeleton, arrow, and plant behaviors. Finish `Env::step` info output and reward/done logic.

**Step 4: Run test to verify it passes**

Run: `cargo test --test entities`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entities.rs src/lib.rs tests/entities.rs
git commit -m "feat: add npc behavior and rewards"
```

### Task 6: Semantic And RGB Rendering

**Files:**
- Create: `src/render.rs`
- Modify: `src/lib.rs`
- Create: `assets/` (copied MIT-licensed sprites if used)
- Test: `tests/render.rs`

**Step 1: Write the failing test**

Add tests that assert:
- semantic output marks materials and supported object classes distinctly
- `render()` returns configured image size
- inventory strip is rendered into the final image

**Step 2: Run test to verify it fails**

Run: `cargo test --test render`
Expected: FAIL because rendering is absent.

**Step 3: Write minimal implementation**

Implement local-view rendering, item rendering, daylight tinting, and semantic view. Use bundled assets if available; otherwise use a deterministic fallback palette renderer.

**Step 4: Run test to verify it passes**

Run: `cargo test --test render`
Expected: PASS

**Step 5: Commit**

```bash
git add src/render.rs src/lib.rs assets tests/render.rs
git commit -m "feat: add crafter rgb renderer"
```

### Task 7: Final Verification And API Cleanup

**Files:**
- Modify: `README.md`
- Modify: `src/lib.rs`
- Test: `tests/*.rs`

**Step 1: Write the failing test**

Add one end-to-end smoke test that runs a short random episode and asserts the environment stays internally consistent.

**Step 2: Run test to verify it fails**

Run: `cargo test random_episode_smoke`
Expected: FAIL until all pieces are connected.

**Step 3: Write minimal implementation**

Tighten the public API, add example docs, and remove dead placeholders.

**Step 4: Run test to verify it passes**

Run: `cargo test`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md src/lib.rs tests
git commit -m "docs: finalize crafter rust environment api"
```
