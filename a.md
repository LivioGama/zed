Git Diff Viewer - Collapse Unchanged Code Feature

## 1) Feature Summary

Implement a collapsible unchanged code feature for the existing git diff viewer that automatically collapses contiguous blocks of unchanged lines, leaving only changed regions and configurable context lines visible. This feature must include a toolbar toggle button to enable/disable the collapsing behavior, with the feature enabled by default. The visual design and interaction patterns must match JetBrains IntelliJ IDEA's diff viewer implementation, providing a familiar and professional code review experience.

---

## 2) Research-Based Context

**Comparable Products & Patterns Identified:**
- **JetBrains IntelliJ IDEA**: Industry-leading implementation with "Collapse Unchanged Fragments" toolbar button, configurable context lines (via slider in settings), and collapsible regions showing ellipsis indicators that expand on click.
- **Kaleidoscope 6**: Uses clickable ellipsis buttons inside collapsed areas that change color on hover to indicate interactivity, supports global toggle and per-region expansion.
- **React Diff Viewer**: Displays "Expand {number} of lines" messages in collapsed regions, uses render props for customization.
- **DiffCheck.io**: Automatically collapses large unchanged sections with "Expand lines..." clickable text.
- **GitLab**: Automatically collapses diffs beyond size thresholds (10% of max patch size, 100 files, 5000 lines, or 500KB) but keeps them expandable.

**Key UX Standards Users Will Expect:**
- Toolbar button with clear icon (collapse/fold symbol) to toggle the feature globally
- Collapsed regions display as visual separators with line count information
- Click-to-expand interaction on collapsed regions (no keyboard required for basic use)
- Configurable "context lines" - unchanged lines surrounding each change that remain visible
- State persistence - collapsed/expanded state should survive navigation within the same session
- Instant visual feedback on toggle - no loading states for collapse/expand operations

**Familiar vs. Innovative:**
- **Familiar**: Ellipsis/fold indicators, toolbar toggle, context line preservation, click-to-expand - these are established patterns users recognize from JetBrains IDEs, VS Code (requested feature), and professional diff tools.
- **Innovative**: The implementation can improve upon competitors by ensuring the collapse state feels lightweight and responsive, with smooth transitions and clear visual hierarchy.

---

## 3) Users & Scope

**Target Users:**
- Software developers reviewing git changes before committing
- Code reviewers examining pull requests or diffs between branches
- Developers comparing file versions to understand what changed

**Primary Use Cases:**
1. Reviewing large files where most content is unchanged (e.g., adding a single method to a 500-line class)
2. Quickly scanning a diff to identify all changed regions without scrolling through irrelevant code
3. Focusing on specific changes while occasionally expanding context to understand surrounding code
4. Toggling between "see everything" and "see only changes" modes during review

**Explicit Non-Goals (What Must NOT Be Built):**
- Do NOT implement smart/semantic collapsing (e.g., collapse by function/class)
- Do NOT implement per-file or per-hunk collapse controls (only global toggle)
- Do NOT implement persistent settings storage (collapse state is session-only)
- Do NOT implement undo/redo for expand/collapse actions
- Do NOT implement keyboard shortcuts for expanding individual regions (global toggle only)
- Do NOT implement configuration UI for context lines in this iteration (hardcode a default value)
- Do NOT implement collapse of changed lines or partially changed blocks
- Do NOT add animations longer than 150ms for collapse/expand
- Do NOT implement this for the unified diff mode (only side-by-side split view)

---

## 4) User Experience & Interaction Specification

### End-to-End User Flows

**Flow 1: First-time user sees a diff with collapse enabled (default)**
1. User opens a file diff in the split diff viewer
2. System immediately collapses all unchanged line blocks longer than (context lines * 2 + minimum collapse threshold)
3. Collapsed regions appear as horizontal separators with expansion affordances
4. User sees only: changed lines + N context lines above/below each change + collapsed region indicators
5. User can immediately understand "this is what changed" without scrolling through unchanged code

**Flow 2: User expands a specific collapsed region**
1. User hovers over a collapsed region separator
2. Separator changes appearance (highlight or color shift) to indicate it's interactive
3. User clicks anywhere on the collapsed region separator
4. Collapsed region smoothly expands (duration: 100-150ms) to reveal hidden unchanged lines
5. Expanded region remains visible until user toggles collapse button again or clicks a collapse affordance (if provided)
6. Other collapsed regions remain collapsed

**Flow 3: User toggles collapse feature off**
1. User clicks the "Collapse Unchanged Fragments" toolbar button (currently showing "enabled" state)
2. Button immediately changes to "disabled" visual state
3. All collapsed regions expand simultaneously (smooth animation, 100-150ms)
4. User now sees the full diff with all unchanged lines visible
5. No collapsed region indicators remain visible

**Flow 4: User toggles collapse feature back on**
1. User clicks the "Collapse Unchanged Fragments" toolbar button (currently showing "disabled" state)
2. Button immediately changes to "enabled" visual state
3. System re-analyzes the diff and collapses eligible unchanged blocks
4. Collapsed regions appear with smooth animation (100-150ms)
5. Previously manually-expanded regions are collapsed again (no memory of per-region state)

**Flow 5: User scrolls through a diff with many changes**
1. User scrolls vertically through the diff
2. Collapsed regions act as visual anchors between changed blocks
3. User can click any collapsed region to expand context without losing scroll position
4. Scroll position remains stable during expand/collapse (no jumping)

### UI Layout & Component Responsibilities

**Toolbar Button (Collapse Toggle)**
- **Location**: Positioned in the diff viewer toolbar, adjacent to other view controls (e.g., split/unified mode toggle)
- **Appearance (Enabled State)**: Icon showing collapse/fold symbol (e.g., two parallel horizontal lines with arrows pointing inward, or a minus-in-box icon), with visual indication that feature is active (highlighted background or border)
- **Appearance (Disabled State)**: Same icon but with muted/inactive styling (gray or low-opacity)
- **Label**: Text label "Collapse Unchanged Fragments" appears on hover (tooltip)
- **Behavior**: Single click toggles between enabled and disabled states

**Collapsed Region Indicator**
- **Location**: Spans full width of both left and right editor panes in split view (acts as a separator between visible code blocks)
- **Visual Structure**:
  - Horizontal separator line (styled similarly to JetBrains IDEs - subtle, not jarring)
  - Center element: Icon (ellipsis "..." or fold icon) + text showing line count (e.g., "24 unchanged lines")
  - Optional: Small expand icon/chevron to reinforce clickability
- **Height**: Compact (approximately 1.5-2x the line height of code) to minimize wasted space
- **Background**: Slightly different from code background to create visual separation
- **Text**: Single line, centered, using a slightly smaller or muted font compared to code

**Expanded Code Region**
- When a collapsed region is expanded, the hidden lines appear exactly as they would in the normal diff view (no special styling)
- Expanded lines maintain proper syntax highlighting and line numbers
- No visual marker distinguishes "manually expanded" lines from "always visible" lines

### Interaction Details

**Scroll Behavior:**
- Collapsing/expanding regions does NOT reset scroll position
- Expanding a region above the current viewport should shift content down smoothly (maintain relative scroll position)
- Expanding a region below the current viewport has no scroll side effects

**Drag Physics:**
- Not applicable (no drag interactions for this feature)

**Snapping:**
- Not applicable (no snapping behavior)

**Focus:**
- Clicking the toolbar toggle button does not move keyboard focus away from the editor
- Clicking a collapsed region indicator does not move keyboard focus
- Keyboard navigation (arrow keys) within the editor treats collapsed regions as single "block" elements (pressing down-arrow at the last visible line before a collapsed region jumps to the first visible line after it)

### Micro-Interactions

**Hover States:**
- **Toolbar Button Hover**: Subtle background color change + cursor changes to pointer + tooltip appears after 500ms
- **Collapsed Region Hover**: Background color lightens or border appears + cursor changes to pointer + text/icon may brighten

**Active States:**
- **Toolbar Button Active**: Pressed appearance (slightly darkened) during click
- **Collapsed Region Active**: Slight scale or color change during click

**Disabled States:**
- **Toolbar Button Disabled**: Not applicable (button is always enabled)
- **Collapsed Region Disabled**: Not applicable (collapsed regions are only visible when feature is enabled)

**Transitions:**
- **Collapse Animation**: Height collapses from full height to collapsed indicator height over 100-150ms with ease-out easing
- **Expand Animation**: Height expands from collapsed indicator height to full height over 100-150ms with ease-out easing
- **Button State Change**: Instant (no animation on toggle button state change itself, only on the resulting collapse/expand of regions)

### Keyboard Shortcuts & Accessibility

**Keyboard Shortcuts:**
- Global toggle: `Ctrl+Shift+.` (Windows/Linux) or `Cmd+Shift+.` (Mac) - toggles collapse feature on/off
- No shortcuts for expanding individual regions (must use mouse for now)

**Accessibility:**
- Toolbar button must have proper ARIA label: "Collapse Unchanged Fragments" + current state ("enabled" or "disabled")
- Collapsed region indicators must have ARIA role="button" and aria-label="Expand {N} unchanged lines"
- Collapsed regions must be keyboard-focusable (Tab key) and activatable with Enter/Space
- Screen readers should announce when collapse mode is toggled: "Unchanged lines collapsed" or "All lines visible"
- Ensure sufficient color contrast for collapsed region text and icons (WCAG AA minimum)

### Empty, Loading, Partial, and Error States

**Empty State (No Changes):**
- If a diff has zero changes (files are identical), collapse feature has no effect
- Toolbar button remains visible and functional but clicking it does nothing (no collapsed regions exist to collapse)
- User sees "No changes" message in diff viewer (existing behavior, unchanged)

**Loading State:**
- While diff is loading, collapse feature is disabled (toolbar button disabled/grayed)
- Once diff loads, if collapse is enabled by default, collapsing happens immediately (no separate loading step)

**Partial State (Collapse in Progress):**
- Not applicable - collapsing is instant/synchronous or fast enough (<50ms) to feel instant

**Error State (Diff Load Failure):**
- If diff fails to load, collapse feature is disabled (toolbar button disabled/grayed)
- Error message displays in diff viewer (existing behavior, unchanged)

**Edge Case: All Lines Are Changed:**
- If every line in the diff is changed or within context distance of a change, no collapsed regions appear
- Toolbar button remains enabled, but has no visual effect
- User sees message (subtle, non-modal): "No unchanged regions to collapse" (optional - could simply show no collapsed regions)

**Edge Case: Very Short File:**
- If file is shorter than (context lines * 2 + minimum threshold), no collapsed regions appear
- Behavior is same as "all lines are changed" case

---

## 5) Visual, Motion & Spatial Design Specification

### Color Rules

**Semantic Roles:**
- **Collapsed Region Background**: Use a neutral background color slightly lighter (light theme) or darker (dark theme) than the editor background - suggests "this area is distinct but not alarming"
- **Collapsed Region Text/Icon**: Use muted text color (60-70% opacity of normal text) - indicates "secondary information"
- **Collapsed Region Hover**: Increase background lightness/darkness by 10-15% - indicates "interactive"
- **Toolbar Button Enabled**: Use accent color (same as other enabled toolbar buttons) with 100% opacity
- **Toolbar Button Disabled**: Use gray or 40% opacity of enabled color
- **Ellipsis Icon**: Use same color as text or slightly muted

**Contrast Constraints:**
- Collapsed region text must meet WCAG AA contrast ratio (4.5:1 for normal text, 3:1 for large text)
- Hover states must be distinguishable from non-hover by at least 15% lightness difference
- Toolbar button enabled/disabled states must be clearly distinguishable (50%+ difference in saturation or lightness)

### Typography Rules

**Hierarchy:**
1. **Primary**: Code text (existing font family, size, weight)
2. **Secondary**: Collapsed region text (same font family, 90-95% of code font size, normal weight)
3. **Tertiary**: Tooltip text (existing UI font, existing tooltip size)

**Truncation:**
- Collapsed region text does NOT truncate - it's short by design (e.g., "24 unchanged lines")
- If line count exceeds 9999, display as "9999+ unchanged lines"

**Wrapping:**
- Collapsed region text does NOT wrap - single line only
- If text is too long (should never happen with current design), truncate with ellipsis

### Spacing & Layout Logic

**Grid:**
- Collapsed region indicator aligns to the left edge of the left editor pane and right edge of the right editor pane
- Center content (icon + text) is horizontally centered within the collapsed region
- Vertical spacing: 4-6px padding above and below the text/icon within the collapsed region

**Paddings:**
- Collapsed region: 8-12px horizontal padding (left/right), 4-6px vertical padding (top/bottom)
- Toolbar button: Match existing toolbar button padding (maintain consistency)

**Alignment Rules:**
- Collapsed region text and icon: Horizontally centered
- Collapsed region indicator: Vertically centered between the visible code blocks above and below
- Toolbar button icon: Centered within button bounds

### Animation Rules

**When Animations Occur:**
- When collapse feature is toggled ON: Animate collapse of eligible regions
- When collapse feature is toggled OFF: Animate expansion of all collapsed regions
- When individual collapsed region is clicked: Animate expansion of that single region
- No animation when scrolling or navigating between files

**Duration Ranges:**
- Collapse animation: 100-150ms (leaning toward 120ms)
- Expand animation: 100-150ms (leaning toward 120ms)
- Hover transitions: 80-100ms (for background color changes)

**Easing Intent:**
- Collapse: Use ease-out (starts fast, ends slow) - feels responsive and controlled
- Expand: Use ease-out (starts fast, ends slow) - feels fluid and intentional
- Hover: Use ease-in-out (smooth acceleration/deceleration) - feels polished

**Geometry & Visual Intent (for collapsed regions):**
- **Shape**: Rectangular bar spanning full width of both editor panes
- **Height Transition**: During collapse, the visual goal is to smoothly shrink the vertical space occupied by unchanged lines until only the collapsed indicator remains (final height: approximately 1.5-2x line height)
- **Opacity Transition**: During collapse, unchanged lines should fade out (opacity 1 → 0) while collapsed indicator fades in (opacity 0 → 1) - creates a seamless handoff
- **Center Element Behavior**: The ellipsis icon and text appear at the midpoint of the collapsing region and remain fixed in position as the region collapses around it
- **No Horizontal Movement**: All animation is vertical (height change) - no horizontal shifts or slides
- **Visual Intent**: The animation should feel like "compressing" or "folding" the code, not "hiding" or "deleting" it - users should intuitively understand the content is still there, just minimized

---

## 6) Product Logic & Business Rules

**Rule 1: Collapse Eligibility**
- A block of unchanged lines is eligible for collapse if and only if:
  1. It contains at least (2 * context_lines + minimum_threshold) consecutive unchanged lines
  2. minimum_threshold = 4 (hardcoded)
  3. context_lines = 3 (hardcoded default)
  4. Example: With context=3 and threshold=4, a block needs ≥10 consecutive unchanged lines to collapse
- Changed lines are NEVER collapsed
- Lines within context_lines distance of any changed line are NEVER collapsed

**Rule 2: Context Lines Preservation**
- When a collapsed region is created, preserve exactly N lines above the first changed line and exactly N lines below the last changed line in each contiguous change block
- N = context_lines (default: 3)
- If two change blocks are separated by fewer than (2 * context_lines + minimum_threshold) lines, do NOT create a collapsed region between them - show all lines

**Rule 3: Default State**
- Collapse feature is ENABLED by default when opening any diff view
- This applies to: new diff views, reopened diff views, and navigation between files within the same diff session

**Rule 4: Toggle Behavior**
- Toggling collapse OFF expands all collapsed regions immediately
- Toggling collapse ON re-analyzes the current diff and collapses all eligible regions
- Toggling does NOT remember which regions were manually expanded - all eligible regions collapse again

**Rule 5: Per-Region Expansion**
- Clicking a collapsed region expands it immediately
- Expanded regions remain expanded until the global toggle is clicked ON again
- Expanding one region does NOT affect other collapsed regions

**Rule 6: Navigation & State Reset**
- When navigating to a different file in the diff viewer:
  - If collapse is globally enabled, apply collapsing to the new file automatically
  - Do NOT preserve which specific regions were manually expanded in the previous file
- When closing and reopening the diff viewer, collapse state resets to default (enabled)

**Conflict Resolution Logic:**
- Not applicable (no conflicts between user actions in this feature)

**Permissions/Roles:**
- Not applicable (all users have same access to this feature)

**User-Facing Constraints:**
- Maximum line count displayable in collapsed region indicator: 99,999 (display as "99999+" if exceeded)
- Minimum context lines: 1 (even if configurable later, never allow 0)
- Minimum threshold for collapse: 4 lines (hardcoded, but should be easy to adjust)

---

## 7) Edge Cases & Failure Modes

**Edge Case 1: File has only 1-2 lines changed**
- **Trigger**: Diff contains very few changes surrounded by large unchanged blocks
- **User Sees**: Most of the file is collapsed into 1-2 large collapsed regions
- **Recovery**: User can expand collapsed regions to see full context; toggle collapse OFF to see everything

**Edge Case 2: File has changes on nearly every line**
- **Trigger**: Diff shows extensive modifications (reformatting, refactoring, etc.)
- **User Sees**: No collapsed regions appear (every line is within context distance of a change)
- **Recovery**: Feature gracefully does nothing; user sees full diff as if collapse were disabled

**Edge Case 3: Very long file (10,000+ lines) with minimal changes**
- **Trigger**: Reviewing a large file (e.g., generated code, data file) with 1-2 small changes
- **User Sees**: Most of file is collapsed into 1-2 massive collapsed regions (e.g., "9,847 unchanged lines")
- **Recovery**: User can expand specific collapsed region or toggle collapse OFF; performance must remain acceptable (expand animation should complete in <200ms even for 10k lines)

**Edge Case 4: User clicks collapsed region multiple times rapidly**
- **Trigger**: User double-clicks or triple-clicks a collapsed region
- **User Sees**: Expansion happens once; subsequent clicks during animation are ignored
- **Recovery**: Animation completes normally; no visual glitches or multiple expansions

**Edge Case 5: User toggles collapse ON/OFF rapidly**
- **Trigger**: User clicks toolbar button multiple times in quick succession
- **User Sees**: Each click reverses the current animation (collapse → expand → collapse...)
- **Recovery**: Final state matches final button state; no stuck animations or broken states

**Edge Case 6: User expands a region, then immediately toggles collapse OFF**
- **Trigger**: User expands a collapsed region, then clicks toolbar toggle to disable collapse
- **User Sees**: Expanded region (and all others) expand/remain expanded; no visual anomaly
- **Recovery**: All collapsed regions are now gone; manual expansion is effectively "undone" by the global toggle

**Edge Case 7: User toggles collapse ON, then immediately scrolls**
- **Trigger**: User enables collapse feature (triggering collapse animations), then scrolls before animations complete
- **User Sees**: Scroll operates normally; animations continue in background without affecting scroll
- **Recovery**: Scroll position remains stable; animations complete smoothly

**Edge Case 8: Collapsed region is at the very top of the file**
- **Trigger**: File starts with unchanged lines (e.g., license header, imports)
- **User Sees**: Collapsed region appears at top of diff; no visible code above it
- **Recovery**: Expanding works normally; user sees the beginning of the file

**Edge Case 9: Collapsed region is at the very bottom of the file**
- **Trigger**: File ends with unchanged lines (e.g., trailing whitespace, closing braces)
- **User Sees**: Collapsed region appears at bottom of diff; no visible code below it
- **Recovery**: Expanding works normally; user sees the end of the file

**Edge Case 10: File has only unchanged lines (no changes)**
- **Trigger**: Comparing two identical files or a file with only whitespace changes (if ignored)
- **User Sees**: "No changes" message (existing behavior); no collapsed regions
- **Recovery**: Collapse toggle button remains enabled but has no effect

**Edge Case 11: Diff viewer is resized while collapsed regions are visible**
- **Trigger**: User resizes window or panel while viewing a collapsed diff
- **User Sees**: Collapsed regions resize to fit new width; no layout breaks
- **Recovery**: Resize is smooth; no re-collapse or re-calculation needed

**Edge Case 12: User expands a region that spans across a scrolled-out viewport**
- **Trigger**: User expands a large collapsed region (500+ lines) that extends beyond visible area
- **User Sees**: Viewport remains at current scroll position; expanded lines appear above/below
- **Recovery**: User can scroll to see newly expanded lines; scroll position is stable (no jump to top/bottom)

**Edge Case 13: Collapse feature is enabled during an ongoing diff load**
- **Trigger**: User toggles collapse ON while diff is still rendering/loading
- **User Sees**: Collapse is queued and applies once diff is fully loaded
- **Recovery**: Graceful degradation; no errors or visual glitches

---

## 8) Performance, Reliability & Safety Expectations

**Performance Targets:**
- **Collapse/Expand Animation**: Must complete within 150ms (target: 120ms) even for 10,000-line diffs
- **Toggle Button Responsiveness**: Button state change must be perceived as instant (<50ms from click to visual feedback)
- **Initial Diff Render with Collapse**: Total time from opening diff to fully rendered collapsed view must not exceed 500ms for typical files (<2000 lines)
- **Memory**: Collapsed regions should not duplicate stored line data - reference existing diff data structures
- **Scroll Performance**: Scrolling through a diff with 20+ collapsed regions must maintain 60 FPS

**Expected Behavior Under Stress:**
- **Very Large Diffs (20,000+ lines)**: Collapse calculations may take 100-200ms; provide subtle loading indicator on toolbar button if calculation exceeds 100ms
- **Rapid Toggling**: Debounce toggle clicks with 50ms delay to prevent excessive re-calculations
- **Low-End Hardware**: If animation frame rate drops below 30 FPS, reduce animation duration to 80ms or disable animation entirely (instant collapse/expand)

**Expected Behavior Under Partial Failure:**
- **Collapse Calculation Error**: If collapse logic fails (bug, unexpected diff format), degrade gracefully by disabling collapse feature and showing full diff with error notification (non-blocking)
- **Animation Failure**: If animation library/system fails, fall back to instant collapse/expand (no animation)

**Basic Security Boundaries:**
- **Auth Assumptions**: Collapse feature has no auth requirements; assumes user already has access to view diffs
- **Unsafe Inputs**: Line counts and diff data are assumed to be safe (validated by upstream diff generation); no special sanitization needed for display

**Privacy Expectations:**
- **Logging**: Do NOT log collapsed region sizes, expansion events, or user interaction patterns with this feature
- **Telemetry**: If telemetry is enabled, only log: "collapse feature toggled" (on/off state) and "collapsed region expanded" (boolean, no line numbers or counts)
- **Local Storage**: Do NOT persist collapsed state or user preferences to localStorage or any backend

---

## 9) Acceptance Criteria (Implementation-Focused)

**Must-Have (Critical):**
- [ ] Toolbar button appears in diff viewer with clear icon and tooltip
- [ ] Button toggles between enabled/disabled states on click
- [ ] Button is enabled by default when opening a diff
- [ ] Unchanged line blocks meeting collapse criteria are collapsed when feature is enabled
- [ ] Collapsed regions display with centered ellipsis icon + line count text
- [ ] Clicking a collapsed region expands it immediately
- [ ] Hovering over a collapsed region changes its appearance (indicates interactivity)
- [ ] Toggling collapse OFF expands all collapsed regions
- [ ] Toggling collapse ON re-collapses all eligible regions
- [ ] Context lines (default: 3) remain visible above/below each change
- [ ] Collapse/expand animations complete within 150ms
- [ ] No layout shifts or scroll position jumps during collapse/expand
- [ ] Keyboard shortcut (Ctrl+Shift+. or Cmd+Shift+.) toggles collapse feature
- [ ] Collapsed regions are keyboard-focusable and activatable (Enter/Space)
- [ ] ARIA labels are present on toolbar button and collapsed regions
- [ ] Feature works in side-by-side split diff view (not unified view)

**Should-Have (Important):**
- [ ] Collapsed region background color differs subtly from editor background
- [ ] Collapsed region text is muted (60-70% opacity) vs. normal text
- [ ] Hover state increases collapsed region background lightness/darkness by 10-15%
- [ ] Toolbar button has distinct enabled/disabled visual states
- [ ] Ellipsis icon and text are horizontally centered in collapsed region
- [ ] Collapsed region height is approximately 1.5-2x line height
- [ ] Expand animation uses ease-out easing
- [ ] Line count displays as "{N} unchanged lines" (e.g., "24 unchanged lines")
- [ ] Line counts >9999 display as "9999+ unchanged lines"
- [ ] Very short files (<10 lines) show no collapsed regions
- [ ] Files with no changes show no collapsed regions
- [ ] Scroll performance remains 60 FPS with 20+ collapsed regions

**Nice-to-Have (Polish):**
- [ ] Collapsed region hover shows subtle border or outline
- [ ] Toolbar button shows subtle pressed state during click
- [ ] Opacity transition during collapse (unchanged lines fade out, indicator fades in)
- [ ] Collapsed region at top/bottom of file aligns properly (no orphaned spacing)
- [ ] Error notification if collapse calculation fails (non-blocking)
- [ ] Subtle loading indicator on toolbar button if collapse calculation exceeds 100ms
- [ ] Screen reader announces "Unchanged lines collapsed" / "All lines visible" on toggle

---

## 10) Minimal Test Guidance (Intentionally Light)

**Critical Sanity Checks:**

1. **Basic Toggle Test:**
   - Open a diff with significant unchanged sections
   - Verify collapsed regions appear by default
   - Click toolbar button → verify all regions expand
   - Click toolbar button again → verify all regions re-collapse

2. **Per-Region Expansion Test:**
   - Open a collapsed diff
   - Click one collapsed region → verify only that region expands
   - Click another collapsed region → verify both are now expanded
   - Other collapsed regions remain collapsed

3. **Context Lines Test:**
   - Open a diff with a single-line change surrounded by many unchanged lines
   - Verify exactly 3 lines above and 3 lines below the change are visible
   - Verify unchanged lines beyond the 3-line context are collapsed

4. **Edge Case: All Changed Lines:**
   - Create a diff where every line is changed
   - Verify no collapsed regions appear
   - Verify toolbar button remains enabled

5. **Edge Case: Tiny File:**
   - Create a diff with <10 total lines and 1 line changed
   - Verify no collapsed regions appear (file too short to collapse anything)

6. **Keyboard Navigation Test:**
   - Open a collapsed diff
   - Tab to a collapsed region → verify it receives focus
   - Press Enter → verify region expands
   - Press Ctrl+Shift+. (or Cmd+Shift+.) → verify collapse toggles

7. **Performance Test:**
   - Open a diff with 5000+ lines and 2-3 small changes
   - Verify collapse happens within 500ms of opening diff
   - Click toolbar to expand all → verify expansion completes within 200ms
   - Scroll through diff → verify smooth 60 FPS scrolling

8. **State Persistence Test:**
   - Enable collapse, manually expand 2 regions
   - Navigate to a different file in diff viewer
   - Navigate back to original file → verify manual expansions are NOT remembered (all eligible regions are collapsed again)

**Manual Testing Checklist (Quick Validation):**
- [ ] Toolbar button appears and looks correct (icon, tooltip)
- [ ] Collapsed regions look like JetBrains IDEA (centered text, subtle separator)
- [ ] Hover states work (button and collapsed regions)
- [ ] Clicking collapsed regions expands them
- [ ] Toggle button works (on/off state changes)
- [ ] Keyboard shortcut works
- [ ] No visual glitches during animations
- [ ] Scroll position stable during expand/collapse
- [ ] Accessible via keyboard (Tab, Enter, Space)

---

## 11) Assumptions & Tunable Parameters

### Assumptions Made

1. **Split View Only**: The feature is only needed for side-by-side split diff view, not unified diff view. (If unified view support is needed later, it can be added.)

2. **Session-Only State**: Collapse state (enabled/disabled, manually expanded regions) is session-only and does not persist across app restarts or navigation to different diffs. (If persistence is needed later, it can be added with a settings system.)

3. **Hardcoded Context Lines**: The number of context lines (3) is hardcoded for this iteration. (A settings UI can be added later if users request configurability.)

4. **Single Global Toggle**: There is one global toggle for the entire diff viewer, not per-file or per-pane toggles. (Per-file collapse could be added later if needed.)

5. **No Smart Collapsing**: The feature does NOT attempt to collapse by function, class, or semantic blocks - only by contiguous unchanged lines. (Semantic collapsing could be added as an advanced feature later.)

6. **Existing Diff Data Structure**: The implementation assumes access to diff data that clearly marks lines as "changed" or "unchanged" (e.g., diff hunks with line ranges). The code should work with the existing diff data model without requiring changes to upstream diff generation.

7. **No Backend Changes**: The feature is entirely frontend/UI - no backend API changes, no database schema changes, no new data storage. (All collapse logic runs client-side.)

8. **Modern Browser/Runtime**: Assumes modern browser APIs (or Electron/GPUI equivalents) for smooth animations (requestAnimationFrame or equivalent).

9. **No Undo/Redo**: Expanding/collapsing is not part of the undo/redo history. (If users request this, it can be added later.)

10. **No Conflict with Other Features**: Assumes no conflicts with existing diff viewer features (line numbers, syntax highlighting, inline comments, etc.). Collapsed regions should integrate seamlessly with existing UI.

### Tunable Parameters (Easy to Adjust)

**Collapse Logic:**
- `CONTEXT_LINES`: Number of unchanged lines to preserve above/below each change (default: 3)
- `MINIMUM_COLLAPSE_THRESHOLD`: Minimum unchanged lines needed to create a collapsed region (default: 4)
- Formula: Collapse if unchanged block ≥ (2 * CONTEXT_LINES + MINIMUM_COLLAPSE_THRESHOLD)

**Animation Durations:**
- `COLLAPSE_DURATION_MS`: Duration of collapse animation (default: 120ms, range: 80-150ms)
- `EXPAND_DURATION_MS`: Duration of expand animation (default: 120ms, range: 80-150ms)
- `HOVER_TRANSITION_MS`: Duration of hover state transitions (default: 80ms, range: 50-100ms)

**Visual Sizing:**
- `COLLAPSED_REGION_HEIGHT_MULTIPLIER`: Height of collapsed region as multiple of line height (default: 1.5, range: 1.2-2.0)
- `COLLAPSED_REGION_PADDING_VERTICAL_PX`: Vertical padding inside collapsed region (default: 5px, range: 4-8px)
- `COLLAPSED_REGION_PADDING_HORIZONTAL_PX`: Horizontal padding inside collapsed region (default: 10px, range: 8-16px)

**Colors (Theme-Dependent):**
- `COLLAPSED_REGION_BG_LIGHTNESS_DELTA`: How much lighter/darker collapsed region is vs. editor bg (default: 5%, range: 3-10%)
- `COLLAPSED_REGION_TEXT_OPACITY`: Opacity of text in collapsed region (default: 0.65, range: 0.5-0.8)
- `COLLAPSED_REGION_HOVER_LIGHTNESS_DELTA`: Additional lightness change on hover (default: 12%, range: 8-20%)

**Typography:**
- `COLLAPSED_REGION_FONT_SIZE_MULTIPLIER`: Font size as multiple of code font size (default: 0.92, range: 0.85-1.0)
- `MAX_LINE_COUNT_DISPLAY`: Maximum line count to display before showing "9999+" (default: 9999)

**Performance:**
- `COLLAPSE_CALCULATION_TIMEOUT_MS`: Show loading indicator if collapse takes longer than this (default: 100ms)
- `TOGGLE_DEBOUNCE_MS`: Debounce rapid toggle clicks (default: 50ms, range: 30-100ms)

**Accessibility:**
- `FOCUS_RING_WIDTH_PX`: Width of focus ring on collapsed regions (default: 2px, range: 1-3px)
- `FOCUS_RING_OFFSET_PX`: Offset of focus ring from collapsed region edge (default: 1px, range: 0-2px)

---

**Sources:**
- [Diff Viewer for files | IntelliJ IDEA Documentation](https://www.jetbrains.com/help/idea/differences-viewer.html)
- [Diff & Merge | IntelliJ IDEA Documentation](https://www.jetbrains.com/help/idea/settings-tools-diff-and-merge.html)
- [React Diff Viewer - GitHub](https://github.com/praneshr/react-diff-viewer)
- [DiffCheck.io - Clear and Concise File Comparison](https://diffcheck.io/)
- [Kaleidoscope 6 - Michael Tsai Blog](https://mjtsai.com/blog/2025/05/28/kaleidoscope-6/)
- [VS Code Feature Request: Collapse unchanged fragments #39138](https://github.com/microsoft/vscode/issues/39138)
- [GitLab Diff Collapsing Issues)
