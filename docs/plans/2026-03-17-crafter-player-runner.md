# Crafter Player Runner Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a local executable that lets a human play the Rust Crafter environment in a desktop window.

**Architecture:** Keep the environment library unchanged and add a thin runner layer on top. Put pure, testable helpers for input mapping and frame conversion in library code, then use a minimal desktop window crate to poll keys, step the env, and blit the rendered frame.

**Tech Stack:** Rust 2024, `cargo test`, `cargo run --bin play`, `minifb`

---

### Task 1: Runner Helper Contracts

**Files:**
- Create: `tests/player_runner.rs`
- Create: `src/runner.rs`
- Modify: `src/lib.rs`

**Step 1: Write the failing test**

Add tests for:
- mapping keyboard state to a single Crafter action
- converting `Frame` RGB bytes into a window buffer format

**Step 2: Run test to verify it fails**

Run: `cargo test --test player_runner`
Expected: FAIL because runner helpers do not exist.

**Step 3: Write minimal implementation**

Add pure helper functions and types that the binary can call without depending on the window crate.

**Step 4: Run test to verify it passes**

Run: `cargo test --test player_runner`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/player_runner.rs src/runner.rs src/lib.rs
git commit -m "feat: add player runner helpers"
```

### Task 2: Play Binary

**Files:**
- Create: `src/bin/play.rs`
- Modify: `Cargo.toml`
- Modify: `src/runner.rs`

**Step 1: Write the failing test**

Add a compile-oriented test or command-level verification target that depends on the play binary’s helper API shape.

**Step 2: Run test to verify it fails**

Run: `cargo test --test player_runner`
Expected: FAIL because the binary integration path is incomplete.

**Step 3: Write minimal implementation**

Open a window, map key presses to actions, step the env, redraw each frame, and reset after episode end.

**Step 4: Run test to verify it passes**

Run: `cargo test --test player_runner`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml src/bin/play.rs src/runner.rs tests/player_runner.rs
git commit -m "feat: add playable crafter runner"
```

### Task 3: Final Verification

**Files:**
- Modify: `README.md`

**Step 1: Write the failing test**

No extra automated test; verify via build and manual launch command.

**Step 2: Run test to verify it fails**

Run: `cargo run --bin play`
Expected: Launch path exists only after implementation.

**Step 3: Write minimal implementation**

Document controls and expected run command.

**Step 4: Run test to verify it passes**

Run: `cargo test && cargo run --bin play`
Expected: Tests pass and the window opens.

**Step 5: Commit**

```bash
git add README.md
git commit -m "docs: add crafter player controls"
```
