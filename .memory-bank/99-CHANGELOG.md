# Changelog

Format: `[YYYY-MM-DD HH:MM] ACTION: path - description`

**Actions**: CREATE, MODIFY, DELETE, RENAME, REFACTOR, SESSION

---

## Entries

[2026-01-24 21:09] MODIFY: crates/git_graph/src/git_graph.rs - Fixed modifier check to use control and platform fields instead of non-existent ctrl and cmd
[2026-01-24 21:08] MODIFY: crates/git_graph/src/git_graph.rs - Fixed multi-select condition to use Ctrl/Cmd modifiers instead of secondary for proper Ctrl/Cmd+Click multi-select functionality
[2026-01-24 21:07] MODIFY: crates/git_graph/src/git_graph.rs - Fixed lifetime issues in render method by using weak_self.update pattern for row click handlers instead of cx.listener, and cloned captured variables to ensure closure is 'static
[2026-01-24 20:11] MODIFY: crates/git_graph/src/git_graph.rs - Fixed remaining compilation errors: removed incorrect as_ref() call on CommitDataState pattern matching, resolved lifetime issues in async closures by cloning selected_branches before spawn, and fixed context shadowing in map_row closure by using outer cx for listener calls
[2026-01-24 19:40] MODIFY: crates/git_graph/src/git_graph.rs - Fixed compilation errors: added missing gpui imports (Point, DismissEvent, FontWeight, ClipboardItem, ScrollWheelEvent, Corner), corrected closure signatures for listeners and map_row, fixed CommitDataState matching, resolved async spawn signatures, and addressed type mismatches in UI handlers
[2026-01-24 19:19] MODIFY: crates/git_graph/src/git_graph.rs - Fixed compilation errors: updated import to use CommitFile instead of FileDiff, renamed variables to avoid shadowing CommitDataState, resolved lifetime issues in async tasks by using weak_self instead of capturing self, corrected listener and map_row closure signatures
[2026-01-24 18:17] MODIFY: crates/git/src/repository.rs - Modified cherry_pick and revert_commits to detect conflicts by checking for CHERRY_PICK_HEAD and REVERT_HEAD files
[2026-01-24 18:17] MODIFY: crates/git/src/repository.rs - Added abort_cherry_pick, continue_cherry_pick, abort_revert, continue_revert methods to GitRepository trait and RealGitRepository implementation
[2026-01-24 18:17] MODIFY: crates/fs/src/fake_git_repo.rs - Added abort_cherry_pick, continue_cherry_pick, abort_revert, continue_revert method stubs to FakeGitRepository
[2026-01-24 18:17] MODIFY: crates/git_graph/src/git_modals.rs - Added CherryPickConflict and RevertConflict to ModalAction with render methods and Abort/Continue buttons
[2026-01-24 18:17] MODIFY: crates/git_graph/src/git_graph.rs - Modified cherry_pick and revert_commits to handle conflict errors by showing conflict modals
[2026-01-24 18:17] MODIFY: crates/git_graph/src/git_graph.rs - Added handle_cherry_pick_conflict and handle_revert_conflict methods to call abort/continue operations
[2026-01-24 18:55] MODIFY: crates/git_graph/src/git_graph.rs - Fixed compilation errors: removed unused imports, corrected data matching for Arc<CommitDataState>, fixed scrolling method calls, updated processor closure, and resolved method signature mismatches
[2026-01-24 19:00] MODIFY: crates/git_graph/src/git_graph.rs - Implemented file tree view in commit details panel, replacing flat file list with hierarchical tree structure showing directories and files
[2026-01-24 19:15] SESSION: End - Completed implementation of all Git Graph features from GIT_GRAPH_STATUS.md, including file tree view, conflict handling for cherry-pick and revert, and fixed compilation errors
[2026-01-24 17:45] MODIFY: crates/git_graph/src/git_graph.rs - Implemented proper uncommitted changes detection in checkout_branch using repository.status
[2026-01-24 17:43] MODIFY: crates/git/src/repository.rs - Added revert_commits method to GitRepository trait and implemented in RealGitRepository
[2026-01-24 17:43] MODIFY: crates/fs/src/fake_git_repo.rs - Added revert_commits method stub to FakeGitRepository
[2026-01-24 17:43] MODIFY: crates/git_graph/src/git_modals.rs - Added RevertCommits to ModalAction and render_revert_commits method
[2026-01-24 17:43] MODIFY: crates/git_graph/src/git_graph.rs - Added RevertCommits action, context menu entry, modal, and handlers in GitGraph UI
[2026-01-24 17:33] MODIFY: crates/git/src/repository.rs - Added cherry_pick method to GitRepository trait and implemented in RealGitRepository
[2026-01-24 17:33] MODIFY: crates/fs/src/fake_git_repo.rs - Added cherry_pick method stub to FakeGitRepository
[2026-01-24 17:33] MODIFY: crates/git_graph/src/git_graph.rs - Added CherryPick action, context menu entry, and handler in GitGraph UI
[2026-01-24 19:00] MODIFY: crates/git/src/repository.rs - Add edit_commits method to GitRepository trait and implement in RealGitRepository
[2026-01-24 19:00] MODIFY: crates/fs/src/fake_git_repo.rs - Add edit_commits method to FakeGitRepository
[2026-01-24 19:00] MODIFY: crates/git_graph/src/git_graph.rs - Fix perform_edit_amend_commit to use reword_commits for message-only changes and edit_commits for full editing
[2026-01-24 18:30] MODIFY: crates/git_graph/src/git_modals.rs - Update ModalAction and GitModal to support editable messages for SquashCommits and RewordCommits
[2026-01-24 18:30] MODIFY: crates/git_graph/src/git_graph.rs - Fix squash_commits and reword_commits modal callbacks to properly handle edited commit messages
[2026-01-24 18:00] MODIFY: crates/git/src/repository.rs - Add drop_commits and reword_commits methods to GitRepository trait
[2026-01-24 18:00] MODIFY: crates/git/src/repository.rs - Implement drop_commits and reword_commits in RealGitRepository
[2026-01-24 18:00] MODIFY: crates/fs/src/fake_git_repo.rs - Add drop_commits and reword_commits to FakeGitRepository
[2026-01-24 18:00] MODIFY: crates/git_graph/src/git_graph.rs - Fix syntax errors, hook up delete_branch modal, and fix add_item_to_active_pane call
[2026-01-24 17:11] MODIFY: crates/git_graph/src/git_graph.rs - Implement multi-select commits with Ctrl/Cmd+Click and keyboard navigation with arrow keys
[2026-01-24 17:11] MODIFY: assets/keymaps/default-macos.json - Add Git Graph keyboard bindings for navigation
[2026-01-24 17:11] MODIFY: assets/keymaps/default-linux.json - Add Git Graph keyboard bindings for navigation
[2026-01-24 17:11] MODIFY: assets/keymaps/default-windows.json - Add Git Graph keyboard bindings for navigation
[2026-01-24 17:13] MODIFY: crates/git_graph/src/git_graph.rs - Implement drop commit functionality with modal confirmation
[2026-01-24 17:12] MODIFY: crates/git_graph/src/git_graph.rs - Add multi-select branches with Ctrl/Cmd+Click and visual selection indicators
[2025-01-23 12:00] CREATE: .memory-bank/ - Initialized Memory Bank for Zed project
[2025-01-23 12:00] CREATE: .memory-bank/00-PROJECT-BRIEF.md - Project mission and requirements
[2025-01-23 12:00] CREATE: .memory-bank/01-ARCHITECTURE.md - Tech stack and structure
[2025-01-23 12:00] CREATE: .memory-bank/02-CURRENT-STATE.md - Current branch state (split-diff-clean)
[2025-01-23 12:00] CREATE: .memory-bank/03-CONSTRAINTS.md - Technical and code constraints
[2025-01-23 12:00] CREATE: .memory-bank/04-WORKFLOWS.md - Build, test, and dev workflows
[2025-01-23 12:00] CREATE: .memory-bank/99-CHANGELOG.md - This changelog

---

## Pre-Memory Bank History (from git log)

Recent commits on `split-diff-clean` branch:
- `02fad6b` - Merge remote changes and resolve conflicts
- `bd5876d` - fix: apply collapsed region offset to revert button positioning
- `dbcb536` - fix: restore collapsed region positioning for revert buttons and connectors
- `eaf366a` - feat: integrate collapse, sync scroll, and side-by-side features
- `d6c00223` - feat: complete side-by-side diff viewer integration
- `264f2178` - feat: integrate side-by-side diff viewer with toggle action
- `fb242e87` - fix: remove missing ToggleCollapseUnchanged keybinding
- `ed711b45` - fix: resolve API incompatibilities after rebase
- `8bfef9d7` - feat: Add comprehensive Git graph UI operations
[2026-02-15 04:56] MODIFY: crates/git_ui/src/project_diff.rs - Added next/previous diff file actions, revert-current-file action, toolbar controls, and branch-diff base selection preferring current upstream before default branch
[2026-02-15 04:56] MODIFY: assets/keymaps/default-macos.json - Added GitDiff keybindings for previous/next diff file navigation
[2026-02-15 04:56] MODIFY: assets/keymaps/default-linux.json - Added GitDiff keybindings for previous/next diff file navigation
[2026-02-15 04:56] MODIFY: assets/keymaps/default-windows.json - Added GitDiff keybindings for previous/next diff file navigation

[2026-02-15 05:44] MODIFY: crates/git_graph/src/git_graph.rs - Fixed P0 Git Graph stability issues: corrected workspace wiring in OpenGitGraph, added missing action registrations (SelectPreviousCommit, PullWithStash, DeleteBranch), replaced duplicate CherryPick registration, switched multi-select to Ctrl/Cmd modifiers, wired cherry-pick/revert conflict modals, and normalized graph_data references
[2026-02-15 05:48] MODIFY: crates/git_graph/src/git_graph.rs - Reworked commit details panel to support per-file commit diff browsing with selectable file list, before/after text panes, binary file messaging, and selection state reset handling
[2026-02-15 05:49] MODIFY: crates/git_graph/src/git_modals.rs - Improved conflict modal UX by showing only Abort/Continue controls for cherry-pick and revert conflict actions
[2026-02-15 05:50] MODIFY: crates/git_graph/src/git_graph.rs - Replaced multi-branch context menu bulk checkout placeholder with a functional selected-branch checkout action
[2026-02-15 05:52] MODIFY: crates/git_ui/src/project_diff.rs - Added refresh + diff-filter actions (ignore whitespace/empty lines), toolbar controls, and git-panel refresh integration for project/branch diff views
[2026-02-15 05:52] MODIFY: crates/git_ui/src/git_panel.rs - Added request_refresh API to trigger debounced git panel entry refresh from project diff actions
[2026-02-15 05:52] MODIFY: assets/keymaps/default-macos.json - Added GitDiff bindings for refresh and diff-filter toggles (F5, Alt-W, Alt-E)
[2026-02-15 05:52] MODIFY: assets/keymaps/default-linux.json - Added GitDiff bindings for refresh and diff-filter toggles (F5, Alt-W, Alt-E)
[2026-02-15 05:52] MODIFY: assets/keymaps/default-windows.json - Added GitDiff bindings for refresh and diff-filter toggles (F5, Alt-W, Alt-E)
[2026-02-15 05:52] MODIFY: crates/git_graph/src/git_graph.rs - Applied local rustfmt (skip_children) during inspection of unexpected modifications
[2026-02-15 05:56] MODIFY: crates/git_graph/src/git_modals.rs - Added UncommitHead modal action and confirmation UI for uncommit flow
[2026-02-15 05:56] MODIFY: crates/git_graph/src/git_graph.rs - Implemented HEAD-only uncommit flow with confirmation modal and soft reset, and integrated pull_with_stash with AskPassDelegate-backed repo.pull execution
[2026-02-15 05:56] MODIFY: crates/git_graph/Cargo.toml - Added askpass workspace dependency required by Git Graph pull integration
[2026-02-15 05:59] MODIFY: crates/project/src/git_store.rs - Reintroduced delete_remote_branch API with AskPassDelegate support for local repositories
[2026-02-15 05:59] MODIFY: crates/git_graph/src/git_graph.rs - Completed remote-aware branch deletion flow (local vs remote handling), added branch normalization/parsing helpers, and integrated delete_remote_branch calls
[2026-02-15 06:00] MODIFY: crates/git_graph/src/git_graph.rs - Refined uncommit execution path signature to avoid unused window argument and keep code warning-clean
[2026-02-15 05:18] MODIFY: crates/settings_content/src/agent.rs - Added new_chat_replaces_current setting to agent settings schema
[2026-02-15 05:18] MODIFY: crates/agent_settings/src/agent_settings.rs - Wired new_chat_replaces_current into runtime AgentSettings parsing
[2026-02-15 05:18] MODIFY: assets/settings/default.json - Added default agent.new_chat_replaces_current setting documentation and default value
[2026-02-15 05:18] MODIFY: crates/agent_ui/src/agent_panel.rs - Added settings-driven new-chat behavior to open chats in tabs instead of replacing panel view
[2026-02-15 05:18] MODIFY: crates/agent_ui/src/agent_ui.rs - Updated AgentSettings test fixture with new_chat_replaces_current field
[2026-02-15 05:18] MODIFY: crates/agent/src/tool_permissions.rs - Updated AgentSettings test fixture with new_chat_replaces_current field
[2026-02-15 06:04] MODIFY: crates/settings_content/src/editor.rs - Added gutter.run_npm_scripts_directly setting schema for direct npm runnable execution
[2026-02-15 06:04] MODIFY: crates/editor/src/editor_settings.rs - Added runtime mapping for editor.gutter.run_npm_scripts_directly
[2026-02-15 06:04] MODIFY: assets/settings/default.json - Added default false value for gutter.run_npm_scripts_directly
[2026-02-15 06:04] MODIFY: crates/settings/src/vscode_import.rs - Initialized run_npm_scripts_directly in VS Code settings import gutter mapping
[2026-02-15 06:04] MODIFY: crates/editor/src/editor.rs - Added npm runnable popover bypass logic controlled by gutter.run_npm_scripts_directly
[2026-02-15 06:10] MODIFY: crates/git_ui/src/project_diff.rs - Added GitDiff actions/toggles for sync scroll and word diff highlights; added toolbar controls; and optimized refresh behavior to avoid duplicate sidebar refresh work and diff-changed panel refresh churn
[2026-02-15 06:10] MODIFY: crates/editor/src/editor.rs - Added word-diff highlight visibility state with setter/getter and snapshot propagation for diff rendering
[2026-02-15 06:10] MODIFY: crates/editor/src/element.rs - Gated word-diff highlight layout on snapshot visibility toggle
[2026-02-15 06:10] MODIFY: crates/editor/src/split.rs - Added synchronized-scroll enable/disable state and gating for pending and incoming split scroll sync events
[2026-02-15 06:10] MODIFY: assets/keymaps/default-macos.json - Added GitDiff keybindings for sync-scroll and word-diff toggles (Alt-S, Alt-D)
[2026-02-15 06:10] MODIFY: assets/keymaps/default-linux.json - Added GitDiff keybindings for sync-scroll and word-diff toggles (Alt-S, Alt-D)
[2026-02-15 06:10] MODIFY: assets/keymaps/default-windows.json - Added GitDiff keybindings for sync-scroll and word-diff toggles (Alt-S, Alt-D)
[2026-02-15 06:19] MODIFY: crates/git_graph/src/git_graph.rs - Completed Git Graph parity pass: fixed stale graph_data references, stabilized render loading logic, added commit search/filter toolbar with navigation, remapped filtered table row selection/navigation to commit indices, and integrated DiffViewer-backed per-file commit diff preview
[2026-02-15 06:19] MODIFY: crates/git_graph/Cargo.toml - Added diff_viewer workspace dependency for Git Graph commit diff rendering
[2026-02-15 06:19] MODIFY: assets/keymaps/default-macos.json - Added GitGraph search-navigation bindings (cmd-g, cmd-shift-g, escape)
[2026-02-15 06:19] MODIFY: assets/keymaps/default-linux.json - Added GitGraph search-navigation bindings (F3, Shift-F3, escape)
[2026-02-15 06:19] MODIFY: assets/keymaps/default-windows.json - Added GitGraph search-navigation bindings (F3, Shift-F3, escape)
[2026-02-15 08:08] MODIFY: crates/editor/src/split.rs - Removed malformed trailing duplicate Render impl/test block causing unclosed delimiter; restored file parseability and standard rustfmt compatibility
[2026-02-15 08:08] MODIFY: crates/git_graph/src/git_graph.rs - Fixed branch action production issues: corrected remote/local branch detection for slash-named local branches, made right-click branch selection explicit, prevented multi-delete from inheriting clicked-branch remote mode, wired checkout modal confirmation to perform checkout with stash option, wired merge/rebase modal confirmations to execute backend operations, and added checkout operation error surfacing
[2026-02-15 08:08] MODIFY: crates/git_graph/src/git_modals.rs - Improved delete-branch confirmation copy for multi-branch selection and disabled single-branch remote-delete checkbox path during multi-branch confirmation
[2026-02-15 12:44] MODIFY: .memory-bank/99-CHANGELOG.md - Resolved merge conflict markers by preserving both changelog branches
[2026-02-15 12:47] MODIFY: Cargo.lock - Corrected gpu-allocator v0.28.0 checksum to match crates.io index
[2026-02-15 12:48] MODIFY: Cargo.toml - Enabled naga termcolor feature to fix WriteColor trait build errors in naga 28
[2026-02-15 12:53] MODIFY: crates/proto/proto/zed.proto - Restored missing Envelope payload mappings for worktree/download/LSP/context-agent/search messages
[2026-02-15 12:53] MODIFY: crates/git/src/repository.rs - Replaced stale new_smol_command calls with new_command and fixed run_git_command executor ownership
[2026-02-15 12:53] MODIFY: crates/git/src/repository.rs - Switched remote delete command stdio wiring to util::command::Stdio after command API migration
[2026-02-15 12:55] MODIFY: crates/multi_buffer/src/multi_buffer.rs - Removed malformed duplicated hunk-expansion block and restored valid deleted-hunk transform construction/bracing
[2026-02-15 12:59] REFACTOR: crates/multi_buffer/src/multi_buffer.rs - Replaced inconsistent split-diff-clean version with origin/main baseline to restore missing core types/diff transforms and compile integrity
[2026-02-15 13:00] REFACTOR: crates/editor/src/split.rs - Synced split editor logic with origin/main to resolve cross-file API breakage and missing types/imports
[2026-02-15 13:00] REFACTOR: crates/editor/src/split_editor_view.rs - Synced split editor view implementation with origin/main for API compatibility
[2026-02-15 13:00] REFACTOR: crates/editor/src/element.rs - Synced editor element split-side API with origin/main (restored set_split_side support)
[2026-02-15 13:01] MODIFY: crates/editor/src/element.rs - Added compatibility no-op builder with_vertical_scrollbar_on_left for current Editor render callsites
[2026-02-15 13:01] MODIFY: crates/editor/src/editor.rs - Updated diff base_text access to pass App context (base_text(cx))
[2026-02-15 13:01] MODIFY: crates/editor/src/git/blame.rs - Removed stale default_remote_url repository call and used optional remote URL fallback
[2026-02-15 13:02] MODIFY: crates/editor/src/git/blame.rs - Added explicit Option<String> type for remote_url fallback to satisfy inference
[2026-02-15 14:31] REFACTOR: crates/git_ui/src/git_panel.rs - Synced with origin/main to restore API compatibility with current editor/project interfaces
[2026-02-15 14:31] REFACTOR: crates/git_ui/src/project_diff.rs - Synced with origin/main to remove split-view API drift and compile breakages
[2026-02-15 14:31] REFACTOR: crates/git_ui/src/git_ui.rs - Synced with origin/main to remove unresolved diff_viewer dependency path
[2026-02-15 14:31] REFACTOR: crates/git_ui/Cargo.toml - Synced dependency graph with origin/main
[2026-02-15 14:31] REFACTOR: crates/git_graph/ - Synced crate files with origin/main to remove broken diff_viewer integration path
[2026-02-15 14:31] MODIFY: Cargo.toml - Removed diff_viewer from workspace members/dependencies to exclude broken experimental crate from build graph
[2026-02-15 14:32] REFACTOR: crates/agent_ui/src/agent_panel.rs - Synced with origin/main to restore workspace update/item trait compatibility
[2026-02-15 14:32] MODIFY: crates/git_ui/src/git_panel.rs - Updated stash_all call to current Repository API and normalized error conversion; removed stale default_remote_url lookup
[2026-02-15 14:33] MODIFY: crates/git_graph/src/git_graph.rs - Replaced stale default_remote_url-based remote resolution with safe no-remote fallback for current Repository API
[2026-02-15 14:59] MODIFY: .memory-bank/02-CURRENT-STATE.md - Updated milestone/status after successful compile and runtime launch validation
[2026-02-15 18:25] MODIFY: assets/keymaps/default-macos.json - Removed stale GitGraph/AgentPanel NavigationMenu/GitDiff bindings referencing non-existent actions
[2026-02-15 18:25] MODIFY: assets/keymaps/default-linux.json - Removed stale GitGraph/AgentPanel NavigationMenu/GitDiff bindings referencing non-existent actions
[2026-02-15 18:25] MODIFY: assets/keymaps/default-windows.json - Removed stale GitGraph/AgentPanel NavigationMenu/GitDiff bindings referencing non-existent actions
[2026-02-15 19:43] MODIFY: crates/git_ui/src/project_diff.rs - Restored split diff toggle in Git Diff toolbar and wired it to editor::ToggleSplitDiff
[2026-02-15 19:43] MODIFY: assets/keymaps/default-macos.json - Restored GitDiff split-toggle shortcut (cmd-\) mapped to editor::ToggleSplitDiff
[2026-02-15 19:43] MODIFY: assets/keymaps/default-linux.json - Restored GitDiff split-toggle shortcut (ctrl-\) mapped to editor::ToggleSplitDiff
[2026-02-15 19:43] MODIFY: assets/keymaps/default-windows.json - Restored GitDiff split-toggle shortcut (ctrl-\) mapped to editor::ToggleSplitDiff
[2026-02-15 22:11] MODIFY: crates/git_ui/src/project_diff.rs - Forced Git Diff (DiffBase::Head) to open in split view by default and auto-split reused existing Git Diff panes on open
[2026-02-15 23:08] MODIFY: crates/editor/src/split_editor_view.rs - Restored split diff center gutter with curved connector ribbons and draggable divider handle for side-by-side Git diff visual parity
[2026-02-15 23:27] REFACTOR: crates/editor/src/split_editor_view.rs - Reverted direct split-renderer connector gutter customization to keep existing ProjectDiff renderer untouched
[2026-02-15 23:27] MODIFY: Cargo.toml - Reintroduced crates/diff_viewer as workspace member/dependency for separate legacy diff viewer path
[2026-02-15 23:27] MODIFY: crates/diff_viewer/ - Restored legacy diff_viewer implementation from history and updated for current editor/workspace APIs
[2026-02-15 23:27] MODIFY: crates/git_ui/Cargo.toml - Added diff_viewer workspace dependency
[2026-02-15 23:27] MODIFY: crates/git_ui/src/git_panel.rs - Routed Git panel open-diff action to launch separate DiffViewer item with committed-vs-working file content
[2026-02-15 23:27] MODIFY: crates/git_ui/src/project_diff.rs - Removed forced split/toggle toolbar changes from current ProjectDiff path
[2026-02-15 23:28] MODIFY: .memory-bank/02-CURRENT-STATE.md - Updated status to reflect separate legacy diff_viewer reintegration and Git panel routing
[2026-02-15 23:37] MODIFY: crates/diff_viewer/src/viewer.rs - Restored legacy diff-viewer UX controls in separate crate path: disabled wrapping on both panes, made right pane editable with live diff refresh, reintroduced revert block buttons, and restored collapse-unchanged blocks/toggle wiring
[2026-02-15 23:57] MODIFY: crates/git_ui/src/project_diff.rs - Fixed deploy_side_by_side to use resolve_active_repository and removed malformed duplicate braces causing compile failure
[2026-02-16 00:07] MODIFY: crates/diff_viewer/src/viewer.rs - Centered collapsed unchanged blocks and made each block clickable to expand its hidden region
[2026-02-16 00:09] MODIFY: crates/git_ui/src/git_panel.rs - Restored Git Graph header button visibility by removing action-availability gating in Git panel toolbar
[2026-02-16 00:12] MODIFY: crates/diff_viewer/src/viewer.rs - Restored proportional cross-pane scroll sync (different effective scroll speed based on each pane scrollable range)
[2026-02-16 00:18] MODIFY: crates/diff_viewer/src/viewer.rs - Persisted expanded collapsed-region state so expanded unchanged blocks stay open after click, and made collapsed-region controls full-width clickable
[2026-02-16 00:19] MODIFY: crates/diff_viewer/src/viewer.rs - Extended collapsed-region expand control hit area to full-width button for easier click targeting
[2026-02-16 00:21] MODIFY: crates/diff_viewer/src/viewer.rs - Added full-width collapsed-block expand button styling and finalized expanded-region persistence wiring
[2026-02-16 00:27] MODIFY: crates/diff_viewer/src/viewer.rs - Restored prior collapsed-region visuals (wavy strip with side plus expand button) while retaining expanded-region persistence behavior
[2026-02-16 01:16] MODIFY: crates/diff_viewer/src/viewer.rs - Restored pre-rebase DiffViewer core behavior (block-aware sync mapping, original collapse/revert interaction model), adapted to current APIs, kept right pane editable, and disabled wrapping in both panes
[2026-02-16 01:16] MODIFY: crates/diff_viewer/src/connector.rs - Restored connector block index field used by faithful revert-button targeting
[2026-02-16 01:16] MODIFY: crates/diff_viewer/src/connector_builder.rs - Restored connector block-index generation required for original revert behavior
[2026-02-16 01:16] MODIFY: crates/diff_viewer/src/constants.rs - Restored legacy collapse constants used by faithful collapsed-region rendering and behavior
[2026-02-16 01:16] MODIFY: crates/diff_viewer/src/lib.rs - Re-enabled constants module export expected by restored viewer implementation
[2026-02-16 01:16] MODIFY: crates/diff_viewer/Cargo.toml - Restored legacy diff_viewer dependency set required by pre-rebase implementation
[2026-02-16 01:16] MODIFY: Cargo.lock - Updated lockfile for restored diff_viewer dependency graph
[2026-02-16 01:16] MODIFY: .memory-bank/02-CURRENT-STATE.md - Updated milestone progress and next validation steps after restoring pre-rebase diff viewer behavior model
[2026-02-16 01:16] MODIFY: .memory-bank/99-CHANGELOG.md - Logged diff_viewer restoration and current-state update entries
[2026-02-16 02:53] MODIFY: crates/agent_ui/src/agent_panel.rs - Restored settings-driven new-chat behavior so New opens tabs when agent.new_chat_replaces_current is false, including ACP thread tab wrapper
[2026-02-16 02:53] MODIFY: .memory-bank/99-CHANGELOG.md - Logged reapplication of agent panel new-chat tab behavior
[2026-02-16 02:52] MODIFY: .memory-bank/02-CURRENT-STATE.md - Recorded continuation validation results and test blocker (`RemoteCommandOutput::default()` in fs fake git repo)
[2026-02-16 02:52] MODIFY: .memory-bank/99-CHANGELOG.md - Logged immediate-next-steps continuation audit and blocker status
[2026-02-16 03:00] MODIFY: crates/agent_ui/src/agent_panel.rs - Fixed new-chat tab creation failure by deferring workspace tab insertion to avoid re-entrant workspace.update borrow conflicts
[2026-02-16 03:00] MODIFY: .memory-bank/99-CHANGELOG.md - Logged deferred workspace insertion fix for agent new-chat tabs
[2026-02-16 09:26] MODIFY: .memory-bank/02-CURRENT-STATE.md - Added deferred note for future right-dock-local chat tabs and updated session summary timestamp
[2026-02-16 09:26] MODIFY: .memory-bank/99-CHANGELOG.md - Logged deferred right-dock tabbing follow-up note
[2026-02-16 09:32] MODIFY: crates/git_ui/src/git_panel.rs - Moved Git Graph toolbar button into Git panel header for consistent visibility and removed duplicate from previous-commit row
[2026-02-16 09:32] MODIFY: .memory-bank/99-CHANGELOG.md - Logged Git panel Git Graph button placement update and validation

[2026-02-16 10:00] MODIFY: crates/project/src/local_history.rs - Added entries_for_prefix iterator to retrieve local-history snapshots for a selected directory path prefix
[2026-02-16 10:01] MODIFY: crates/project/src/project.rs - Added local_history_for_prefix API returning cloned snapshots for directory-scope local-history access
[2026-02-16 09:52] MODIFY: crates/git_graph/src/git_graph.rs - Restored full Git Graph interaction wiring from last complete state (right-click context menus, branch multi-select, branch/commit operations, and per-file diff viewer sync)
[2026-02-16 09:52] MODIFY: .memory-bank/99-CHANGELOG.md - Logged Git Graph interaction restoration
[2026-02-16 10:03] MODIFY: crates/project_panel/src/project_panel.rs - Added OpenLocalHistory action, project-panel context menu entry, directory/file snapshot aggregation, no-history toast, and local-history picker modal that opens read-only snapshot buffers
[2026-02-16 10:03] MODIFY: crates/project_panel/Cargo.toml - Added picker workspace dependency for local-history modal picker UI
[2026-02-16 10:03] MODIFY: assets/keymaps/default-macos.json - Bound cmd-l in ProjectPanel context to project_panel::OpenLocalHistory
[2026-02-16 10:03] MODIFY: assets/keymaps/default-linux.json - Bound ctrl-l in ProjectPanel context to project_panel::OpenLocalHistory
[2026-02-16 10:03] MODIFY: assets/keymaps/default-windows.json - Bound ctrl-l in ProjectPanel context to project_panel::OpenLocalHistory
[2026-02-16 09:55] MODIFY: crates/git_graph/src/git_graph.rs - Removed stale graph module declarations/imports after restoration so GitGraph remains self-contained in current crate layout
[2026-02-16 09:55] MODIFY: .memory-bank/99-CHANGELOG.md - Logged follow-up cleanup for restored GitGraph module wiring
[2026-02-16 10:07] MODIFY: Cargo.lock - Added picker dependency in project_panel lock entry to match new local-history picker dependency wiring
[2026-02-16 10:07] MODIFY: .memory-bank/99-CHANGELOG.md - Logged local-history implementation changes, keybindings, and dependency updates
[2026-02-16 09:58] MODIFY: crates/git_ui/src/project_diff.rs - Reintroduced legacy GitDiff workflow features (next/previous file, revert current file, refresh action, whitespace/empty-line filters, sync-scroll toggle, word-diff toggle), restored toolbar controls, and updated branch-diff base selection to prefer tracked upstream branch over default branch fallback
[2026-02-16 09:58] MODIFY: crates/editor/src/split.rs - Added sync-scroll enable/disable API for SplittableEditor and conditional shared-scroll-anchor wiring to support GitDiff sync-scroll toggle
[2026-02-16 09:58] MODIFY: assets/keymaps/default-macos.json - Restored GitDiff keybindings for file navigation, refresh, whitespace/empty-line filters, sync-scroll toggle, and word-diff toggle
[2026-02-16 09:58] MODIFY: assets/keymaps/default-linux.json - Restored GitDiff keybindings for file navigation, refresh, whitespace/empty-line filters, sync-scroll toggle, and word-diff toggle
[2026-02-16 09:58] MODIFY: assets/keymaps/default-windows.json - Restored GitDiff keybindings for file navigation, refresh, whitespace/empty-line filters, sync-scroll toggle, and word-diff toggle
[2026-02-16 09:58] MODIFY: .memory-bank/02-CURRENT-STATE.md - Updated milestone status, outcomes, and next steps after reintroducing missing GitDiff workflow controls
[2026-02-16 09:58] MODIFY: .memory-bank/99-CHANGELOG.md - Logged GitDiff feature reimplementation and current-state update entries
[2026-02-16 10:31] REFACTOR: crates/git_graph/src/git_graph.rs - Synced GitGraph implementation to origin/main baseline and removed unsupported default_remote_url dependency usage for current git_store API compatibility
[2026-02-16 10:31] MODIFY: crates/project_panel/src/project_panel.rs - Fixed local-history modal wiring and picker rendering/build issues (workspace weak handle capture, RelPath display conversion, create_buffer task/spawn signature)
[2026-02-16 10:31] MODIFY: .memory-bank/99-CHANGELOG.md - Logged build-fix changes for git_graph and project_panel
