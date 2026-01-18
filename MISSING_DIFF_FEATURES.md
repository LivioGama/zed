# Missing Features: Zed vs JetBrains "Keep Differing Side by Side"

This document compares the current implementation in the `feat/collapsible-unchanged-lines` branch against JetBrains IntelliJ IDEA's side-by-side diff viewer features.

## Current Implementation Status

### ✅ Already Implemented

1. **Side-by-Side View**
   - Two synchronized editors (left: committed, right: working)
   - Visual connector curves between changes
   - Proper syntax highlighting

2. **Collapse Unchanged Fragments**
   - Automatic collapsing of unchanged lines (10+ line threshold)
   - Brain-like visual indicator for collapsed regions
   - Click-to-expand functionality with (+) button
   - Preserves 3 lines of context above/below changes
   - Global toggle button for enable/disable

3. **Scroll Synchronization**
   - Semantic scroll sync between left and right editors
   - Maintains alignment between corresponding lines

4. **Visual Diff Representation**
   - Color-coded changes (green: insertions, red: deletions, yellow: modifications)
   - Bezier curve connectors between diff regions
   - Crushed block indicators for pure insertions/deletions

5. **Basic Navigation**
   - Manual scrolling through changes
   - Visual distinction between different types of changes

---

## ❌ Missing Features (Compared to JetBrains)

### 1. **Navigation Between Changes** (HIGH PRIORITY)

**What JetBrains Has:**
- `F7` / `Shift+F7` to jump forward/backward through differences
- Visual indication when reaching first/last change
- `Alt+Right` / `Alt+Left` to navigate between multiple modified files
- Dedicated arrow buttons in toolbar

**Current Implementation:**
- No keyboard shortcuts for change navigation
- No "go to next/previous change" functionality
- User must manually scroll to find changes

**Implementation Needed:**
```rust
// In viewer.rs
- Add `current_change_index: usize` field
- Implement `goto_next_change()` method
- Implement `goto_previous_change()` method
- Register actions: `GoToNextChange`, `GoToPreviousChange`
- Add keyboard shortcuts (F7, Shift+F7 or similar)
- Add toolbar buttons with arrow icons
- Scroll to and highlight the target change
```

**Estimated Complexity:** Medium

---

### 2. **Applying Changes / Accepting Modifications** (HIGH PRIORITY)

**What JetBrains Has:**
- Chevron buttons (<<, >>) next to each change to apply/revert
- Click to replace content
- Ctrl+Click to append instead of replace
- Ability to selectively accept changes from either side

**Current Implementation:**
- NO change application functionality
- Editors are read-only
- No way to accept/reject individual changes
- No buttons to move changes between panes

**Implementation Needed:**
```rust
// In viewer.rs
- Make right editor editable (remove read-only for working copy)
- Add chevron buttons next to each connector curve
- Implement `apply_change(block_index, direction)` method
- Implement `append_change(block_index, direction)` method
- Add undo/redo support for applied changes
- Update diff analysis after changes applied
- Add visual feedback for modified regions
- Implement save functionality
```

**Estimated Complexity:** High (requires careful handling of text mutations and diff recalculation)

---

### 3. **Configurable Context Lines** (MEDIUM PRIORITY)

**What JetBrains Has:**
- User-configurable number of non-collapsible lines in Tools | Diff & Merge settings
- Allows customization of context size

**Current Implementation:**
- Hardcoded `CONTEXT_LINES = 3`
- Hardcoded `MINIMUM_COLLAPSE_THRESHOLD = 4`
- No UI to change these values

**Implementation Needed:**
```rust
// In split_diff_settings.rs
- Add `context_lines: usize` field (currently exists but unused)
- Add `minimum_collapse_threshold: usize` field
- Create settings UI panel
- Pass settings to DiffViewer
- Use settings values instead of constants
```

**Estimated Complexity:** Low

---

### 4. **Highlighting Granularity Options** (MEDIUM PRIORITY)

**What JetBrains Has:**
- 5 highlighting modes:
  1. By word
  2. By line
  3. Highlight split changes
  4. By character
  5. No highlighting
- Configurable via dropdown/menu

**Current Implementation:**
- Only line-based highlighting
- Fixed highlighting strategy
- No user control over granularity

**Implementation Needed:**
```rust
// In viewer.rs
- Add `HighlightMode` enum (Word, Line, SplitChanges, Character, None)
- Implement intra-line diff algorithm (character/word level)
- Add UI dropdown to select mode
- Implement different highlight rendering strategies
- Store preference in settings
```

**Estimated Complexity:** Medium-High (requires implementing multiple diff algorithms)

---

### 5. **Whitespace Handling Options** (MEDIUM PRIORITY)

**What JetBrains Has:**
- 4 whitespace modes:
  1. Do not ignore (default)
  2. Trim whitespaces at line boundaries
  3. Ignore all whitespaces
  4. Ignore whitespaces and empty lines
- Toolbar dropdown to switch modes

**Current Implementation:**
- Uses default `imara-diff` behavior (respects all whitespace)
- No options to ignore whitespace
- No toolbar controls

**Implementation Needed:**
```rust
// In split_diff_settings.rs
- Add `WhitespaceMode` enum
- Update `ignore_whitespace: bool` to use enum instead
- Modify diff computation to preprocess text based on mode
- Add toolbar dropdown
- Recompute diff when mode changes
```

**Estimated Complexity:** Low-Medium

---

### 6. **Swap Sides Functionality** (LOW PRIORITY)

**What JetBrains Has:**
- Button to swap left and right panes
- Useful for comparing in reverse direction

**Current Implementation:**
- Fixed left=committed, right=working
- No swap functionality

**Implementation Needed:**
```rust
// In viewer.rs
- Add `sides_swapped: bool` field
- Implement `swap_sides()` method
- Swap editor references and content
- Update connector curve directions
- Add toolbar button
```

**Estimated Complexity:** Low

---

### 7. **Three-Way Comparison** (LOW PRIORITY)

**What JetBrains Has:**
- Right-click to "Switch to Three-Side Viewer"
- Shows base, left, and right versions
- Useful for merge conflicts

**Current Implementation:**
- Only two-way comparison
- No merge conflict support

**Implementation Needed:**
```rust
// Major refactor required
- Add third editor pane
- Implement three-way diff algorithm
- Update connector curve logic for 3 panes
- Add UI to switch between 2-way and 3-way
```

**Estimated Complexity:** Very High (significant architectural change)

---

### 8. **Git Blame Integration** (LOW PRIORITY)

**What JetBrains Has:**
- Context menu access to Git Blame annotations
- Shows commit info for each line

**Current Implementation:**
- No Git Blame integration
- No commit metadata display

**Implementation Needed:**
```rust
// Integration with git crate
- Add Git Blame data loading
- Display annotations in gutter or tooltip
- Add context menu option
```

**Estimated Complexity:** Medium

---

### 9. **Advanced Editor Features in Diff View** (LOW PRIORITY)

**What JetBrains Has:**
- Code completion in diff viewer
- Live templates
- Full IDE editing capabilities

**Current Implementation:**
- Read-only editors (no editing)
- Basic syntax highlighting only
- No code intelligence

**Implementation Needed:**
```rust
// Requires making editors fully editable
- Enable LSP features in diff editors
- Add code completion support
- Add refactoring capabilities
```

**Estimated Complexity:** High (depends on Zed's LSP architecture)

---

### 10. **All-in-One Diff Viewer** (LOW PRIORITY)

**What JetBrains Has:**
- IntelliJ IDEA 2023.3+ shows all modified files in single scrollable frame
- No need to switch between files
- Continuous review of entire changeset

**Current Implementation:**
- One file at a time
- Must close and open different files

**Implementation Needed:**
```rust
// Major architectural change
- Create multi-file diff viewer
- Vertical stacking of file diffs
- File headers with expand/collapse
- Efficient rendering of multiple diffs
```

**Estimated Complexity:** Very High

---

### 11. **Clipboard Comparison** (LOW PRIORITY)

**What JetBrains Has:**
- "Compare with Clipboard" functionality
- Open blank diff viewer to paste arbitrary text

**Current Implementation:**
- Only compares git versions and working files
- No arbitrary text comparison

**Implementation Needed:**
```rust
// In project_diff.rs
- Add "Compare with Clipboard" action
- Load clipboard content as one side
- Allow pasting text into blank diff
```

**Estimated Complexity:** Low

---

### 12. **Better Collapsed Region Customization** (LOW PRIORITY)

**What JetBrains Has:**
- Configurable number of non-collapsible lines
- Per-region controls (not just global toggle)

**Current Implementation:**
- Only global toggle for all regions
- Hardcoded 3-line context
- No per-region expand/collapse control

**Implementation Needed:**
```rust
// In viewer.rs
- Add individual expand buttons per region (already exists)
- Add "expand all" / "collapse all" buttons
- Add hover tooltips showing line count
- Allow double-click to expand/collapse
```

**Estimated Complexity:** Low

---

## Priority Summary

### **Critical Path (Must-Have)**
1. ✅ Side-by-Side View
2. ✅ Collapse Unchanged Fragments
3. ❌ **Navigation Between Changes** (F7/Shift+F7)
4. ❌ **Applying Changes** (Accept/Reject modifications)

### **High Value (Should-Have)**
5. ❌ **Highlighting Granularity** (Word/Char level)
6. ❌ **Whitespace Handling Options**
7. ❌ **Configurable Context Lines**

### **Nice-to-Have**
8. ❌ Swap Sides
9. ❌ Clipboard Comparison
10. ❌ Better Collapsed Region Controls

### **Future Enhancements**
11. ❌ Three-Way Comparison
12. ❌ Git Blame Integration
13. ❌ All-in-One Diff Viewer
14. ❌ Full Editor Features in Diff

---

## Implementation Roadmap

### Phase 1: Core Missing Features (1-2 weeks)
- [ ] Navigation between changes (F7/Shift+F7)
- [ ] Applying changes (chevron buttons)
- [ ] Configurable context lines (settings UI)

### Phase 2: Enhanced Diff Experience (1 week)
- [ ] Whitespace handling options
- [ ] Highlighting granularity (word/character level)
- [ ] Swap sides functionality

### Phase 3: Advanced Features (2-3 weeks)
- [ ] Clipboard comparison
- [ ] Git Blame integration
- [ ] Better collapsed region controls
- [ ] Three-way comparison (if needed)

### Phase 4: Polish (1 week)
- [ ] All-in-One diff viewer (multi-file)
- [ ] Full editor features in diff view
- [ ] Performance optimizations
- [ ] Documentation

---

## Technical Debt & Architecture Improvements

### Current Issues
1. **Read-only limitation:** Makes "apply changes" feature difficult
2. **Hardcoded constants:** Should be in settings
3. **No undo/redo:** Needed for change application
4. **Single-file only:** Limits workflow efficiency

### Recommended Refactors
1. Make right editor conditionally editable
2. Add proper settings management
3. Implement undo/redo stack for diff operations
4. Create multi-file diff viewer architecture

---

## Conclusion

The current implementation has **excellent foundations**:
- ✅ Beautiful side-by-side view
- ✅ Smart collapsing of unchanged fragments
- ✅ Smooth scroll synchronization
- ✅ Professional visual design

**Critical missing features** for feature parity with JetBrains:
1. **Navigation between changes** (keyboard shortcuts)
2. **Applying changes** (accept/reject individual modifications)
3. **Highlighting options** (word/character level diffs)
4. **Whitespace handling** (ignore options)

Once these 4 features are implemented, Zed's diff viewer will match or exceed JetBrains' "Keep Differing Side by Side" functionality.

---

## Sources

- [Diff Viewer for files | IntelliJ IDEA Documentation](https://www.jetbrains.com/help/idea/differences-viewer.html)
- [Compare files, folders, and text sources | IntelliJ IDEA](https://www.jetbrains.com/help/idea/comparing-files-and-folders.html)
- [IntelliJ IDEA 2023.3 EAP 2: All-in-One Diff Viewer](https://blog.jetbrains.com/idea/2023/09/intellij-idea-2023-3-eap-2/)
- [Beyond Comparison: Compare Anything in IntelliJ IDEA](https://blog.jetbrains.com/idea/2022/06/compare-anything-in-intellij-idea/)
