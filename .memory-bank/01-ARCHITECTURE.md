# Architecture

## Tech Stack
| Layer | Technology | Version |
|-------|------------|---------|
| Language | Rust | 2021 edition |
| UI Framework | GPUI (custom GPU-accelerated) | Internal |
| Build System | Cargo | Workspace |
| Testing | Rust test + cargo test | Built-in |
| Platform | macOS, Linux, Windows | Native |
| Collaboration | WebRTC + Collab Server | Custom |

## Directory Structure
```
zed/
├── assets/          # Static assets, keymaps, themes
├── crates/          # 211 Rust crates (workspace members)
│   ├── agent*/      # AI agent integration
│   ├── buffer_diff/ # Buffer diffing algorithms
│   ├── diff_viewer/ # Diff visualization UI
│   ├── editor/      # Core editor functionality
│   ├── git_ui/      # Git panel and UI components
│   ├── gpui/        # GPU-accelerated UI framework
│   ├── language/    # Language server protocol
│   ├── project/     # Project management
│   ├── workspace/   # Window/workspace management
│   └── zed/         # Main application crate
├── docs/            # Documentation
├── extensions/      # Built-in extensions
├── script/          # Build and CI scripts
└── tooling/         # Development tools
```

## Design Patterns
- **Entity-Component System**: GPUI uses an ECS-like model for UI state
- **Actor Model**: Async message passing between components
- **Workspace/Pane/Item**: Hierarchical UI organization
- **Model-View pattern**: Separation of data models from rendering

## Key Components (Current Branch Focus)

### Diff Viewer (`crates/diff_viewer/`)
- `viewer.rs` - Main diff viewer component with side-by-side support
- `connector.rs` / `connector_builder.rs` - Visual connectors between diff sections
- `rendering/revert_buttons.rs` - Inline revert functionality
- `constants.rs` - Shared constants for rendering

### Git UI (`crates/git_ui/`)
- `git_panel.rs` - Main Git panel with status, staging, commits
- `project_diff.rs` - Project-wide diff display
- `file_diff_view.rs` - Single file diff rendering
- `commit_view.rs` - Commit details view

## Integration Points
- Language Server Protocol (LSP) for language features
- Tree-sitter for syntax parsing
- Git2 library for Git operations
- WebRTC for collaboration
- OpenAI/Anthropic APIs for AI features
