# Current State

## Last Updated
2026-02-16 09:58

## Last Session Summary
Reimplemented missing GitDiff workflow controls from the older implementation: next/previous file navigation, revert current file, explicit refresh, whitespace/empty-line filtering, sync-scroll toggle, and word-diff toggle. Also restored platform keybindings and added a `SplittableEditor` sync-scroll enable/disable API used by `ProjectDiff`.

## Active Milestone
**Legacy DiffViewer Reintegration (Separate Path)**: In Progress
- Progress: 90%
- Progress: 96%
- Outcome:
  - ✅ `crates/diff_viewer` now runs the pre-rebase behavior model with current API compatibility fixes
  - ✅ Right pane remains editable and soft-wrap disabled in both panes
  - ✅ Reintroduced old `ProjectDiff` controls and toolbar actions with current API compatibility
  - ✅ Branch diff base now prefers current branch's tracked upstream over default-branch fallback
  - ✅ `cargo check -p git_ui` passes after reintroduction
  - ⚠️ `cargo test -p git_ui` targeted runs blocked by unrelated `fs` test-helper compile break (`RemoteCommandOutput::default()` missing)
  - ⏳ Final manual UX validation in app for restored toggles/navigation and diff filtering behavior

## Immediate Next Steps
1. [ ] Manually validate restored GitDiff controls in app (`crates/git_ui/src/project_diff.rs`): next/prev file, revert current file, refresh, whitespace/empty-line filters, sync-scroll toggle, word-diff toggle
2. [ ] Validate tracked-upstream branch base selection for Branch Diff and fallback behavior when no upstream exists (`crates/git_ui/src/project_diff.rs`)
3. [ ] Verify split sync toggle behavior after disabling/re-enabling across pane focus/scroll interactions (`crates/editor/src/split.rs`)
4. [ ] Decide whether to fix or defer the unrelated `fs` test-helper compile issue before broader `git_ui` test sweeps (`crates/fs/src/fake_git_repo.rs`)
5. [ ] Defer: design right-dock-local multi-chat tabs by making `AgentPanel` pane-backed (or equivalent dock-pane item host) instead of opening tabs in the center workspace panes (`crates/agent_ui/src/agent_panel.rs`, `crates/workspace/src/dock.rs`)

## Known Issues
| ID | Severity | Description | Status |
|----|----------|-------------|--------|
| WARN-001 | Low | Unused variable warnings in git/project crates | Open |

## Working Context
Legacy and current diff implementations now coexist and are both active: Git panel confirm still opens standalone `DiffViewer`, while `ProjectDiff` once again exposes advanced old-style workflow controls and diff filtering toggles.
