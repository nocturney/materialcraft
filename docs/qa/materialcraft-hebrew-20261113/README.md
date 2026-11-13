# MaterialCraft Hebrew — isolated combined native WGPU evidence (2026-11-13)

This branch contains **evidence only**; it does not modify either original upstream PR.

**Exact inputs:**
- [MaterialCraft Hebrew translation/UI PR #197](https://github.com/storytold/materialcraft/pull/197) exact captured head: `b362ce04c5154cff311e8fda0f6dce036f60a628`.
- [MaterialCraft Hebrew/Arabic font fallback PR #198](https://github.com/storytold/materialcraft/pull/198) exact captured head: `9b7f336a74dea1675bb12fe074b7583e55bf7173`.
- The isolated integration checkout is based on #197 and uses only the **verbatim** `crates/ui-egui/src/fonts.rs` from #198; source and copied font-file SHA256 both `3c0bb0eadf1538bc9b3b00dc53a889ac2b17aa43e21c91c1c3e4d77`. This is **not** a single merged upstream source head.

**Test environment:** MacMiniOffice2 Apple Silicon macOS, isolated local Cargo build/cache and a temporary scratch-only document. No original user documents or installed app state were modified.

**Executed tests on this combined worktree:**
- `cargo fmt --all -- --check`: PASS.
- `cargo check -p materialcraft-ui-egui --tests --locked`: PASS.
- `cargo test -p materialcraft-ui-egui --test hebrew_native_smoke --locked -- --nocapture`: PASS, 2/2 executable tests with one intentionally ignored offscreen-GPU test; includes actual Hebrew save/reopen using a fresh `MaterialCraftApp` reader.
- `MC_SNAPSHOT_DIR=... cargo test -p materialcraft-ui-egui --test hebrew_native_smoke --locked -- --ignored --nocapture`: PASS, 1/1 explicit opt-in WGPU test. It wrote the [actual 1440×900 screenshot](screenshots/materialcraft-hebrew-native-rtl.png).

**Visual observation:** The rendered application shows readable Hebrew in major menus, panels and controls, functional left-library/right-property organization, canvas and 3D preview; no obvious missing-glyph boxes in the visible screenshot. **Not accepted as complete:** many technical node names and library items remain English; mixed-language alignment, text overlap, all native dialogs, responsive layouts, macOS/Windows/Linux platform-specific behavior, screen reader and complete native-speaker terminology coverage still require review. A PNG's existence and size alone do not validate all typography.

The upstream CI and SonarCloud checks may require the source project's approval of external contributor workflows. No maintainer approval, merge, release or full visual verification is claimed.

The exact screenshot SHA256 is recorded in [SHA256SUMS](SHA256SUMS).
