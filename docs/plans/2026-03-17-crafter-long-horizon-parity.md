# Crafter Long-Horizon Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port the missing long-horizon environment behavior from the original Python Crafter so learned agents see closer skeleton, zombie, cow, and resource-access dynamics in Rust.

**Architecture:** Keep the existing Rust entity update rules, but add the missing chunk bookkeeping and periodic chunk balancing that the Python environment applies every 10 steps. Back the change with regression tests that fail without balancing and verify object counts and spawn locations remain aligned with the original rules.

**Tech Stack:** Rust, cargo integration tests, existing `Env`/`World` runtime, original Python Crafter source as reference

---

### Task 1: Lock in the missing behavior with failing tests

**Files:**
- Modify: `/Users/jun/myWorks/crafters/tests/entities.rs`
- Test: `/Users/jun/myWorks/crafters/tests/entities.rs`

**Step 1: Write a regression test for chunk-based skeleton spawning**

Add a test that prepares a deterministic all-path chunk with no nearby player interference, advances 10 steps, and asserts a skeleton appears in a valid path tile only after the periodic balancing pass.

**Step 2: Run the new test to verify it fails**

Run: `cargo test --test entities chunk_balancing -- --nocapture`
Expected: FAIL because the current Rust env never balances chunks.

**Step 3: Add a regression test for balancing bookkeeping**

Add a test that moves or removes objects and confirms chunk membership stays correct enough for balancing decisions to observe current object placement.

**Step 4: Run the bookkeeping test to verify it fails or is blocked by missing APIs**

Run: `cargo test --test entities chunk_bookkeeping -- --nocapture`
Expected: FAIL or require missing world support.

### Task 2: Add world chunk bookkeeping support

**Files:**
- Modify: `/Users/jun/myWorks/crafters/src/world.rs`

**Step 1: Add chunk-key computation and chunk object membership tracking**

Store per-chunk object membership so balancing can inspect objects by chunk without scanning the full world every time.

**Step 2: Keep chunk membership correct on spawn, move, and removal**

Update spawn, move, and removal code paths so chunk membership reflects current positions for cows, zombies, skeletons, arrows, plants, and fences.

**Step 3: Expose minimal chunk queries needed by env balancing**

Add methods for iterating chunk keys, reading chunk materials, and listing object handles in a chunk.

### Task 3: Port periodic chunk balancing from Python env

**Files:**
- Modify: `/Users/jun/myWorks/crafters/src/lib.rs`

**Step 1: Add the missing every-10-step balancing call in `Env::step`**

Mirror Python `Env.step()` by running chunk balancing after local object updates.

**Step 2: Port `_balance_chunk` target formulas**

Implement the Python spawn/despawn targets for zombie, skeleton, and cow counts as a function of daylight and walkable space.

**Step 3: Port `_balance_object` spawn/despawn behavior**

Match chunk-local material masks, player-distance checks, spawn probabilities, and despawn probabilities.

### Task 4: Verify the port and guard against regressions

**Files:**
- Modify: `/Users/jun/myWorks/crafters/tests/entities.rs`
- Test: `/Users/jun/myWorks/crafters/tests/parity.rs`
- Test: `/Users/jun/myWorks/crafters/tests/worldgen.rs`

**Step 1: Re-run the focused regression tests**

Run: `cargo test --test entities chunk_ -- --nocapture`
Expected: PASS

**Step 2: Re-run existing environment verification**

Run: `cargo test --test entities -- --nocapture`
Expected: PASS

Run: `cargo test --test parity -- --nocapture`
Expected: PASS

Run: `cargo test --test worldgen -- --nocapture`
Expected: PASS

**Step 3: Note remaining non-port gaps if tests still pass but parity risks remain**

Record any residual differences such as RNG implementation or Python hash seeding if they still diverge after balancing.
