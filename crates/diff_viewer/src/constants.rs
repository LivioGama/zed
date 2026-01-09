/// Height in pixels for crushed blocks (empty ranges represented as thin lines)
pub const CRUSHED_BLOCK_HEIGHT: f32 = 2.0;

/// Width in pixels of the connector gutter between left and right editors
pub const CONNECTOR_GUTTER_WIDTH: f32 = 45.0;

/// Number of segments used to render Bezier curves for connectors
pub const BEZIER_SEGMENTS: usize = 48;

/// Alpha transparency value for diff highlight backgrounds
pub const DIFF_HIGHLIGHT_ALPHA: f32 = 0.5;

/// Base control point offset ratio for Bezier curves (relative to gutter width)
pub const CONNECTOR_BASE_CONTROL_OFFSET_RATIO: f32 = 0.25;

/// Thickness in pixels for crushed block indicator lines
pub const CRUSHED_THICKNESS: f32 = 2.0;

/// Default viewport height in pixels (fallback value)
pub const DEFAULT_VIEWPORT_HEIGHT: f32 = 600.0;

/// Default line height in pixels (fallback value)
pub const DEFAULT_LINE_HEIGHT: f32 = 22.0;

/// Number of context lines to preserve around changes
pub const CONTEXT_LINES: usize = 3;

/// Minimum threshold for unchanged lines to collapse
pub const MINIMUM_COLLAPSE_THRESHOLD: usize = 4;

/// Duration in milliseconds for collapse/expand animations
pub const COLLAPSE_DURATION_MS: u64 = 120;

/// Height multiplier for collapsed region indicator (relative to line height)
pub const COLLAPSED_REGION_HEIGHT_MULTIPLIER: f32 = 1.5;

/// Vertical padding inside collapsed region indicator
pub const COLLAPSED_REGION_PADDING_VERTICAL_PX: f32 = 5.0;

/// Horizontal padding inside collapsed region indicator
pub const COLLAPSED_REGION_PADDING_HORIZONTAL_PX: f32 = 10.0;

/// Opacity for text in collapsed region
pub const COLLAPSED_REGION_TEXT_OPACITY: f32 = 0.65;

/// Font size multiplier for collapsed region text
pub const COLLAPSED_REGION_FONT_SIZE_MULTIPLIER: f32 = 0.92;

/// Maximum line count to display before showing "9999+"
pub const MAX_LINE_COUNT_DISPLAY: usize = 9999;

/// Debounce delay for toggle clicks in milliseconds
pub const TOGGLE_DEBOUNCE_MS: u64 = 50;

/// Timeout for showing loading indicator if collapse calculation exceeds this
pub const COLLAPSE_CALCULATION_TIMEOUT_MS: u64 = 100;
