// =============================================================================
// Code Style Rules — Enforced at compile time
//
// These lints implement the rules defined in .claude/rules/code-style.md.
// CI runs: cargo clippy -- -D warnings (zero tolerance)
//
// Rule mapping:
//   - Error handling        → clippy::unwrap_used, clippy::expect_used, clippy::panic
//   - Security              → clippy::todo, clippy::unimplemented
//   - Dead code             → unused_must_use, unused_imports, dead_code
//   - Code quality          → clippy.toml thresholds (cognitive complexity, fn size)
// =============================================================================

// ---- Deny: violations fail the build ----
#![deny(
    // Error handling: no unwrap/expect/panic in production code
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    // Security: no unfinished code in production
    clippy::unimplemented,
    clippy::todo,
    // Dead code and unused
    unused_must_use,
    unused_imports,
    dead_code
)]
// ---- Allow: intentional exceptions ----
#![allow(
    // async_trait and DDD naming triggers this
    clippy::module_name_repetitions,
    // Handler signatures have similar param names
    clippy::similar_names
)]

pub mod application;
pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod shared;
