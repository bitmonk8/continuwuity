# Known Issues

## Cross-Platform (xplatform branch)

### Cargo.toml feature set duplication
`xplatform` and `standard` feature sets share 11 features with no common base.
Adding a shared feature requires editing both lists. Extract a `common` base
feature to reduce divergence risk.
**File:** `src/main/Cargo.toml:66-78`

### maximize_fd_limit return type inconsistency
Unix variant returns `Result<(), nix::errno::Errno>`, non-Unix returns
`Result<()>` (`conduwuit_core::Error`). Works now via `.expect()` but will
break if anyone adds typed error handling at the call site.
**File:** `src/core/utils/sys.rs:34`

### Silent ignore of unix_socket_path on Windows
The `unix_socket_path` config field is not platform-gated. Setting it on
Windows silently falls through to TCP serving with no warning. Consider adding
a startup warning or gating the config field.
**File:** `src/router/serve/mod.rs:33-36`

### No tests for cfg(not(unix)) code paths
The restart stub, signal handler, fd limit stub, and storage stubs have no
test coverage. CI `cargo test` on Windows/macOS passes vacuously for these
paths. Add platform-specific unit tests.
**Files:** `src/main/restart.rs`, `src/main/signal.rs`, `src/core/utils/sys.rs`, `src/core/utils/sys/storage.rs`
