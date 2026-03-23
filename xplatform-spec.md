# Cross-Platform Support — continuwuity

**Branch:** `xplatform`
**Status:** Implemented
**Date:** 2026-03-23

## Overview

continuwuity compiles, runs, and passes tests on **macOS** and **Windows** in
addition to Linux. The codebase uses a single source tree with platform-specific
code gated behind `cfg` attributes. Linux functionality is unchanged.

## Platform Feature Matrix

| Feature | Linux | macOS | Windows |
|---|---|---|---|
| TCP listener | yes | yes | yes |
| Unix socket listener | yes | yes | no |
| Signal handling (graceful shutdown) | full | full | Ctrl-C only |
| SIGUSR1/2 (hot reload / admin cmd) | yes | yes | no (use admin room) |
| Process restart (exec) | yes | yes | no (manual restart) |
| jemalloc | yes | yes (opt-in) | no (system allocator) |
| hardened_malloc | yes | no | no |
| io_uring | yes | no | no |
| journald | yes | no | no |
| systemd notify | yes | no | no |
| FD limit raising | yes | yes | no-op |
| LDAP | yes | yes | yes |
| RocksDB | yes | yes | yes |

## Conditional Compilation Gates

| File | Gate | Purpose |
|---|---|---|
| `src/main/signal.rs` | `#[cfg(unix)]` / `#[cfg(not(unix))]` | Unix signals vs Ctrl-C only |
| `src/main/restart.rs` | `#[cfg(unix)]` / `#[cfg(not(unix))]` | exec restart vs error+exit |
| `src/core/utils/sys.rs` | `#[cfg(unix)]` / `#[cfg(not(unix))]` | FD limit raising vs no-op |
| `src/core/utils/sys/storage.rs` | `#[cfg(target_os = "linux")]` | Block device detection via sysfs |
| `src/main/logging.rs` | `#[cfg(all(target_os = "linux", feature = "journald"))]` | journald integration |
| `src/router/serve/mod.rs` | `#[cfg(unix)]` | Unix socket listener |
| `src/main/Cargo.toml` | `standard` vs `xplatform` feature sets | Platform-appropriate dependencies |

## Feature Sets

- **`standard`** (Linux): includes `jemalloc`, `io_uring`, `journald`, `systemd`
- **`xplatform`** (macOS/Windows): excludes the above Linux-specific features

## What Was Done

### Build system
- CI matrix in `.github/workflows/xplatform.yml` runs `check`, `test`, and
  `clippy` on Ubuntu, macOS, and Windows.
- Linux-only Cargo features gated behind the `standard` feature set.
- RocksDB builds on all three platforms. Windows requires LLVM (installed via
  choco in CI).

### Signal handling
- Unix: full signal support (SIGTERM, SIGQUIT, SIGUSR1 for config reload,
  SIGUSR2 for admin commands, Ctrl-C).
- Windows: Ctrl-C triggers graceful shutdown. Config reload available via
  `!server reload_config` admin room command.

### Unix socket listener
- Gated with `#[cfg(unix)]`. Works on Linux and macOS.
- Windows: silently skipped if `unix_socket_path` is configured.

### Process restart
- Unix: `Command::exec()` with deleted-binary safety check.
- Windows: returns error with message to restart manually.

### FD limit
- Unix: raises `RLIMIT_NOFILE` from soft to hard limit via `nix::sys::resource`.
- Windows: no-op returning `Ok(())`.

### Logging
- Journald gated to `#[cfg(all(target_os = "linux", feature = "journald"))]`.
- Console logging works on all platforms.

### Memory allocator
- jemalloc available on Linux and macOS (opt-in via feature).
- Windows (MSVC): system allocator.

### Documentation
- `docs/deploying/windows.mdx` — build instructions, LLVM setup, feature
  support matrix, troubleshooting.
- `docs/deploying/macos.mdx` — build instructions, optional jemalloc.
- `docs/ISSUES.md` — known cross-platform issues.

## Resolved Questions

| Question | Resolution |
|---|---|
| Windows hot-reload | Use `!server reload_config` admin room command |
| Windows restart | Disabled; returns error with manual restart guidance |
| LDAP on Windows | Supported via `ldap3` crate (no system deps) |
| CI runners | GitHub Actions (Ubuntu, macOS, Windows) |
| RocksDB on Windows | Works with LLVM installed for bindgen |

## Known Issues

See `docs/ISSUES.md` for the full list. Summary:

1. `unix_socket_path` config silently ignored on Windows (no warning emitted).
2. `maximize_fd_limit()` return type inconsistency between Unix (`nix::errno`)
   and non-Unix (`conduwuit_core::Error`).
3. `xplatform` and `standard` feature sets duplicate 11 common features — both
   must be edited when adding shared features.
4. No unit tests for `cfg(not(unix))` code paths (restart stub, signal handler,
   FD limit stub, storage stubs).

## Out of Scope (unchanged)

- Native Windows service integration (NSSM/WinSW recommended in docs)
- macOS launchd plist / Homebrew formula
- GUI or tray icon
- Performance parity (Linux remains the optimised target)
- Official release binaries for macOS/Windows
