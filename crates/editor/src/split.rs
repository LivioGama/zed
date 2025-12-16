use buffer_diff::DiffHunkStatusKind;
use feature_flags::{FeatureFlag, FeatureFlagAppExt as _};
use gpui::{
    Action, AppContext as _, Background, Entity, EventEmitter, Focusable, Hsla, NoAction,
    PathBuilder, Pixels, Point as GpuiPoint, Subscription, WeakEntity, canvas, point, prelude::*,
    px, size,
};
use multi_buffer::{MultiBuffer, MultiBufferFilterMode};
use project::Project;
use std::ops::Range;
use theme::ActiveTheme;
use ui::{App, Context, Render, Window, div};
use workspace::{
    ActivePaneDecorator, Item, ItemHandle, Pane, PaneGroup, SplitDirection, Workspace,
};

use crate::{Editor, EditorEvent};

const BEZIER_SEGMENTS: usize = 48;
const CONNECTOR_BASE_CONTROL_OFFSET_RATIO: f32 = 0.35;
const CRUSHED_BLOCK_HEIGHT: f32 = 4.0;
const DIFF_HIGHLIGHT_ALPHA: f32 = 0.18;

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
}

struct DiffBlock {
    left_range: Range<usize>,
    right_range: Range<usize>,
    kind: DiffHunkStatusKind,
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
    _subscriptions: Vec<Subscription>,
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
                editor
            })
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

        let subscriptions =
            vec![
                cx.subscribe(&secondary_editor, |this, _, event: &EditorEvent, cx| {
                    if let EditorEvent::SelectionsChanged { .. } = event
                        && let Some(secondary) = &mut this.secondary
                    {
                        secondary.has_latest_selection = true;
                    }
                    cx.emit(event.clone())
                }),
            ];
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

    fn build_diff_blocks(&self, cx: &App) -> Vec<DiffBlock> {
        let Some(secondary) = &self.secondary else {
            return Vec::new();
        };

        let primary_buffer = self.primary_editor.read(cx).buffer();
        let primary_snapshot = primary_buffer.read(cx).snapshot(cx);
        let secondary_buffer = secondary.editor.read(cx).buffer();
        let secondary_snapshot = secondary_buffer.read(cx).snapshot(cx);

        let mut diff_blocks = Vec::new();

        let primary_hunks: Vec<_> = primary_snapshot.diff_hunks().collect();
        let secondary_hunks: Vec<_> = secondary_snapshot.diff_hunks().collect();

        for (primary_hunk, secondary_hunk) in primary_hunks.iter().zip(secondary_hunks.iter()) {
            let status = primary_hunk.status();
            let kind = status.kind;

            let right_start = primary_hunk.row_range.start.0 as usize;
            let right_end = primary_hunk.row_range.end.0 as usize;
            let left_start = secondary_hunk.row_range.start.0 as usize;
            let left_end = secondary_hunk.row_range.end.0 as usize;

            diff_blocks.push(DiffBlock {
                left_range: left_start..left_end,
                right_range: right_start..right_end,
                kind,
            });
        }

        diff_blocks
    }

    fn render_connector_overlay(&self, cx: &App) -> impl IntoElement {
        let left_editor = self.secondary.as_ref().unwrap().editor.clone();
        let right_editor = self.primary_editor.clone();
        let diff_blocks = self.build_diff_blocks(cx);
        let curves = build_connector_curves(&diff_blocks);
        let (deleted_bg, created_bg, modified_bg) = get_diff_colors(cx);

        canvas(
            move |bounds, window, cx| {
                let (left_line_height, left_scroll_pixels, left_editor_bounds) = left_editor
                    .update(cx, |editor, cx| {
                        let line_height = f32::from(
                            editor
                                .style(cx)
                                .text
                                .line_height_in_pixels(window.rem_size()),
                        );
                        let scroll_rows = editor.scroll_position(cx).y;
                        let scroll_pixels = (scroll_rows as f32) * line_height;
                        let editor_bounds = editor.last_bounds().cloned();
                        (line_height, scroll_pixels, editor_bounds)
                    });

                let (_right_line_height, right_scroll_pixels, right_editor_bounds) = right_editor
                    .update(cx, |editor, cx| {
                        let line_height = f32::from(
                            editor
                                .style(cx)
                                .text
                                .line_height_in_pixels(window.rem_size()),
                        );
                        let scroll_rows = editor.scroll_position(cx).y;
                        let scroll_pixels = (scroll_rows as f32) * line_height;
                        let editor_bounds = editor.last_bounds().cloned();
                        (line_height, scroll_pixels, editor_bounds)
                    });

                let line_height = left_line_height;
                let left_top_origin = left_editor_bounds
                    .as_ref()
                    .map(|b| f32::from(b.origin.y))
                    .unwrap_or(f32::from(bounds.origin.y));
                let right_top_origin = right_editor_bounds
                    .as_ref()
                    .map(|b| f32::from(b.origin.y))
                    .unwrap_or(f32::from(bounds.origin.y));

                ConnectorCanvasData {
                    curves,
                    line_height,
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
        .absolute()
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
        let header_height = data.left_top_origin - f32::from(bounds.origin.y);
        let viewport_top = header_height;
        let viewport_bottom = f32::from(bounds.size.height);

        let left_offset = data.left_top_origin - f32::from(bounds.origin.y);
        let right_offset = data.right_top_origin - f32::from(bounds.origin.y);

        let minimal_block_height = CRUSHED_BLOCK_HEIGHT;

        for curve in &data.curves {
            let left_row = curve.left_start as f32;
            let right_row = curve.right_start as f32;

            let left_y = (left_row * data.line_height) - data.left_scroll_pixels;
            let right_y = (right_row * data.line_height) - data.right_scroll_pixels;

            let left_bottom = if curve.left_crushed {
                left_y + minimal_block_height
            } else {
                ((curve.left_end as f32 + 1.0) * data.line_height - data.left_scroll_pixels)
                    .max(left_y + minimal_block_height)
            };

            let right_bottom = if curve.right_crushed {
                right_y + minimal_block_height
            } else {
                ((curve.right_end as f32 + 1.0) * data.line_height - data.right_scroll_pixels)
                    .max(right_y + minimal_block_height)
            };

            let adjusted_left_top = left_y + left_offset;
            let adjusted_left_bottom = left_bottom + left_offset;
            let adjusted_right_top = right_y + right_offset;
            let adjusted_right_bottom = right_bottom + right_offset;

            let connector_height = (adjusted_left_bottom - adjusted_left_top)
                .max(adjusted_right_bottom - adjusted_right_top);
            let base_control_offset = gutter_width * CONNECTOR_BASE_CONTROL_OFFSET_RATIO;
            let reference_line_height = data.line_height.max(1.0);
            let control_offset = if connector_height < reference_line_height * 2.0 {
                base_control_offset * (connector_height / (reference_line_height * 2.0)).max(0.3)
            } else {
                base_control_offset
            };

            let connector_top = adjusted_left_top.min(adjusted_right_top);
            let connector_bottom = adjusted_left_bottom.max(adjusted_right_bottom);

            let base_color = match curve.kind {
                ConnectorKind::Insert => data.created_bg,
                ConnectorKind::Delete => data.deleted_bg,
                ConnectorKind::Modify => data.modified_bg,
            };

            let is_visible = connector_bottom >= viewport_top && connector_top <= viewport_bottom;

            if is_visible {
                Self::draw_connector_ribbon(
                    window,
                    bounds,
                    adjusted_left_top,
                    adjusted_left_bottom,
                    adjusted_right_top,
                    adjusted_right_bottom,
                    control_offset,
                    base_color,
                    header_height,
                );
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
        header_height: f32,
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
            let clip_top = f32::from(bounds.origin.y) + header_height;
            let clip_bounds = gpui::Bounds {
                origin: point(px(f32::from(bounds.origin.x)), px(clip_top)),
                size: size(
                    bounds.size.width,
                    px(f32::from(bounds.size.height) - header_height),
                ),
            };

            window.with_content_mask(
                Some(gpui::ContentMask {
                    bounds: clip_bounds,
                }),
                |window| {
                    let background: Background = color.into();
                    window.paint_path(path, background);
                },
            );
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
        let has_secondary = self.secondary.is_some();
        let inner = if !has_secondary {
            self.primary_editor.clone().into_any_element()
        } else if let Some(active) = self.panes.panes().into_iter().next() {
            self.panes
                .render(
                    None,
                    &ActivePaneDecorator::new(active, &self.workspace),
                    window,
                    cx,
                )
                .into_any_element()
        } else {
            div().into_any_element()
        };

        let connector_overlay = if has_secondary {
            Some(self.render_connector_overlay(cx))
        } else {
            None
        };

        div()
            .id("splittable-editor")
            .relative()
            .on_action(cx.listener(Self::split))
            .on_action(cx.listener(Self::unsplit))
            .size_full()
            .child(inner)
            .when_some(connector_overlay, |this, overlay| this.child(overlay))
    }
}
