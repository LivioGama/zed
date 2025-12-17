use buffer_diff::DiffHunkStatusKind;
use feature_flags::{FeatureFlag, FeatureFlagAppExt as _};
use gpui::{
    Action, AppContext as _, Background, Entity, EventEmitter, Focusable, Hsla, NoAction,
    PathBuilder, Pixels, Point as GpuiPoint, Subscription, WeakEntity, canvas, point, prelude::*,
    px,
};
use multi_buffer::{MultiBuffer, MultiBufferFilterMode};
use project::Project;
use std::ops::Range;
use theme::ActiveTheme;
use ui::{App, Context, IconButton, IconName, IconSize, Render, Tooltip, Window, div, prelude::*};
use workspace::{Item, ItemHandle, Pane, PaneGroup, SplitDirection, Workspace};

use crate::{Editor, EditorEvent};

const BEZIER_SEGMENTS: usize = 48;
const CONNECTOR_BASE_CONTROL_OFFSET_RATIO: f32 = 0.35;
const CRUSHED_BLOCK_HEIGHT: f32 = 4.0;
const CRUSHED_THICKNESS: f32 = 2.0;
const DIFF_HIGHLIGHT_ALPHA: f32 = 0.18;
const CONNECTOR_GUTTER_WIDTH: f32 = 48.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnectorKind {
    Modify,
    Insert,
    Delete,
}

#[derive(Debug, Clone)]
struct ConnectorCurve {
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
    kind: ConnectorKind,
    left_crushed: bool,
    right_crushed: bool,
    block_index: usize,
}

struct DiffBlock {
    left_range: Range<usize>,
    right_range: Range<usize>,
    kind: DiffHunkStatusKind,
    index: usize,
}

#[derive(Clone)]
struct ConnectorCanvasData {
    curves: Vec<ConnectorCurve>,
    line_height: f32,
    left_scroll_pixels: f32,
    right_scroll_pixels: f32,
    left_top_origin: f32,
    right_top_origin: f32,
    created_bg: Hsla,
    deleted_bg: Hsla,
    modified_bg: Hsla,
}

fn build_connector_curves(blocks: &[DiffBlock]) -> Vec<ConnectorCurve> {
    blocks
        .iter()
        .filter_map(|block| {
            if block.left_range.is_empty() && block.right_range.is_empty() {
                return None;
            }

            let kind = match block.kind {
                DiffHunkStatusKind::Modified => ConnectorKind::Modify,
                DiffHunkStatusKind::Added => ConnectorKind::Insert,
                DiffHunkStatusKind::Deleted => ConnectorKind::Delete,
            };

            let left_crushed = block.left_range.is_empty();
            let right_crushed = block.right_range.is_empty();

            let left_start = block.left_range.start;
            let left_end = if left_crushed {
                left_start
            } else {
                block
                    .left_range
                    .end
                    .saturating_sub(1)
                    .max(block.left_range.start)
            };

            let right_start = block.right_range.start;
            let right_end = if right_crushed {
                right_start
            } else {
                block
                    .right_range
                    .end
                    .saturating_sub(1)
                    .max(block.right_range.start)
            };

            Some(ConnectorCurve {
                left_start,
                left_end,
                right_start,
                right_end,
                kind,
                left_crushed,
                right_crushed,
                block_index: block.index,
            })
        })
        .collect()
}

fn get_diff_colors(cx: &App) -> (Hsla, Hsla, Hsla) {
    let theme = cx.theme();
    let mut deleted_bg = theme.status().deleted_background;
    deleted_bg.a = DIFF_HIGHLIGHT_ALPHA;
    let mut created_bg = theme.status().created_background;
    created_bg.a = DIFF_HIGHLIGHT_ALPHA;
    let mut modified_bg = theme.status().modified_background;
    modified_bg.a = DIFF_HIGHLIGHT_ALPHA;
    (deleted_bg, created_bg, modified_bg)
}

fn cubic_bezier(
    p0: GpuiPoint<Pixels>,
    p1: GpuiPoint<Pixels>,
    p2: GpuiPoint<Pixels>,
    p3: GpuiPoint<Pixels>,
    t: f32,
) -> GpuiPoint<Pixels> {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    point(
        px(uuu * f32::from(p0.x)
            + 3.0 * uu * t * f32::from(p1.x)
            + 3.0 * u * tt * f32::from(p2.x)
            + ttt * f32::from(p3.x)),
        px(uuu * f32::from(p0.y)
            + 3.0 * uu * t * f32::from(p1.y)
            + 3.0 * u * tt * f32::from(p2.y)
            + ttt * f32::from(p3.y)),
    )
}

struct SplitDiffFeatureFlag;

impl FeatureFlag for SplitDiffFeatureFlag {
    const NAME: &'static str = "split-diff";

    fn enabled_for_staff() -> bool {
        true
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Action, Default)]
#[action(namespace = editor)]
struct SplitDiff;

#[derive(Clone, Copy, PartialEq, Eq, Action, Default)]
#[action(namespace = editor)]
struct UnsplitDiff;

pub struct SplittableEditor {
    primary_editor: Entity<Editor>,
    secondary: Option<SecondaryEditor>,
    panes: PaneGroup,
    workspace: WeakEntity<Workspace>,
    is_syncing_scroll: bool,
    pending_scroll_sync: Option<PendingScrollSync>,
    _subscriptions: Vec<Subscription>,
}

struct PendingScrollSync {
    target_editor: Entity<Editor>,
    scroll_position: gpui::Point<crate::scroll::ScrollOffset>,
}

struct SecondaryEditor {
    editor: Entity<Editor>,
    pane: Entity<Pane>,
    has_latest_selection: bool,
    _subscriptions: Vec<Subscription>,
}

impl SplittableEditor {
    pub fn primary_editor(&self) -> &Entity<Editor> {
        &self.primary_editor
    }

    pub fn last_selected_editor(&self) -> &Entity<Editor> {
        if let Some(secondary) = &self.secondary
            && secondary.has_latest_selection
        {
            &secondary.editor
        } else {
            &self.primary_editor
        }
    }

    pub fn new_unsplit(
        buffer: Entity<MultiBuffer>,
        project: Entity<Project>,
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let primary_editor =
            cx.new(|cx| Editor::for_multibuffer(buffer, Some(project.clone()), window, cx));
        let pane = cx.new(|cx| {
            let mut pane = Pane::new(
                workspace.downgrade(),
                project,
                Default::default(),
                None,
                NoAction.boxed_clone(),
                true,
                window,
                cx,
            );
            pane.set_should_display_tab_bar(|_, _| false);
            pane.add_item(primary_editor.boxed_clone(), true, true, None, window, cx);
            pane
        });
        let panes = PaneGroup::new(pane);
        // TODO(split-diff) we might want to tag editor events with whether they came from primary/secondary
        let subscriptions =
            vec![
                cx.subscribe(&primary_editor, |this, _, event: &EditorEvent, cx| {
                    if let EditorEvent::SelectionsChanged { .. } = event
                        && let Some(secondary) = &mut this.secondary
                    {
                        secondary.has_latest_selection = false;
                    }
                    cx.emit(event.clone())
                }),
            ];

        window.defer(cx, {
            let workspace = workspace.downgrade();
            let primary_editor = primary_editor.downgrade();
            move |window, cx| {
                workspace
                    .update(cx, |workspace, cx| {
                        primary_editor.update(cx, |editor, cx| {
                            editor.added_to_workspace(workspace, window, cx);
                        })
                    })
                    .ok();
            }
        });
        Self {
            primary_editor,
            secondary: None,
            panes,
            workspace: workspace.downgrade(),
            is_syncing_scroll: false,
            pending_scroll_sync: None,
            _subscriptions: subscriptions,
        }
    }

    fn split(&mut self, _: &SplitDiff, window: &mut Window, cx: &mut Context<Self>) {
        if !cx.has_flag::<SplitDiffFeatureFlag>() {
            return;
        }
        if self.secondary.is_some() {
            return;
        }
        let Some(workspace) = self.workspace.upgrade() else {
            return;
        };
        let project = workspace.read(cx).project().clone();
        let follower = self.primary_editor.update(cx, |primary, cx| {
            primary.buffer().update(cx, |buffer, cx| {
                let follower = buffer.get_or_create_follower(cx);
                buffer.set_filter_mode(Some(MultiBufferFilterMode::KeepInsertions));
                follower
            })
        });
        follower.update(cx, |follower, _| {
            follower.set_filter_mode(Some(MultiBufferFilterMode::KeepDeletions));
        });
        let secondary_editor = workspace.update(cx, |workspace, cx| {
            cx.new(|cx| {
                let mut editor = Editor::for_multibuffer(follower, Some(project), window, cx);
                // TODO(split-diff) this should be at the multibuffer level
                editor.set_use_base_text_line_numbers(true, cx);
                editor.added_to_workspace(workspace, window, cx);
                // Unfold all content for the split diff view
                let snapshot = editor.buffer().read(cx).snapshot(cx);
                editor.unfold_ranges(
                    &[multi_buffer::MultiBufferOffset(0)..snapshot.len()],
                    true,
                    false,
                    cx,
                );
                editor
            })
        });
        // Unfold all content in primary editor too
        self.primary_editor.update(cx, |editor, cx| {
            let snapshot = editor.buffer().read(cx).snapshot(cx);
            editor.unfold_ranges(
                &[multi_buffer::MultiBufferOffset(0)..snapshot.len()],
                true,
                false,
                cx,
            );
        });
        let secondary_pane = cx.new(|cx| {
            let mut pane = Pane::new(
                workspace.downgrade(),
                workspace.read(cx).project().clone(),
                Default::default(),
                None,
                NoAction.boxed_clone(),
                true,
                window,
                cx,
            );
            pane.set_should_display_tab_bar(|_, _| false);
            pane.add_item(
                ItemHandle::boxed_clone(&secondary_editor),
                false,
                false,
                None,
                window,
                cx,
            );
            pane
        });

        let subscriptions = vec![
            cx.subscribe(&secondary_editor, |this, _, event: &EditorEvent, cx| {
                if let EditorEvent::SelectionsChanged { .. } = event
                    && let Some(secondary) = &mut this.secondary
                {
                    secondary.has_latest_selection = true;
                }
                cx.emit(event.clone())
            }),
            // Sync scroll from secondary (left) to primary (right)
            cx.subscribe(
                &secondary_editor,
                |this, editor, event: &EditorEvent, cx| {
                    if let EditorEvent::ScrollPositionChanged { .. } = event {
                        this.sync_scroll_from_editor(editor.clone(), false, cx);
                    }
                },
            ),
        ];
        // Add scroll sync subscription from primary to secondary
        self._subscriptions.push(cx.subscribe(
            &self.primary_editor,
            |this, editor, event: &EditorEvent, cx| {
                if let EditorEvent::ScrollPositionChanged { .. } = event {
                    this.sync_scroll_from_editor(editor.clone(), true, cx);
                }
            },
        ));
        self.secondary = Some(SecondaryEditor {
            editor: secondary_editor,
            pane: secondary_pane.clone(),
            has_latest_selection: false,
            _subscriptions: subscriptions,
        });
        let primary_pane = self.panes.first_pane();
        self.panes
            .split(&primary_pane, &secondary_pane, SplitDirection::Left, cx)
            .unwrap();
        cx.notify();
    }

    fn unsplit(&mut self, _: &UnsplitDiff, _: &mut Window, cx: &mut Context<Self>) {
        let Some(secondary) = self.secondary.take() else {
            return;
        };
        self.panes.remove(&secondary.pane, cx).unwrap();
        self.primary_editor.update(cx, |primary, cx| {
            primary.buffer().update(cx, |buffer, _| {
                buffer.set_filter_mode(None);
            });
        });
        cx.notify();
    }

    fn sync_scroll_from_editor(
        &mut self,
        source_editor: Entity<Editor>,
        from_primary: bool,
        cx: &mut Context<Self>,
    ) {
        if self.is_syncing_scroll {
            return;
        }
        let Some(secondary) = &self.secondary else {
            return;
        };

        // Get scroll position from source editor
        let scroll_position = source_editor.update(cx, |editor, cx| editor.scroll_position(cx));

        let target_editor = if from_primary {
            secondary.editor.clone()
        } else {
            self.primary_editor.clone()
        };

        // Store pending scroll sync to be applied during render when we have window access
        self.pending_scroll_sync = Some(PendingScrollSync {
            target_editor,
            scroll_position,
        });
        cx.notify();
    }

    fn apply_pending_scroll_sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(pending) = self.pending_scroll_sync.take() {
            self.is_syncing_scroll = true;
            pending.target_editor.update(cx, |editor, cx| {
                editor.set_scroll_position(pending.scroll_position, window, cx);
            });
            self.is_syncing_scroll = false;
        }
    }

    pub fn added_to_workspace(
        &mut self,
        workspace: &mut Workspace,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.workspace = workspace.weak_handle();
        self.primary_editor.update(cx, |primary_editor, cx| {
            primary_editor.added_to_workspace(workspace, window, cx);
        });
        if let Some(secondary) = &self.secondary {
            secondary.editor.update(cx, |secondary_editor, cx| {
                secondary_editor.added_to_workspace(workspace, window, cx);
            });
        }
    }

    fn build_diff_blocks(&self, window: &mut Window, cx: &mut Context<Self>) -> Vec<DiffBlock> {
        let Some(secondary) = &self.secondary else {
            return Vec::new();
        };

        let mut diff_blocks = Vec::new();

        // Get display snapshots to convert buffer rows to display rows
        // This accounts for excerpt headers and other display-level transformations
        let (primary_hunks, primary_display) = self.primary_editor.update(cx, |editor, cx| {
            let buffer = editor.buffer().read(cx);
            let buffer_snapshot = buffer.snapshot(cx);
            let hunks: Vec<_> = buffer_snapshot.diff_hunks().collect();
            let display_snapshot = editor.snapshot(window, cx).display_snapshot;
            (hunks, display_snapshot)
        });

        let (secondary_hunks, secondary_display) = secondary.editor.update(cx, |editor, cx| {
            let buffer = editor.buffer().read(cx);
            let buffer_snapshot = buffer.snapshot(cx);
            let hunks: Vec<_> = buffer_snapshot.diff_hunks().collect();
            let display_snapshot = editor.snapshot(window, cx).display_snapshot;
            (hunks, display_snapshot)
        });

        for (index, (primary_hunk, secondary_hunk)) in
            primary_hunks.iter().zip(secondary_hunks.iter()).enumerate()
        {
            let status = primary_hunk.status();
            let kind = status.kind;

            // Convert buffer rows to display rows
            let right_start_point = crate::MultiBufferPoint::new(primary_hunk.row_range.start.0, 0);
            let right_end_point = crate::MultiBufferPoint::new(primary_hunk.row_range.end.0, 0);
            let left_start_point =
                crate::MultiBufferPoint::new(secondary_hunk.row_range.start.0, 0);
            let left_end_point = crate::MultiBufferPoint::new(secondary_hunk.row_range.end.0, 0);

            let right_start = primary_display
                .point_to_display_point(right_start_point, text::Bias::Left)
                .row()
                .0 as usize;
            let right_end = primary_display
                .point_to_display_point(right_end_point, text::Bias::Left)
                .row()
                .0 as usize;
            let left_start = secondary_display
                .point_to_display_point(left_start_point, text::Bias::Left)
                .row()
                .0 as usize;
            let left_end = secondary_display
                .point_to_display_point(left_end_point, text::Bias::Left)
                .row()
                .0 as usize;

            diff_blocks.push(DiffBlock {
                left_range: left_start..left_end,
                right_range: right_start..right_end,
                kind,
                index,
            });
        }

        diff_blocks
    }

    fn render_revert_buttons(
        &mut self,
        diff_blocks: &[DiffBlock],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<gpui::AnyElement> {
        let Some(secondary) = &self.secondary else {
            return Vec::new();
        };

        let curves = build_connector_curves(diff_blocks);

        let secondary_editor = secondary.editor.clone();

        // Get scroll and line height info - we need to match the connector drawing logic
        // The connectors calculate: left_offset = left_top_origin - gutter_origin_y
        // Then position at: (row * line_height) - scroll_pixels + left_offset
        // For buttons in a relative div, we use the same offset calculation
        let (line_height, left_scroll_pixels, left_editor_origin_y) =
            secondary.editor.update(cx, |editor, cx| {
                let line_height = f32::from(
                    editor
                        .style(cx)
                        .text
                        .line_height_in_pixels(window.rem_size()),
                );
                let scroll_rows = editor.scroll_position(cx).y;
                let scroll_pixels = (scroll_rows as f32) * line_height;
                let origin_y = editor
                    .last_bounds()
                    .map(|b| f32::from(b.origin.y))
                    .unwrap_or(0.0);
                (line_height, scroll_pixels, origin_y)
            });

        let (right_scroll_pixels, right_editor_origin_y) =
            self.primary_editor.update(cx, |editor, cx| {
                let line_height = f32::from(
                    editor
                        .style(cx)
                        .text
                        .line_height_in_pixels(window.rem_size()),
                );
                let scroll_rows = editor.scroll_position(cx).y;
                let scroll_pixels = (scroll_rows as f32) * line_height;
                let origin_y = editor
                    .last_bounds()
                    .map(|b| f32::from(b.origin.y))
                    .unwrap_or(0.0);
                (scroll_pixels, origin_y)
            });

        // The gutter div starts at the same Y as the editors (approximately)
        // We use the average editor origin as the gutter origin estimate
        let gutter_origin_y = (left_editor_origin_y + right_editor_origin_y) / 2.0;
        let left_offset = left_editor_origin_y - gutter_origin_y;
        let right_offset = right_editor_origin_y - gutter_origin_y;

        let mut buttons = Vec::new();

        for curve in curves.iter() {
            // Calculate the Y position for the button based on the connector center
            // This matches exactly how draw_connectors calculates positions
            let left_row = curve.left_start as f32;
            let right_row = curve.right_start as f32;

            let left_top = (left_row * line_height) - left_scroll_pixels + left_offset;
            let right_top = (right_row * line_height) - right_scroll_pixels + right_offset;

            let left_bottom = if curve.left_crushed {
                left_top + CRUSHED_BLOCK_HEIGHT
            } else {
                ((curve.left_end as f32 + 1.0) * line_height - left_scroll_pixels + left_offset)
                    .max(left_top + CRUSHED_BLOCK_HEIGHT)
            };

            let _right_bottom = if curve.right_crushed {
                right_top + CRUSHED_BLOCK_HEIGHT
            } else {
                ((curve.right_end as f32 + 1.0) * line_height - right_scroll_pixels + right_offset)
                    .max(right_top + CRUSHED_BLOCK_HEIGHT)
            };

            // Position button at the vertical center of the left side of the connector
            let left_center = (left_top + left_bottom) / 2.0;

            let button_size = 16.0;
            let button_top = left_center - button_size / 2.0;

            // Skip if button would be off-screen (using a reasonable viewport estimate)
            if button_top + button_size < 0.0 || button_top > 2000.0 {
                continue;
            }

            let block_index = curve.block_index;
            let secondary_editor_clone = secondary_editor.clone();

            let button = div()
                .id(("revert-btn", block_index))
                .absolute()
                .top(px(button_top))
                .left(px(-button_size - 4.0))
                .child(
                    IconButton::new(("revert", block_index), IconName::ArrowRight)
                        .icon_size(IconSize::XSmall)
                        .tooltip(Tooltip::text("Revert this change"))
                        .on_click(cx.listener(move |this, _event, window, cx| {
                            this.revert_hunk(block_index, &secondary_editor_clone, window, cx);
                        })),
                )
                .into_any_element();

            buttons.push(button);
        }

        buttons
    }

    fn revert_hunk(
        &mut self,
        block_index: usize,
        _secondary_editor: &Entity<Editor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Get the hunk at the given index from the primary editor
        let hunks = self.primary_editor.update(cx, |editor, cx| {
            let buffer = editor.buffer().read(cx);
            let snapshot = buffer.snapshot(cx);
            snapshot.diff_hunks().collect::<Vec<_>>()
        });

        let Some(hunk) = hunks.get(block_index) else {
            return;
        };

        // Use the existing prepare_restore_change mechanism
        let mut revert_changes = std::collections::HashMap::default();
        self.primary_editor.update(cx, |editor, cx| {
            editor.prepare_restore_change(&mut revert_changes, hunk, cx);
        });

        if !revert_changes.is_empty() {
            self.primary_editor.update(cx, |editor, cx| {
                editor.transact(window, cx, |editor, window, cx| {
                    editor.restore(revert_changes, window, cx);
                });
            });
        }
    }

    fn render_connector_overlay(&self, diff_blocks: &[DiffBlock], cx: &App) -> impl IntoElement {
        let left_editor = self.secondary.as_ref().unwrap().editor.clone();
        let right_editor = self.primary_editor.clone();
        let curves = build_connector_curves(diff_blocks);
        let (deleted_bg, created_bg, modified_bg) = get_diff_colors(cx);

        canvas(
            move |bounds, window, cx| {
                let (left_line_height, left_scroll_pixels, left_bounds) =
                    left_editor.update(cx, |editor, cx| {
                        let line_height = f32::from(
                            editor
                                .style(cx)
                                .text
                                .line_height_in_pixels(window.rem_size()),
                        );
                        let scroll_rows = editor.scroll_position(cx).y;
                        let scroll_pixels = (scroll_rows as f32) * line_height;
                        let bounds = editor.last_bounds().cloned();
                        (line_height, scroll_pixels, bounds)
                    });

                let (right_scroll_pixels, right_bounds) = right_editor.update(cx, |editor, cx| {
                    let line_height = f32::from(
                        editor
                            .style(cx)
                            .text
                            .line_height_in_pixels(window.rem_size()),
                    );
                    let scroll_rows = editor.scroll_position(cx).y;
                    let scroll_pixels = (scroll_rows as f32) * line_height;
                    let bounds = editor.last_bounds().cloned();
                    (scroll_pixels, bounds)
                });

                let left_top_origin = left_bounds
                    .as_ref()
                    .map(|b| f32::from(b.origin.y))
                    .unwrap_or(f32::from(bounds.origin.y));
                let right_top_origin = right_bounds
                    .as_ref()
                    .map(|b| f32::from(b.origin.y))
                    .unwrap_or(f32::from(bounds.origin.y));

                ConnectorCanvasData {
                    curves,
                    line_height: left_line_height,
                    left_scroll_pixels,
                    right_scroll_pixels,
                    left_top_origin,
                    right_top_origin,
                    created_bg,
                    deleted_bg,
                    modified_bg,
                }
            },
            move |bounds, data, window, _cx| {
                Self::draw_connectors(&bounds, &data, window);
            },
        )
        .size_full()
    }

    fn draw_connectors(
        bounds: &gpui::Bounds<Pixels>,
        data: &ConnectorCanvasData,
        window: &mut Window,
    ) {
        if data.curves.is_empty() {
            return;
        }

        let gutter_width = f32::from(bounds.size.width);
        let gutter_height = f32::from(bounds.size.height);
        let gutter_origin_y = f32::from(bounds.origin.y);
        let minimal_block_height = CRUSHED_BLOCK_HEIGHT;

        // Calculate how far down from the gutter top the editor content starts
        // This accounts for any header/toolbar above the editor content
        let left_offset = data.left_top_origin - gutter_origin_y;
        let right_offset = data.right_top_origin - gutter_origin_y;

        for curve in &data.curves {
            let left_row = curve.left_start as f32;
            let right_row = curve.right_start as f32;

            // Position relative to editor content, then add offset from gutter top
            let left_top = (left_row * data.line_height) - data.left_scroll_pixels + left_offset;
            let right_top =
                (right_row * data.line_height) - data.right_scroll_pixels + right_offset;

            let left_bottom = if curve.left_crushed {
                left_top + minimal_block_height
            } else {
                ((curve.left_end as f32 + 1.0) * data.line_height - data.left_scroll_pixels
                    + left_offset)
                    .max(left_top + minimal_block_height)
            };

            let right_bottom = if curve.right_crushed {
                right_top + minimal_block_height
            } else {
                ((curve.right_end as f32 + 1.0) * data.line_height - data.right_scroll_pixels
                    + right_offset)
                    .max(right_top + minimal_block_height)
            };

            let connector_height = (left_bottom - left_top).max(right_bottom - right_top);
            let base_control_offset = gutter_width * CONNECTOR_BASE_CONTROL_OFFSET_RATIO;
            let reference_line_height = data.line_height.max(1.0);
            let control_offset = if connector_height < reference_line_height * 2.0 {
                base_control_offset * (connector_height / (reference_line_height * 2.0)).max(0.3)
            } else {
                base_control_offset
            };

            let connector_top_y = left_top.min(right_top);
            let connector_bottom_y = left_bottom.max(right_bottom);

            let base_color = match curve.kind {
                ConnectorKind::Insert => data.created_bg,
                ConnectorKind::Delete => data.deleted_bg,
                ConnectorKind::Modify => data.modified_bg,
            };

            let is_visible = connector_bottom_y >= 0.0 && connector_top_y <= gutter_height;

            if is_visible {
                // Draw crushed indicator for pure insertions/deletions
                if curve.left_crushed || curve.right_crushed {
                    Self::draw_crushed_indicator(
                        window,
                        bounds,
                        curve.left_crushed,
                        curve.right_crushed,
                        left_top,
                        left_bottom,
                        right_top,
                        right_bottom,
                        base_color,
                    );
                }

                Self::draw_connector_ribbon(
                    window,
                    bounds,
                    left_top,
                    left_bottom,
                    right_top,
                    right_bottom,
                    control_offset,
                    base_color,
                );
            }
        }
    }

    fn draw_crushed_indicator(
        window: &mut Window,
        bounds: &gpui::Bounds<Pixels>,
        left_crushed: bool,
        right_crushed: bool,
        left_top: f32,
        left_bottom: f32,
        right_top: f32,
        right_bottom: f32,
        color: Hsla,
    ) {
        let gutter_x = f32::from(bounds.origin.x);
        let gutter_y = f32::from(bounds.origin.y);
        let gutter_width = f32::from(bounds.size.width);

        // For a crushed block on one side, draw a horizontal line indicator
        if left_crushed && !right_crushed {
            // Left side is empty (insertion on right) - draw indicator on left edge
            let y_center = gutter_y + (left_top + left_bottom) / 2.0;
            let indicator_width = gutter_width * 0.3;

            let mut builder = PathBuilder::fill();
            builder.move_to(point(px(gutter_x), px(y_center - CRUSHED_THICKNESS / 2.0)));
            builder.line_to(point(
                px(gutter_x + indicator_width),
                px(y_center - CRUSHED_THICKNESS / 2.0),
            ));
            builder.line_to(point(
                px(gutter_x + indicator_width),
                px(y_center + CRUSHED_THICKNESS / 2.0),
            ));
            builder.line_to(point(px(gutter_x), px(y_center + CRUSHED_THICKNESS / 2.0)));
            builder.close();

            if let Ok(path) = builder.build() {
                let background: Background = color.into();
                window.paint_path(path, background);
            }
        } else if right_crushed && !left_crushed {
            // Right side is empty (deletion on left) - draw indicator on right edge
            let y_center = gutter_y + (right_top + right_bottom) / 2.0;
            let indicator_width = gutter_width * 0.3;

            let mut builder = PathBuilder::fill();
            builder.move_to(point(
                px(gutter_x + gutter_width - indicator_width),
                px(y_center - CRUSHED_THICKNESS / 2.0),
            ));
            builder.line_to(point(
                px(gutter_x + gutter_width),
                px(y_center - CRUSHED_THICKNESS / 2.0),
            ));
            builder.line_to(point(
                px(gutter_x + gutter_width),
                px(y_center + CRUSHED_THICKNESS / 2.0),
            ));
            builder.line_to(point(
                px(gutter_x + gutter_width - indicator_width),
                px(y_center + CRUSHED_THICKNESS / 2.0),
            ));
            builder.close();

            if let Ok(path) = builder.build() {
                let background: Background = color.into();
                window.paint_path(path, background);
            }
        }
    }

    fn draw_connector_ribbon(
        window: &mut Window,
        bounds: &gpui::Bounds<Pixels>,
        left_top: f32,
        left_bottom: f32,
        right_top: f32,
        right_bottom: f32,
        control_offset: f32,
        color: Hsla,
    ) {
        let segments = BEZIER_SEGMENTS;
        let mut builder = PathBuilder::fill();

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let top_point = cubic_bezier(
                point(
                    px(f32::from(bounds.origin.x)),
                    px(f32::from(bounds.origin.y) + left_top),
                ),
                point(
                    px(f32::from(bounds.origin.x) + control_offset),
                    px(f32::from(bounds.origin.y) + left_top),
                ),
                point(
                    px(f32::from(bounds.origin.x) + f32::from(bounds.size.width) - control_offset),
                    px(f32::from(bounds.origin.y) + right_top),
                ),
                point(
                    px(f32::from(bounds.origin.x) + f32::from(bounds.size.width)),
                    px(f32::from(bounds.origin.y) + right_top),
                ),
                t,
            );
            if i == 0 {
                builder.move_to(top_point);
            } else {
                builder.line_to(top_point);
            }
        }

        for i in (0..=segments).rev() {
            let t = i as f32 / segments as f32;
            let bottom_point = cubic_bezier(
                point(
                    px(f32::from(bounds.origin.x)),
                    px(f32::from(bounds.origin.y) + left_bottom),
                ),
                point(
                    px(f32::from(bounds.origin.x) + control_offset),
                    px(f32::from(bounds.origin.y) + left_bottom),
                ),
                point(
                    px(f32::from(bounds.origin.x) + f32::from(bounds.size.width) - control_offset),
                    px(f32::from(bounds.origin.y) + right_bottom),
                ),
                point(
                    px(f32::from(bounds.origin.x) + f32::from(bounds.size.width)),
                    px(f32::from(bounds.origin.y) + right_bottom),
                ),
                t,
            );
            builder.line_to(bottom_point);
        }

        if let Ok(path) = builder.build() {
            window.with_content_mask(Some(gpui::ContentMask { bounds: *bounds }), |window| {
                let background: Background = color.into();
                window.paint_path(path, background);
            });
        }
    }
}

impl EventEmitter<EditorEvent> for SplittableEditor {}
impl Focusable for SplittableEditor {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.primary_editor.read(cx).focus_handle(cx)
    }
}

impl Render for SplittableEditor {
    fn render(
        &mut self,
        window: &mut ui::Window,
        cx: &mut ui::Context<Self>,
    ) -> impl ui::IntoElement {
        // Apply any pending scroll sync now that we have window access
        self.apply_pending_scroll_sync(window, cx);

        let has_secondary = self.secondary.is_some();

        // Get values that need cx before render_connector_overlay borrows it
        let split_listener = cx.listener(Self::split);
        let unsplit_listener = cx.listener(Self::unsplit);
        let surface_bg = cx.theme().colors().surface_background;

        if !has_secondary {
            return div()
                .id("splittable-editor")
                .on_action(split_listener)
                .on_action(unsplit_listener)
                .size_full()
                .child(self.primary_editor.clone())
                .into_any_element();
        }

        let secondary_editor = self.secondary.as_ref().unwrap().editor.clone();
        let primary_editor = self.primary_editor.clone();

        // Build diff blocks once and use for both connectors and buttons
        let diff_blocks = self.build_diff_blocks(window, cx);

        // Render buttons first (needs &mut self)
        let revert_buttons = self.render_revert_buttons(&diff_blocks, window, cx);

        // Render connector overlay (only needs &self) - convert to AnyElement to end borrow
        let connector_overlay = self
            .render_connector_overlay(&diff_blocks, cx)
            .into_any_element();

        div()
            .id("splittable-editor")
            .on_action(split_listener)
            .on_action(unsplit_listener)
            .size_full()
            .flex()
            .flex_row()
            .child(div().flex_1().min_w_0().child(secondary_editor))
            .child(
                div()
                    .w(px(CONNECTOR_GUTTER_WIDTH))
                    .bg(surface_bg)
                    .relative()
                    .child(connector_overlay)
                    .children(revert_buttons),
            )
            .child(div().flex_1().min_w_0().child(primary_editor))
            .into_any_element()
    }
}
