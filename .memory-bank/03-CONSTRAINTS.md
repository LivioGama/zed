# Constraints

## Technical Constraints
- Must compile on Rust 2021 edition
- Must support macOS, Linux, and Windows platforms
- GPUI framework is the only UI layer (no web/Electron)
- All UI must be GPU-accelerated for performance
- No blocking operations on the main thread

## Security Policies
- No hardcoded secrets or credentials
- All user input must be sanitized
- Git credentials handled via system keychain or askpass
- External API keys stored in secure credential providers

## Code Standards
- Follow Rust idioms and clippy lints
- Use `cargo fmt` for formatting
- All public APIs require documentation
- Error handling via `Result` types, not panics
- Async code uses Rust's native async/await

## Performance Requirements
- Sub-millisecond latency for editing operations
- Smooth 60fps UI rendering
- Efficient memory usage for large files
- Background processing for expensive operations

## Forbidden Patterns
- `unwrap()` in production code paths (use `?` or proper error handling)
- Blocking the main/UI thread
- Direct filesystem access without async wrappers
- Hardcoded paths or platform-specific assumptions without cfg attributes
- Mutable global state

## Pre-Change Validation
Before committing any change:
- [ ] `cargo check` passes
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied
- [ ] No regressions in existing functionality
- [ ] Tests pass (if applicable)

## Branch-Specific Constraints (split-diff-clean)
- Maintain backwards compatibility with existing diff view
- Side-by-side view must be toggleable
- Collapsed regions must not break revert button positioning
- Synchronized scrolling must be optional
