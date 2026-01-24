# Project Brief

## Mission
Zed is a high-performance, multiplayer code editor built in Rust by the creators of Atom and Tree-sitter, designed to be the fastest and most collaborative code editor available.

## Core Requirements
1. High-performance text editing with native speed
2. Real-time collaboration and multiplayer editing
3. AI-powered coding assistance integration
4. Modern UI with GPU-accelerated rendering (GPUI)
5. Cross-platform support (macOS, Linux, Windows)
6. Extensibility through extensions system

## Success Criteria
- Sub-millisecond latency for common editing operations
- Seamless real-time collaboration without sync conflicts
- Full language server protocol support
- Active extension ecosystem

## User Experience Targets
- Instant startup and file opening
- Responsive UI regardless of file size
- Intuitive multiplayer experience
- Native look and feel on each platform

## Out of Scope
- Web-based editor (tracking issue exists but not current priority)
- Mobile platforms

## Current Branch Focus: `split-diff-clean`
This branch focuses on enhancing the diff viewer with:
- Side-by-side diff view capability
- Synchronized scrolling between diff panes
- Collapsible unchanged regions
- Improved revert button positioning
- Git graph UI operations
