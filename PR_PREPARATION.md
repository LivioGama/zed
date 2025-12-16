# Git Diff Viewer: Side-by-Side Implementation

## Overview

This PR introduces a side-by-side diff viewer implementation using GPUI, addressing a highly-voted community request.
The implementation provides fundamental diff visualization capabilities with line highlighting and connector curves
between related code blocks.

**Context**: As per [community discussion #26770](https://github.com/zed-industries/zed/discussions/26770?sort=old),
this feature addresses a gap in Zed's Git
workflow. [Max Brunsfeld offered assistance](https://github.com/zed-industries/zed/discussions/26770?sort=old#discussioncomment-14557428)
for this implementation.

## Why This Matters

Developers need visual diff tools to:

- Review changes before committing
- Understand code evolution during code review
- Resolve merge conflicts effectively
- Some personal preference is the side-by-side view, which is still acceptable compared to preferring tabs to spaces.

This does not replace but proposes an alternative and persisted preference to how the user sees the Git changes
meaningfully for them. This is inspired by some other IDEs.

## Implementation Details

### Architecture

The diff viewer leverages existing Zed crates:

- **`gpui`**: Dual `Editor` components for side-by-side rendering with custom paint layers for connectors
- **`editor`**: Reuses syntax highlighting and line numbering infrastructure
- **`git`**: Integrates with existing Git state management
- **`theme`**: Consistent diff colors across all Zed themes
- **`diff_viewer`**: New crate containing diff-specific logic

### Diff Algorithm

Uses [Imara diff](https://github.com/pascalkuthe/imara-diff), a Rust implementation that provides:

- Line-level diff computation
- Block alignment for visual clarity
- Performance optimization (achieved 148% speedup over naive implementation)

### Key Features Implemented

- ✅ Side-by-side diff visualization using dual GPUI `Editor` components
- ✅ Diff computation using Imara diff algorithm
- ✅ Line highlighting for additions, deletions, and modifications
- ✅ Connector curves with Bézier curve rendering between related code blocks
- ✅ Scroll synchronization between left and right editors
- ✅ Crushed blocks for collapsed regions with visual indicators
- ✅ Theme integration with Zed's color system
- ✅ Syntax highlighting copying language from source buffers (matching unified viewer behavior)

### Missing Core Features

- [ ] Jump to file in workspace (keyboard shortcut: `cmd+↓`)
- [ ] Partial revert UI for individual change fragments
- [ ] Collapse/expand unchanged code fragments
- [ ] Word-level diff highlighting within lines
- [ ] Ignore whitespace options:
    - Do not ignore
    - Trim whitespaces
    - Ignore all whitespaces
    - Ignore whitespaces and empty lines
    - Ignore formatting changes
- [ ] Toggle scroll synchronization on/off

### Advanced Features (Future)

- [ ] Best git diff feature ever: three-way merge view for conflict resolution
- [ ] Partial staging (include specific fragments in commit)
- [ ] Infinite scroll for multi-file diff review
- [ ] Support for non-textual files (images, binary diffs)
- [ ] Visual indicator in scrollbar for changed regions

### Minor / Nice-to-Have

- [ ] Line numbers shall display white within highlighted blocks

## Testing

**Current test coverage:**

- [x] Basic diff computation
- [x] Line alignment logic
- [x] Theme color integration
- [x] Language copying from source buffers and syntax highlighting
- [x] Scrollbar positioning
- [ ] Scroll synchronization edge cases
- [ ] Large file performance
- [ ] Unicode and multi-byte character handling

## Alignment with Contributing Guidelines

Per [CONTRIBUTING.md](https://github.com/zed-industries/zed/blob/main/CONTRIBUTING.md):

✅ **Fixes existing bugs and issues**:
Addresses [discussion #26770](https://github.com/zed-industries/zed/discussions/26770?sort=old)
✅ **Includes tests**: Core diff logic is tested
✅ **UI changes documented**: Screenshots/recordings needed (see below)
⚠️ **Early PR for larger change**: Seeking feedback on architecture and implementation
❌ **Not AI-generated**: Well, that's a bummer. I don't know Rust, but I'm still a developer, and an engineer.\*
✅ **Works with existing crates**: Integrates with `gpui`, `editor`, `git`, `theme`

\*I would be surprised if the code was closer to complete crap rather than a good and perfectible implementation. I read it and prettified it to the maximum I could with the minimum of changes possible (I didn't even fixed the few existing warnings) so it looks like it was written by a human. Happy to have your thoughts.

## Screenshots / Recordings

## Request for Feedback

This PR is opened early for guidance on:

1. **Architecture**: Is the approach of dual `Editor` components aligned with Zed's vision?
2. **GPUI patterns**: Are the custom paint layers for connectors implemented correctly?
3. **Performance**: Any concerns with large file diffs?
4. **UX**: Feedback on visual design decisions (colors, spacing, interactions)
5. **Scope**: Should any features be split into follow-up PRs?

## Related Documentation

- [CONTRIBUTING.md](https://github.com/zed-industries/zed/blob/main/CONTRIBUTING.md) - Contribution guidelines
- [CODE_REVIEW_SUMMARY.md](./CODE_REVIEW_SUMMARY.md) - Detailed file-by-file changes (to be removed before merge)
- [Zed Glossary](docs/src/development/glossary.md) - Codebase terminology reference

## Next Steps

1. Fix critical bugs listed above
2. [Book a call](https://cal.com/maxbrunsfeld/60-minute-pairing) with Max
3. Incorporate maintainer feedback
4. Complete missing core features or split into follow-up PRs

---

**Note**: This is a work-in-progress PR. Community contributions and feedback are welcome to help evolve this
implementation into a production-ready feature. Also I am about to disappear for a whole month for holiday, feel free to
move on without me, finally happy I am not alone anymore \o/
