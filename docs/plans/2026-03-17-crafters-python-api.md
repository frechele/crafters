# Crafters Python API Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a PyO3-backed Python package whose public API is close enough to the original `crafter` package that most callers can switch `from crafter import ...` to `from crafters import ...`.

**Architecture:** Keep the existing Rust environment as the source of truth and expose it through a small PyO3 extension module. Put the Crafter-compatible constructor, spaces, gym registration, and recorder wrappers in a mixed Python package so compatibility logic stays easy to change without disturbing the Rust core.

**Tech Stack:** Rust 2024, PyO3, numpy crate, maturin mixed project layout, Python `unittest`

---

### Task 1: Lock The Python Surface With Failing Tests

**Files:**
- Create: `python_tests/test_env_api.py`

**Step 1: Write the failing test**

Add tests that expect:
- `from crafters import Env, Recorder` succeeds
- `Env()` exposes `action_names`, `action_space`, `observation_space`, `reset()`, `step()`, and `render()`
- `step()` returns `(obs, reward, done, info)` with Crafter-style keys

**Step 2: Run test to verify it fails**

Run: `python3 -m unittest discover -s python_tests -v`
Expected: FAIL with `ModuleNotFoundError: No module named 'crafters'`

### Task 2: Add Mixed Python Package And PyO3 Extension

**Files:**
- Modify: `Cargo.toml`
- Create: `pyproject.toml`
- Create: `python/crafters/__init__.py`
- Create: `python/crafters/env.py`
- Create: `python/crafters/recorder.py`
- Create: `src/python_api.rs`
- Modify: `src/lib.rs`

**Step 1: Write minimal implementation**

Expose a `_core` extension that wraps the Rust `Env` and converts frames and semantic grids into NumPy arrays. Keep Crafter-compatible constructor normalization, spaces, and gym registration in Python.

**Step 2: Run targeted verification**

Run: `python3 -m unittest discover -s python_tests -v`
Expected: PASS

### Task 3: Verify Packaging And Document Usage

**Files:**
- Modify: `README.md`

**Step 1: Verify editable install and runtime imports**

Run:
- `python3 -m pip install -e .`
- `python3 -c "from crafters import Env; env = Env(seed=0); print(env.reset().shape)"`

**Step 2: Run full verification**

Run:
- `cargo test`
- `python3 -m unittest discover -s python_tests -v`
