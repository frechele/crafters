# Crafter Parity Tests Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a parity test suite that checks the Rust implementation against reference behavior from the original Python Crafter and make the suite pass.

**Architecture:** Generate a small set of deterministic reference fixtures from the original Python implementation for controlled scenarios and short action sequences. Add Rust integration tests that recreate the same scenarios, compare structured state and render outputs, then fix the Rust code where parity breaks.

**Tech Stack:** Rust integration tests, JSON fixtures, temporary Python reference runner, original `danijar/crafter` source

---

### Task 1: Gather parity surface

**Files:**
- Modify: `/Users/jun/myWorks/crafters/docs/plans/2026-03-17-crafter-parity-tests.md`
- Inspect: `/tmp/codex-crafter-src/crafter/crafter/env.py`
- Inspect: `/tmp/codex-crafter-src/crafter/crafter/engine.py`
- Inspect: `/Users/jun/myWorks/crafters/src/lib.rs`
- Inspect: `/Users/jun/myWorks/crafters/src/render.rs`

**Step 1: Identify comparable behaviors**

List deterministic behaviors to compare:
- reset-time daylight and observation shape
- controlled render scenes for day, night, sleeping, objects, inventory
- short action sequences on controlled maps
- step outputs: reward, done, inventory, achievements, semantic player position

**Step 2: Confirm reference runner dependencies**

Run commands that verify whether the original Python environment can be executed locally, and install missing dependencies in a temporary environment if required.

**Step 3: Record the chosen parity scenarios**

Write down the final scenario list before generating fixtures.

### Task 2: Generate failing parity tests

**Files:**
- Create: `/Users/jun/myWorks/crafters/tests/parity.rs`
- Create: `/Users/jun/myWorks/crafters/tests/fixtures/crafter_reference.json`

**Step 1: Create the Rust parity test harness**

Add tests that load reference fixtures and compare:
- render pixels for controlled scenes
- step metadata for short action sequences
- reset metadata for the default environment

**Step 2: Run the parity test file**

Run: `cargo test --test parity`

Expected: FAIL because at least one current behavior differs from the Python reference.

### Task 3: Generate reference fixtures

**Files:**
- Create: `/Users/jun/myWorks/crafters/scripts/generate_reference_fixtures.py`
- Modify: `/Users/jun/myWorks/crafters/tests/fixtures/crafter_reference.json`

**Step 1: Write the reference fixture generator**

The script should:
- import the original Python Crafter from `/tmp/codex-crafter-src/crafter`
- construct the chosen scenarios
- output compact JSON fixtures for the Rust tests

**Step 2: Run the generator**

Run the script and inspect the fixture output.

### Task 4: Fix parity gaps

**Files:**
- Modify: `/Users/jun/myWorks/crafters/src/lib.rs`
- Modify: `/Users/jun/myWorks/crafters/src/render.rs`
- Modify: `/Users/jun/myWorks/crafters/src/world.rs`
- Modify: other Rust source files only if parity tests prove they differ

**Step 1: Fix the smallest failing gap**

Run: `cargo test --test parity -- --exact <failing_test_name>`

Implement the minimum code change needed for the first failing test.

**Step 2: Re-run the targeted test**

Run the exact failing test again and make it pass.

**Step 3: Repeat for remaining parity failures**

Continue one failure at a time until `cargo test --test parity` passes.

### Task 5: Verify the full crate

**Files:**
- Verify only

**Step 1: Run formatting**

Run: `cargo fmt --check`

**Step 2: Run all tests**

Run: `cargo test`

**Step 3: Build the playable runner**

Run: `cargo build --bin play`
