# Workflows

## Development Workflow
1. Read current state from Memory Bank
2. Check branch status with `git status`
3. Implement changes
4. Run `cargo check` and `cargo clippy`
5. Test changes locally
6. Log changes to changelog
7. Update current state

## Build Commands

### Check (fast compilation check)
```bash
cargo check
```

### Build Debug
```bash
cargo build
```

### Build Release
```bash
cargo build --release
```

### Run Zed (Debug)
```bash
cargo run
```

### Run Zed (Release)
```bash
cargo run --release
```

## Testing

### Run All Tests
```bash
cargo test
```

### Run Tests for Specific Crate
```bash
cargo test -p diff_viewer
cargo test -p git_ui
```

### Run Specific Test
```bash
cargo test -p <crate_name> <test_name>
```

## Linting & Formatting

### Check Formatting
```bash
cargo fmt --check
```

### Apply Formatting
```bash
cargo fmt
```

### Run Clippy
```bash
cargo clippy --workspace
```

## Git Workflow

### Feature Branch
```bash
git checkout -b feat/description
# make changes
git add -p
git commit -m "feat: description"
git push origin feat/description
```

### Sync with Main
```bash
git fetch origin main
git rebase origin/main
```

## Common Tasks

### Add Dependency to Crate
```bash
cd crates/<crate_name>
cargo add <dependency>
```

### Check Specific Crate
```bash
cargo check -p diff_viewer
cargo check -p git_ui
```

### View Changes in Diff Viewer Crate
```bash
git diff crates/diff_viewer/
```

### Run with Logging
```bash
RUST_LOG=debug cargo run
```
