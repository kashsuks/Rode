# AGENTS.md for Rode

You are an advanced Rust developer that always puts efficiency and stability as your priority.

**Tech Stack:** Rust 2021/2024, [key crates like tokio, serde, clap, anyhow]

## Quick Start

```bash
# Build and test
cargo build
cargo test
cargo clippy
cargo fmt

# Run
cargo run -- [args]

# Run with test data
cargo run -- tests/fixtures/example.txt
```

## File Structure

```
project/
├── src/
│   ├── main.rs          # Binary entry point
│   ├── lib.rs           # Library root
│   ├── module.rs        # Modules
│   └── utils.rs
├── tests/               # Integration tests
│   └── common/          # Test utilities
├── benches/             # Benchmarks
├── examples/            # Usage examples
├── Cargo.toml           # Manifest
└── Cargo.lock           # Locked dependencies
```

**Module responsibilities:**
- `main.rs` - CLI, orchestration, error handling
- `lib.rs` - Public API
- `[module].rs` - Feature implementations

## Common Commands

```bash
# Build
cargo build                      # Debug
cargo build --release            # Release (optimized)
cargo check                      # Fast compile check
cargo check --all-targets        # Check everything

# Test
cargo test                       # All tests
cargo test -- --nocapture        # Show output
cargo test pattern               # Match pattern
cargo test --release             # With optimizations

# Code quality
cargo clippy                     # Lint
cargo clippy --all-targets       # Include tests
cargo clippy -- -D warnings      # Deny all warnings
cargo fmt                        # Format
cargo fmt -- --check             # Check format only

# Dependencies
cargo add [crate]                # Add dependency
cargo add --dev [crate]          # Dev dependency
cargo add tokio --features full  # With features
cargo update                     # Update deps

# Documentation
cargo doc --open                 # Generate and view docs

# Full cycle
cargo fmt && cargo clippy -- -D warnings && cargo test && cargo build --release
```

## Code Style

- **Style:** Official Rust style (enforced by rustfmt)
- **Format:** `cargo fmt`
- **Lint:** `cargo clippy -- -D warnings`
- **Max line length:** 100 characters (default)

**Naming:**
- Functions/variables: `snake_case`
- Types/traits: `PascalCase`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

## Common Patterns

**Error handling (applications):**
```rust
use anyhow::{Context, Result};

fn process(path: &str) -> Result<Data> {
    let file = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read: {}", path))?;
    
    let data = parse(&file)
        .context("Failed to parse")?;
    
    Ok(data)
}
```

**Error handling (libraries):**
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    
    #[error("IO error")]
    Io(#[from] std::io::Error),
}
```

**Tests:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function() {
        assert_eq!(function("input"), expected);
    }
    
    #[test]
    #[should_panic(expected = "error")]
    fn test_panic() {
        function_that_panics();
    }
}
```

**Documentation:**
```rust
/// Brief description.
///
/// # Arguments
///
/// * `x` - Description
///
/// # Returns
///
/// Description
///
/// # Errors
///
/// When errors occur
///
/// # Examples
///
/// ```
/// let result = function("input");
/// ```
pub fn function(x: &str) -> Result<String> {
    Ok(x.to_string())
}
```

## Development Workflow

**Commits:** Conventional format (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `perf:`)

**Branching:** `feature/[name]`, `fix/[name]`, `refactor/[name]`

**PR Checklist:**
- [ ] Code compiles: `cargo build`
- [ ] Tests pass: `cargo test`
- [ ] No clippy warnings: `cargo clippy -- -D warnings`
- [ ] Formatted: `cargo fmt -- --check`
- [ ] Documentation builds: `cargo doc --no-deps`
- [ ] No `unwrap()` in production code

## Coding Conventions

- Use `?` operator for error propagation (not `unwrap()`)
- Prefer immutability (`let` over `let mut`)
- Use iterators over loops when appropriate
- Keep functions small and focused
- Leverage type system for safety
- Add context to errors with `.context()`
- [Add project-specific conventions]

## Performance

- Use `--release` for benchmarking
- Profile with `cargo flamegraph`

**Cargo.toml optimization:**
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

## Project-Specific Notes

[Add special instructions, test data generation, deployment, etc.]
