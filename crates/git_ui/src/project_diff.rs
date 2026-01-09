use crate::{
    conflict_view::ConflictAddon,
    git_panel::{GitPanel, GitPanelAddon, GitStatusEntry},
    git_panel_settings::GitPanelSettings,
    // remote_button::{render_publish_button, render_push_button}, // unused imports removed
    split_diff_settings::{SplitDiffSettings, SplitDiffViewMode},
};
use anyhow::{Context as _, Result, anyhow};
use buffer_diff::{BufferDiff, DiffHunkSecondaryStatus};
use collections::{HashMap, HashSet};
use diff_viewer::DiffViewer;
use editor::{
    Addon, Editor, EditorEvent, SelectionEffects, SplittableEditor,
    actions::{GoToHunk, GoToPreviousHunk},
    multibuffer_context_lines,
    scroll::Autoscroll,
};
use git::{
    StageAll, StageAndNext, ToggleStaged, UnstageAll, UnstageAndNext, repository::RepoPath,
    status::FileStatus,
};
use gpui::{
    Action, AnyElement, App, AppContext as _, AsyncApp, AsyncWindowContext, Entity, EventEmitter,
    FocusHandle, Focusable, Render, Subscription, Task, WeakEntity, actions,
};
use language::{Anchor, Buffer, Capability, OffsetRangeExt};
use multi_buffer::{MultiBuffer, PathKey};
use postage::prelude::Stream;
use project::{
    Project, ProjectPath,
    git_store::{
        Repository,
        branch_diff::{self, BranchDiffEvent, DiffBase},
    },
};
use settings::{Settings, SettingsStore};
use smol::future::yield_now;
use std::any::Any;
use std::path::Path;
use std::sync::Arc;
use theme::ActiveTheme;
use ui::{KeyBinding, Tooltip, prelude::*, vertical_divider};
use util::{ResultExt as _, paths::PathStyle, rel_path::RelPath};
use workspace::{
    ItemHandle, ToolbarItemEvent, ToolbarItemLocation, ToolbarItemView, Workspace,
    item::{Item, ItemEvent, TabContentParams},
    notifications::NotifyTaskExt,
    searchable::SearchableItemHandle,
};
use ztracing::instrument;

actions!(
    git,
    [
        /// Shows the diff between the working directory and the index.
        Diff,
        /// Adds files to the git staging area.
        Add,
        /// Shows the diff between the working directory and your default
        /// branch (typically main or master).
        BranchDiff,
        LeaderAndFollower,
        /// Toggle between unified and split diff view.
        ToggleSplitDiff,
    ]
);

#[derive(Clone, Debug)]
pub enum ProjectDiffEvent {
    ViewModeChanged,
    Editor(EditorEvent),
}

pub struct ProjectDiff {
    project: Entity<Project>,
    multibuffer: Entity<MultiBuffer>,
    branch_diff: Entity<branch_diff::BranchDiff>,
    editor: Entity<SplittableEditor>,
    buffer_diff_subscriptions: HashMap<Arc<RelPath>, (Entity<BufferDiff>, Subscription)>,
    pub workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    update_needed: postage::watch::Sender<()>,
    pending_scroll: Option<PathKey>,
    view_mode: SplitDiffViewMode,
    split_diff_view: Option<Entity<DiffViewer>>,
    toolbar: Entity<ProjectDiffToolbar>,
    _task: Task<Result<()>>,
    _subscription: Subscription,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshReason {
    DiffChanged,
    StatusesChanged,
    EditorSaved,
}

const CONFLICT_SORT_PREFIX: u64 = 1;
const TRACKED_SORT_PREFIX: u64 = 2;
const NEW_SORT_PREFIX: u64 = 3;

impl ProjectDiff {
    pub(crate) fn register(workspace: &mut Workspace, cx: &mut Context<Workspace>) {
        workspace.register_action(Self::deploy);
        workspace.register_action(Self::deploy_branch_diff);
        workspace.register_action(|workspace, _: &Add, window, cx| {
            Self::deploy(workspace, &Diff, window, cx);
        });
        workspace.register_action(|workspace, _: &ToggleSplitDiff, window, cx| {
            if let Some(active_item) = workspace.active_item(cx) {
                if let Some(project_diff) = active_item.downcast::<ProjectDiff>() {
                    project_diff.update(cx, |view, cx| {
                        view.toggle_split_diff(&ToggleSplitDiff, window, cx);
                    });
                }
            }
        });
        workspace::register_serializable_item::<ProjectDiff>(cx);

        let _weak_workspace = cx.entity().downgrade(); // unused variable prefixed with underscore
        // Register toolbar item on existing panes (REMOVED - now inline)
        // Leaving empty/removed
    }

    fn deploy(
        workspace: &mut Workspace,
        _: &Diff,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        Self::deploy_at(workspace, None, window, cx)
    }

    fn deploy_branch_diff(
        workspace: &mut Workspace,
        _: &BranchDiff,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        telemetry::event!("Git Branch Diff Opened");
        let project = workspace.project().clone();

        let existing = workspace
            .items_of_type::<Self>(cx)
            .find(|item| matches!(item.read(cx).diff_base(cx), DiffBase::Merge { .. }));
        if let Some(existing) = existing {
            workspace.activate_item(&existing, true, true, window, cx);
            return;
        }
        let workspace = cx.entity();
        window
            .spawn(cx, async move |cx| {
                let this = cx
                    .update(|window, cx| {
                        Self::new_with_default_branch(project, workspace.clone(), window, cx)
                    })?
                    .await?;
                workspace
                    .update_in(cx, |workspace, window, cx| {
                        workspace.add_item_to_active_pane(Box::new(this), None, true, window, cx);
                    })
                    .ok();
                anyhow::Ok(())
            })
            .detach_and_notify_err(window, cx);
    }

    pub fn deploy_at(
        workspace: &mut Workspace,
        entry: Option<GitStatusEntry>,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) {
        telemetry::event!(
            "Git Diff Opened",
            source = if entry.is_some() {
                "Git Panel"
            } else {
                "Action"
            }
        );
        let existing = workspace
            .items_of_type::<Self>(cx)
            .find(|item| matches!(item.read(cx).diff_base(cx), DiffBase::Head));
        let project_diff = if let Some(existing) = existing {
            existing.update(cx, |project_diff, cx| {
                project_diff.move_to_beginning(window, cx);
            });

            workspace.activate_item(&existing, true, true, window, cx);
            existing
        } else {
            let workspace_handle = cx.entity();
            let project_diff =
                cx.new(|cx| Self::new(workspace.project().clone(), workspace_handle, window, cx));
            workspace.add_item_to_active_pane(
                Box::new(project_diff.clone()),
                None,
                true,
                window,
                cx,
            );
            project_diff
        };
        if let Some(entry) = entry {
            project_diff.update(cx, |project_diff, cx| {
                project_diff.move_to_entry(entry, window, cx);
            })
        }
    }

    pub fn autoscroll(&self, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.primary_editor().update(cx, |editor, cx| {
                editor.request_autoscroll(Autoscroll::fit(), cx);
            })
        })
    }

    fn new_with_default_branch(
        project: Entity<Project>,
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<Result<Entity<Self>>> {
        let Some(repo) = project.read(cx).git_store().read(cx).active_repository() else {
            return Task::ready(Err(anyhow!("No active repository")));
        };
        let main_branch = repo.update(cx, |repo, _| repo.default_branch());

        let view_mode = SplitDiffSettings::get_global(cx).default_view.clone();

        window.spawn(cx, async move |cx| {
            let main_branch = main_branch
                .await??
                .context("Could not determine default branch")?;

            let branch_diff = cx.new_window_entity(|window, cx| {
                branch_diff::BranchDiff::new(
                    DiffBase::Merge {
                        base_ref: main_branch,
                    },
                    project.clone(),
                    window,
                    cx,
                )
            })?;
            cx.new_window_entity(|window, cx| {
                let mut diff = Self::new_impl(branch_diff, project, workspace, window, cx);

                diff.view_mode = view_mode.clone();
                if view_mode == SplitDiffViewMode::Split {
                    diff.create_split_diff_view(window, cx);
                }
                cx.notify();

                diff
            })
        })
    }

    fn new(
        project: Entity<Project>,
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let branch_diff =
            cx.new(|cx| branch_diff::BranchDiff::new(DiffBase::Head, project.clone(), window, cx));

        let view_mode = SplitDiffSettings::get_global(cx).default_view.clone();

        let mut diff = Self::new_impl(branch_diff, project, workspace, window, cx);

        diff.view_mode = view_mode.clone();
        if view_mode == SplitDiffViewMode::Split {
            diff.create_split_diff_view(window, cx);
        }

        diff
    }

    fn new_impl(
        branch_diff: Entity<branch_diff::BranchDiff>,
        project: Entity<Project>,
        workspace: Entity<Workspace>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let toolbar_focus_handle = focus_handle.clone();
        let multibuffer = cx.new(|cx| {
            let mut multibuffer = MultiBuffer::new(Capability::ReadWrite);
            multibuffer.set_all_diff_hunks_expanded(cx);
            multibuffer
        });

        let editor = cx.new(|cx| {
            let diff_display_editor = SplittableEditor::new_unsplit(
                multibuffer.clone(),
                project.clone(),
                workspace.clone(),
                window,
                cx,
            );
            diff_display_editor
                .primary_editor()
                .update(cx, |editor, cx| {
                    editor.disable_diagnostics(cx);

                    match branch_diff.read(cx).diff_base() {
                        DiffBase::Head => {
                            editor.register_addon(GitPanelAddon {
                                workspace: workspace.downgrade(),
                            });
                        }
                        DiffBase::Merge { .. } => {
                            editor.register_addon(BranchDiffAddon {
                                branch_diff: branch_diff.clone(),
                            });
                            editor.start_temporary_diff_override();
                            editor.set_render_diff_hunk_controls(
                                Arc::new(|_, _, _, _, _, _, _, _| gpui::Empty.into_any_element()),
                                cx,
                            );
                        }
                    }
                });
            diff_display_editor
        });
        cx.subscribe_in(&editor, window, |this, editor, event, window, cx| {
            this.handle_editor_event(editor, event, window, cx);
            cx.emit(ProjectDiffEvent::Editor(event.clone()));
        })
        .detach();

        let git_store = project.read(cx).git_store().clone();
        let _git_store_subscription = cx.subscribe_in(
            &git_store,
            window,
            move |this, _, event, window, cx| match event {
                project::git_store::GitStoreEvent::RepositoryUpdated(_, event, _) => match event {
                    project::git_store::RepositoryEvent::BranchChanged
                    | project::git_store::RepositoryEvent::StatusesChanged => {
                        *this.update_needed.borrow_mut() = ();
                    }
                    _ => {}
                },
                _ => {}
            },
        ); // unused variable prefixed with underscore

        let branch_diff_subscription = cx.subscribe_in(
            &branch_diff,
            window,
            move |this, _git_store, event, window, cx| match event {
                BranchDiffEvent::FileListChanged => {
                    *this.update_needed.borrow_mut() = ();
                }
            },
        );

        let (mut send, mut recv) = postage::watch::channel();
        let worker = window.spawn(cx, {
            let this = cx.weak_entity();
            async move |cx| {
                while let Some(()) = recv.recv().await {
                    let this = if let Some(this) = this.upgrade() {
                        this
                    } else {
                        break;
                    };
                    Self::refresh(this.downgrade(), RefreshReason::StatusesChanged, cx).await?;
                }
                Ok(())
            }
        });
        *send.borrow_mut() = ();

        let mut was_sort_by_path = GitPanelSettings::get_global(cx).sort_by_path;
        let mut was_collapse_untracked_diff =
            GitPanelSettings::get_global(cx).collapse_untracked_diff;
        cx.observe_global_in::<SettingsStore>(window, move |this, window, cx| {
            let is_sort_by_path = GitPanelSettings::get_global(cx).sort_by_path;
            let is_collapse_untracked_diff =
                GitPanelSettings::get_global(cx).collapse_untracked_diff;
            if is_sort_by_path != was_sort_by_path
                || is_collapse_untracked_diff != was_collapse_untracked_diff
            {
                *this.update_needed.borrow_mut() = ();
            }
            was_sort_by_path = is_sort_by_path;
            was_collapse_untracked_diff = is_collapse_untracked_diff;
        })
        .detach();

        Self {
            project,
            workspace: workspace.downgrade(),
            branch_diff,
            focus_handle,
            editor,
            multibuffer,
            buffer_diff_subscriptions: Default::default(),
            pending_scroll: None,
            view_mode: SplitDiffViewMode::Unified,
            split_diff_view: None,
            toolbar: cx
                .new(|cx| ProjectDiffToolbar::new(workspace.downgrade(), toolbar_focus_handle)),
            update_needed: send,
            _task: worker,
            _subscription: branch_diff_subscription,
        }
    }

    pub fn diff_base<'a>(&'a self, cx: &'a App) -> &'a DiffBase {
        self.branch_diff.read(cx).diff_base()
    }

    pub fn move_to_entry(
        &mut self,
        entry: GitStatusEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(git_repo) = self.branch_diff.read(cx).repo() else {
            return;
        };
        let repo = git_repo.read(cx);
        let sort_prefix = sort_prefix(repo, &entry.repo_path, entry.status, cx);
        let path_key = PathKey::with_sort_prefix(
            sort_prefix,
            Arc::from(
                RelPath::new(entry.repo_path.as_std_path(), PathStyle::Posix)
                    .unwrap()
                    .into_owned()
                    .as_rel_path(),
            ),
        );

        self.move_to_path(path_key, window, cx);

        if self.view_mode == SplitDiffViewMode::Split {
            self.update_split_diff_for_entry(&entry, window, cx);
        }
    }

    pub fn toggle_view_mode(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = match self.view_mode {
            SplitDiffViewMode::Unified => SplitDiffViewMode::Split,
            SplitDiffViewMode::Split => SplitDiffViewMode::Unified,
        };
        cx.notify();
    }

    pub fn tooltip_suffix(&self, _cx: &App) -> &'static str {
        "Hunk"
    }

    pub fn active_path(&self, cx: &App) -> Option<ProjectPath> {
        let editor = self.editor.read(cx).last_selected_editor().read(cx);
        let position = editor.selections.newest_anchor().head();
        let multi_buffer = editor.buffer().read(cx);
        let (_, buffer, _) = multi_buffer.excerpt_containing(position, cx)?;

        let file = buffer.read(cx).file()?;
        Some(ProjectPath {
            worktree_id: file.worktree_id(cx),
            path: file.path().clone(),
        })
    }

    fn move_to_beginning(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.primary_editor().update(cx, |editor, cx| {
                editor.move_to_beginning(&Default::default(), window, cx);
            });
        });
    }

    fn move_to_path(&mut self, path_key: PathKey, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(position) = self.multibuffer.read(cx).location_for_path(&path_key, cx) {
            self.editor.update(cx, |editor, cx| {
                editor.primary_editor().update(cx, |editor, cx| {
                    editor.change_selections(
                        SelectionEffects::scroll(Autoscroll::focused()),
                        window,
                        cx,
                        |s| {
                            s.select_ranges([position..position]);
                        },
                    )
                })
            });
        } else {
            self.pending_scroll = Some(path_key);
        }
    }

    fn toggle_split_diff(
        &mut self,
        _: &ToggleSplitDiff,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let new_view_mode = match self.view_mode {
            SplitDiffViewMode::Unified => SplitDiffViewMode::Split,
            SplitDiffViewMode::Split => SplitDiffViewMode::Unified,
        };

        self.view_mode = new_view_mode.clone();

        let fs = self.project.read(cx).fs().clone();
        let new_view_mode_clone = new_view_mode.clone();
        settings::update_settings_file(fs, cx, move |settings, _cx| {
            settings.git_split_diff.get_or_insert_default().default_view =
                Some(new_view_mode_clone);
        });

        match new_view_mode {
            SplitDiffViewMode::Unified => {
                self.split_diff_view = None;
            }
            SplitDiffViewMode::Split => {
                self.create_split_diff_view(window, cx);
            }
        }

        cx.emit(ProjectDiffEvent::ViewModeChanged);
        cx.notify();
    }

    fn render_split_view(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(split_diff_view) = &self.split_diff_view {
            div()
                .flex_1()
                .min_h_0()
                .w_full()
                .flex()
                .flex_col()
                .child(split_diff_view.clone())
        } else {
            div().flex_1().min_h_0().w_full().child(self.editor.clone())
        }
    }

    fn update_split_diff_for_entry(
        &mut self,
        entry: &GitStatusEntry,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(git_repo) = self.branch_diff.read(cx).repo() else {
            return;
        };

        let project_path = git_repo
            .read(cx)
            .repo_path_to_project_path(&entry.repo_path, cx);
        let Some(project_path) = project_path else {
            return;
        };

        self.update_split_diff_for_path(&project_path, window, cx);
    }

    fn update_split_diff_for_path(
        &mut self,
        project_path: &ProjectPath,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(git_repo) = self.branch_diff.read(cx).repo() else {
            return;
        };

        let repo_path = git_repo
            .read(cx)
            .project_path_to_repo_path(project_path, cx);
        let Some(repo_path) = repo_path else {
            return;
        };

        let project = self.project.clone();
        let git_repo_clone = git_repo.clone();
        let project_path_clone = project_path.clone();

        if let Some(viewer) = &self.split_diff_view {
            let viewer = viewer.clone();
            window
                .spawn(cx, async move |cx| {
                    let left_content = git_repo_clone
                        .update(cx, |repo, _cx| {
                            repo.get_committed_text(repo_path.clone(), _cx)
                        })?
                        .await;

                    let right_buffer = project
                        .update(cx, |project, cx| {
                            project.open_buffer(project_path_clone.clone(), cx)
                        })?
                        .await?;

                    let right_content = right_buffer.read_with(cx, |buffer, _| buffer.text())?;

                    viewer.update(cx, |viewer, cx| {
                        viewer.update_content(left_content, right_content, cx);
                        viewer.set_language_from_source_buffers(
                            Some(&right_buffer),
                            Some(&right_buffer),
                            cx,
                        );
                    })?;

                    anyhow::Ok(())
                })
                .detach_and_log_err(cx);
        }
    }

    fn create_split_diff_view(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let active_path = self.active_path(cx).or_else(|| {
            let multibuffer = self.multibuffer.read(cx);
            let paths = multibuffer.paths().collect::<Vec<_>>();
            if let Some(first_path) = paths.first() {
                if let Some(git_repo) = self.branch_diff.read(cx).repo() {
                    git_repo.read(cx).repo_path_to_project_path(
                        &RepoPath::from_rel_path(
                            &RelPath::unix(Path::new(first_path.path.as_ref().as_unix_str()))
                                .unwrap(),
                        ),
                        cx,
                    )
                } else {
                    None
                }
            } else {
                None
            }
        });

        let Some(active_path) = active_path else {
            return;
        };

        let Some(git_repo) = self.branch_diff.read(cx).repo().cloned() else {
            return;
        };

        let repo_path = git_repo
            .read(cx)
            .project_path_to_repo_path(&active_path, cx);
        let Some(repo_path) = repo_path else {
            return;
        };

        let project = self.project.clone();

        let view = cx.new(|cx| {
            let mut viewer = DiffViewer::new(None, None, window, cx);
            viewer.initialize(window, cx);
            viewer
        });
        self.split_diff_view = Some(view.clone());

        cx.notify();

        let git_repo_clone = git_repo.clone();
        let project_clone = project.clone();
        let active_path_clone = active_path.clone();
        let repo_path_clone = repo_path.clone();
        let view_clone = view.clone();
        let this = cx.weak_entity();

        window
            .spawn(cx, async move |cx| {
                let left_content = git_repo_clone
                    .update(cx, |repo, _cx| {
                        repo.get_committed_text(repo_path_clone.clone(), _cx)
                    })?
                    .await;

                let right_buffer: Entity<Buffer> = project_clone
                    .update(cx, |project, cx| {
                        project.open_buffer(active_path_clone.clone(), cx)
                    })?
                    .await?;

                let right_content = right_buffer.read_with(cx, |buffer, _| buffer.text())?;

                this.update(cx, |_, cx| {
                    view_clone.update(cx, |viewer, cx| {
                        viewer.update_content(left_content, right_content, cx);
                        viewer.set_language_from_source_buffers(
                            Some(&right_buffer),
                            Some(&right_buffer),
                            cx,
                        );
                    });
                })?;

                anyhow::Ok(())
            })
            .detach();
    }

    fn button_states(&self, cx: &App) -> ButtonStates {
        let editor = self.editor.read(cx).primary_editor().read(cx);
        let snapshot = self.multibuffer.read(cx).snapshot(cx);
        let prev_next = snapshot.diff_hunks().nth(1).is_some();
        let mut selection = true;

        let mut ranges = editor
            .selections
            .disjoint_anchor_ranges()
            .collect::<Vec<_>>();
        if !ranges.iter().any(|range| range.start != range.end) {
            selection = false;
            if let Some((excerpt_id, _, range)) = self
                .editor
                .read(cx)
                .primary_editor()
                .read(cx)
                .active_excerpt(cx)
            {
                ranges = vec![multi_buffer::Anchor::range_in_buffer(excerpt_id, range)];
            } else {
                ranges = Vec::default();
            }
        }
        let mut has_staged_hunks = false;
        let mut has_unstaged_hunks = false;
        for hunk in editor.diff_hunks_in_ranges(&ranges, &snapshot) {
            match hunk.secondary_status {
                DiffHunkSecondaryStatus::HasSecondaryHunk
                | DiffHunkSecondaryStatus::SecondaryHunkAdditionPending => {
                    has_unstaged_hunks = true;
                }
                DiffHunkSecondaryStatus::OverlapsWithSecondaryHunk => {
                    has_staged_hunks = true;
                    has_unstaged_hunks = true;
                }
                DiffHunkSecondaryStatus::NoSecondaryHunk
                | DiffHunkSecondaryStatus::SecondaryHunkRemovalPending => {
                    has_staged_hunks = true;
                }
            }
        }
        let mut stage_all = false;
        let mut unstage_all = false;
        self.workspace
            .read_with(cx, |workspace, cx| {
                if let Some(git_panel) = workspace.panel::<GitPanel>(cx) {
                    let git_panel = git_panel.read(cx);
                    stage_all = git_panel.can_stage_all();
                    unstage_all = git_panel.can_unstage_all();
                }
            })
            .ok();

        ButtonStates {
            stage: has_unstaged_hunks,
            unstage: has_staged_hunks,
            prev_next,
            selection,
            stage_all,
            unstage_all,
        }
    }

    fn handle_editor_event(
        &mut self,
        editor: &Entity<SplittableEditor>,
        event: &EditorEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            EditorEvent::SelectionsChanged { local: true } => {
                let Some(project_path) = self.active_path(cx) else {
                    return;
                };
                self.workspace
                    .update(cx, |workspace, cx| {
                        if let Some(git_panel) = workspace.panel::<GitPanel>(cx) {
                            git_panel.update(cx, |git_panel, cx| {
                                git_panel.select_entry_by_path(project_path.clone(), window, cx)
                            })
                        }
                    })
                    .ok();

                if self.view_mode == SplitDiffViewMode::Split {
                    self.update_split_diff_for_path(&project_path, window, cx);
                }
            }
            EditorEvent::Saved => {
                self._task = cx.spawn_in(window, async move |this, cx| {
                    Self::refresh(this, RefreshReason::EditorSaved, cx).await
                });
            }
            _ => {}
        }
        if editor.focus_handle(cx).contains_focused(window, cx)
            && self.multibuffer.read(cx).is_empty()
        {
            self.focus_handle.focus(window)
        }
    }

    #[instrument(skip_all)]
    fn register_buffer(
        &mut self,
        path_key: PathKey,
        file_status: FileStatus,
        buffer: Entity<Buffer>,
        diff: Entity<BufferDiff>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let subscription = cx.subscribe_in(&diff, window, move |this, _, _, window, cx| {
            this._task = window.spawn(cx, {
                let this = cx.weak_entity();
                async |cx| Self::refresh(this, RefreshReason::DiffChanged, cx).await
            })
        });
        self.buffer_diff_subscriptions
            .insert(path_key.path.clone(), (diff.clone(), subscription));

        // TODO(split-diff) we shouldn't have a conflict addon when split
        let conflict_addon = self
            .editor
            .read(cx)
            .primary_editor()
            .read(cx)
            .addon::<ConflictAddon>()
            .expect("project diff editor should have a conflict addon");

        let snapshot = buffer.read(cx).snapshot();
        let diff_read = diff.read(cx);

        let excerpt_ranges = {
            let diff_hunk_ranges = diff_read
                .hunks_intersecting_range(
                    Anchor::min_max_range_for_buffer(diff_read.buffer_id),
                    &snapshot,
                    cx,
                )
                .map(|diff_hunk| diff_hunk.buffer_range.to_point(&snapshot));
            let conflicts = conflict_addon
                .conflict_set(snapshot.remote_id())
                .map(|conflict_set| conflict_set.read(cx).snapshot().conflicts)
                .unwrap_or_default();
            let mut conflicts = conflicts
                .iter()
                .map(|conflict| conflict.range.to_point(&snapshot))
                .peekable();

            if conflicts.peek().is_some() {
                conflicts.collect::<Vec<_>>()
            } else {
                diff_hunk_ranges.collect()
            }
        };

        let multibuffer_was_empty = self.multibuffer.read(cx).is_empty();

        let (was_empty, is_excerpt_newly_added) = self.multibuffer.update(cx, |multibuffer, cx| {
            let was_empty = multibuffer.is_empty();
            let (_, is_newly_added) = multibuffer.set_excerpts_for_path(
                path_key.clone(),
                buffer,
                excerpt_ranges,
                multibuffer_context_lines(cx),
                cx,
            );
            if self.branch_diff.read(cx).diff_base().is_merge_base() {
                multibuffer.add_diff(diff.clone(), cx);
            }
            (was_empty, is_newly_added)
        });

        self.editor.update(cx, |editor, cx| {
            editor.primary_editor().update(cx, |editor, cx| {
                if was_empty {
                    editor.change_selections(
                        SelectionEffects::no_scroll(),
                        window,
                        cx,
                        |selections| {
                            selections.select_ranges([
                                multi_buffer::Anchor::min()..multi_buffer::Anchor::min()
                            ])
                        },
                    );
                }
                if is_excerpt_newly_added
                    && (file_status.is_deleted()
                        || (file_status.is_untracked()
                            && GitPanelSettings::get_global(cx).collapse_untracked_diff))
                {
                    editor.fold_buffer(snapshot.text.remote_id(), cx)
                }
            })
        });

        if self.multibuffer.read(cx).is_empty()
            && self
                .editor
                .read(cx)
                .focus_handle(cx)
                .contains_focused(window, cx)
        {
            self.focus_handle.focus(window);
        } else if self.focus_handle.is_focused(window) && !self.multibuffer.read(cx).is_empty() {
            self.editor.update(cx, |editor, cx| {
                editor.focus_handle(cx).focus(window);
            });
        }
        if self.pending_scroll.as_ref() == Some(&path_key) {
            self.move_to_path(path_key, window, cx);
        }

        if multibuffer_was_empty && !self.multibuffer.read(cx).is_empty() {
            if self.view_mode == SplitDiffViewMode::Split && self.split_diff_view.is_none() {
                self.create_split_diff_view(window, cx);
            }
        }
    }

    pub async fn refresh(
        this: WeakEntity<Self>,
        reason: RefreshReason,
        cx: &mut AsyncWindowContext,
    ) -> Result<()> {
        let mut path_keys = Vec::new();
        let buffers_to_load = this.update(cx, |this, cx| {
            let (repo, buffers_to_load) = this.branch_diff.update(cx, |branch_diff, cx| {
                let load_buffers = branch_diff.load_buffers(cx);
                (branch_diff.repo().cloned(), load_buffers)
            });
            let mut previous_paths = this
                .multibuffer
                .read(cx)
                .paths()
                .cloned()
                .collect::<HashSet<_>>();

            if let Some(repo) = repo {
                let repo = repo.read(cx);

                path_keys = Vec::with_capacity(buffers_to_load.len());
                for entry in buffers_to_load.iter() {
                    let sort_prefix = sort_prefix(&repo, &entry.repo_path, entry.file_status, cx);
                    let path_key =
                        PathKey::with_sort_prefix(sort_prefix, entry.repo_path.as_ref().clone());
                    previous_paths.remove(&path_key);
                    path_keys.push(path_key)
                }
            }

            this.multibuffer.update(cx, |multibuffer, cx| {
                for path in previous_paths {
                    if let Some(buffer) = multibuffer.buffer_for_path(&path, cx) {
                        let skip = match reason {
                            RefreshReason::DiffChanged | RefreshReason::EditorSaved => {
                                buffer.read(cx).is_dirty()
                            }
                            RefreshReason::StatusesChanged => false,
                        };
                        if skip {
                            continue;
                        }
                    }

                    this.buffer_diff_subscriptions.remove(&path.path);
                    multibuffer.remove_excerpts_for_path(path.clone(), cx);
                }
            });
            buffers_to_load
        })?;

        for (entry, path_key) in buffers_to_load.into_iter().zip(path_keys.into_iter()) {
            if let Some((buffer, diff)) = entry.load.await.log_err() {
                // We might be lagging behind enough that all future entry.load futures are no longer pending.
                // If that is the case, this task will never yield, starving the foreground thread of execution time.
                yield_now().await;
                cx.update(|window, cx| {
                    this.update(cx, |this, cx| {
                        let multibuffer = this.multibuffer.read(cx);
                        let skip = multibuffer.buffer(buffer.read(cx).remote_id()).is_some()
                            && multibuffer
                                .diff_for(buffer.read(cx).remote_id())
                                .is_some_and(|prev_diff| prev_diff.entity_id() == diff.entity_id())
                            && match reason {
                                RefreshReason::DiffChanged | RefreshReason::EditorSaved => {
                                    buffer.read(cx).is_dirty()
                                }
                                RefreshReason::StatusesChanged => false,
                            };
                        if !skip {
                            this.register_buffer(
                                path_key,
                                entry.file_status,
                                buffer,
                                diff,
                                window,
                                cx,
                            )
                        }
                    })
                    .ok();
                })?;
            }
        }
        this.update(cx, |this, cx| {
            this.pending_scroll.take();
            cx.notify();
        })?;

        Ok(())
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn excerpt_paths(&self, cx: &App) -> Vec<std::sync::Arc<util::rel_path::RelPath>> {
        self.multibuffer
            .read(cx)
            .paths()
            .map(|key| key.path.clone())
            .collect()
    }
}

fn sort_prefix(repo: &Repository, repo_path: &RepoPath, status: FileStatus, cx: &App) -> u64 {
    let settings = GitPanelSettings::get_global(cx);

    // Tree view can only sort by path
    if settings.sort_by_path || settings.tree_view {
        TRACKED_SORT_PREFIX
    } else if repo.had_conflict_on_last_merge_head_change(repo_path) {
        CONFLICT_SORT_PREFIX
    } else if status.is_created() {
        NEW_SORT_PREFIX
    } else {
        TRACKED_SORT_PREFIX
    }
}

impl EventEmitter<ProjectDiffEvent> for ProjectDiff {}

impl Focusable for ProjectDiff {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        if self.multibuffer.read(cx).is_empty() {
            self.focus_handle.clone()
        } else {
            self.editor.focus_handle(cx)
        }
    }
}

impl Item for ProjectDiff {
    type Event = ProjectDiffEvent;

    fn tab_icon(&self, _window: &Window, _cx: &App) -> Option<Icon> {
        Some(Icon::new(IconName::GitBranch).color(Color::Muted))
    }

    fn to_item_events(event: &ProjectDiffEvent, mut f: impl FnMut(ItemEvent)) {
        match event {
            ProjectDiffEvent::ViewModeChanged => f(ItemEvent::Edit),
            ProjectDiffEvent::Editor(editor_event) => Editor::to_item_events(editor_event, f),
        }
    }

    fn deactivated(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editor.update(cx, |editor, cx| {
            editor.primary_editor().update(cx, |primary_editor, cx| {
                primary_editor.deactivated(window, cx);
            })
        });
    }

    fn navigate(
        &mut self,
        data: Box<dyn Any>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        self.editor.update(cx, |editor, cx| {
            editor.primary_editor().update(cx, |primary_editor, cx| {
                primary_editor.navigate(data, window, cx)
            })
        })
    }

    fn tab_tooltip_text(&self, _: &App) -> Option<SharedString> {
        Some("Project Diff".into())
    }

    fn tab_content(&self, params: TabContentParams, _window: &Window, cx: &App) -> AnyElement {
        Label::new(self.tab_content_text(0, cx))
            .color(if params.selected {
                Color::Default
            } else {
                Color::Muted
            })
            .into_any_element()
    }

    fn tab_content_text(&self, _detail: usize, cx: &App) -> SharedString {
        match self.branch_diff.read(cx).diff_base() {
            DiffBase::Head => "Uncommitted Changes".into(),
            DiffBase::Merge { base_ref } => format!("Changes since {}", base_ref).into(),
        }
    }

    fn telemetry_event_text(&self) -> Option<&'static str> {
        Some("Project Diff Opened")
    }

    fn as_searchable(&self, _: &Entity<Self>, cx: &App) -> Option<Box<dyn SearchableItemHandle>> {
        // TODO(split-diff) SplitEditor should be searchable
        Some(Box::new(self.editor.read(cx).primary_editor().clone()))
    }
}

impl Render for ProjectDiff {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_empty = self.multibuffer.read(cx).is_empty();

        div()
            .key_context(if is_empty { "EmptyPane" } else { "GitDiff" })
            .bg(cx.theme().colors().editor_background)
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .when(is_empty, |el| {
                let remote_button = if let Some(panel) = self
                    .workspace
                    .upgrade()
                    .and_then(|w| w.read(cx).panel::<GitPanel>(cx))
                {
                    match self.branch_diff.read(cx).diff_base() {
                        DiffBase::Merge { base_ref } => {
                            let repo = panel.read(cx).active_repository.clone();
                            if let Some(repo) = repo {
                                let repo = repo.read(cx);
                                if let Some(branch) = repo.branch.as_ref() {
                                    if branch.name() == base_ref.as_ref() {
                                        Some(crate::remote_button::render_fetch_button(
                                            None,
                                            "project-diff-remote-button".into(),
                                        ))
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                        DiffBase::Head => None,
                    }
                } else {
                    None
                };

                el.child(
                    div()
                        .size_full()
                        .flex()
                        .flex_col()
                        .justify_center()
                        .items_center()
                        .gap_2()
                        .child(Label::new("No changes").color(Color::Muted))
                        .children(remote_button)
                        .child(
                            Button::new("open-git-status-picker", "Toggle Changes List")
                                .key_binding(KeyBinding::for_action(
                                    &crate::git_panel::ToggleFocus,
                                    cx,
                                ))
                                .on_click(|_, window, cx| {
                                    window.dispatch_action(
                                        Box::new(crate::git_panel::ToggleFocus),
                                        cx,
                                    );
                                }),
                        ),
                )
            })
            .when(!is_empty, |el| {
                el.child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .track_focus(&self.focus_handle)
                        .child(self.toolbar.clone())
                        .child(match self.view_mode {
                            SplitDiffViewMode::Unified => div()
                                .flex_1()
                                .min_h_0()
                                .w_full()
                                .child(self.editor.clone())
                                .into_any_element(),
                            SplitDiffViewMode::Split => {
                                self.render_split_view(window, cx).into_any_element()
                            }
                        }),
                )
            })
    }
}

pub struct ProjectDiffToolbar {
    project_diff: Option<WeakEntity<ProjectDiff>>,
    workspace: WeakEntity<Workspace>,
    focus_handle: FocusHandle,
    _subscription: Option<Subscription>,
}

impl ProjectDiffToolbar {
    pub fn new(workspace: WeakEntity<Workspace>, focus_handle: FocusHandle) -> Self {
        Self {
            project_diff: None,
            workspace,
            focus_handle,
            _subscription: None,
        }
    }

    fn project_diff(&self, cx: &App) -> Option<Entity<ProjectDiff>> {
        self.project_diff
            .as_ref()
            .and_then(|project_diff| project_diff.upgrade())
            .filter(|project_diff| {
                if let Some(workspace_project_diff) = self
                    .workspace
                    .upgrade()
                    .and_then(|workspace| workspace.read(cx).active_item(cx))
                    .and_then(|item| item.downcast::<ProjectDiff>())
                {
                    workspace_project_diff == *project_diff
                } else {
                    false
                }
            })
    }

    fn dispatch_action(&self, action: &dyn Action, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(project_diff) = self.project_diff(cx) {
            project_diff.focus_handle(cx).focus(window);
        }
        window.dispatch_action(action.boxed_clone(), cx);
    }

    fn stage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dispatch_action(&StageAll, window, cx)
    }

    fn unstage_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dispatch_action(&UnstageAll, window, cx)
    }

    fn prev_hunk(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dispatch_action(&GoToPreviousHunk, window, cx)
    }

    fn next_hunk(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dispatch_action(&GoToHunk, window, cx)
    }

    fn toggle_staged(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.dispatch_action(&ToggleStaged, window, cx)
    }
}

struct ButtonStates {
    stage: bool,
    unstage: bool,
    prev_next: bool,
    selection: bool,
    stage_all: bool,
    unstage_all: bool,
}

impl EventEmitter<ToolbarItemEvent> for ProjectDiffToolbar {}

impl ToolbarItemView for ProjectDiffToolbar {
    fn set_active_pane_item(
        &mut self,
        active_pane_item: Option<&dyn ItemHandle>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ToolbarItemLocation {
        if let Some(item) = active_pane_item {
            if let Some(diff) = item.downcast::<ProjectDiff>() {
                self.project_diff = Some(diff.downgrade());
                self.focus_handle = item.item_focus_handle(cx);
                self.workspace = diff.read(cx).workspace.clone();
                self._subscription = Some(cx.subscribe(&diff, |_, _, _, cx| {
                    cx.notify();
                }));
                return ToolbarItemLocation::PrimaryLeft;
            }
        }
        self.project_diff = None;
        self._subscription = None;
        ToolbarItemLocation::Hidden
    }
}

impl Render for ProjectDiffToolbar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(project_diff) = self.project_diff.as_ref().and_then(|p| p.upgrade()) else {
            return div();
        };

        let (button_states, tooltip_suffix) = project_diff.update(cx, |diff, cx| {
            (diff.button_states(cx), diff.tooltip_suffix(cx))
        });

        let focus_handle = self.focus_handle.clone();

        h_flex()
            .gap(DynamicSpacing::Base08.rems(cx))
            .child(
                h_group_sm().child(
                    Button::new("toggle_view_mode", "Inline/Side-by-side")
                        .tooltip(Tooltip::text("Toggle Inline/Side-by-side Diff"))
                        .on_click(cx.listener(|this, _, window, cx| {
                            if let Some(diff) = this.project_diff.as_ref().and_then(|p| p.upgrade())
                            {
                                diff.update(cx, |diff, cx| diff.toggle_view_mode(window, cx));
                            }
                        })),
                ),
            )
            .child(vertical_divider())
            .child(
                h_group_sm()
                    .child(
                        Button::new("prev_hunk", "Previous")
                            .tooltip(Tooltip::for_action_title_in(
                                "Go to Previous Hunk",
                                &GoToPreviousHunk,
                                &focus_handle,
                            ))
                            .key_binding(KeyBinding::for_action_in(
                                &GoToPreviousHunk,
                                &focus_handle,
                                cx,
                            ))
                            .disabled(!button_states.prev_next)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.prev_hunk(window, cx)),
                            ),
                    )
                    .child(
                        Button::new("next_hunk", "Next")
                            .tooltip(Tooltip::for_action_title_in(
                                "Go to Next Hunk",
                                &GoToHunk,
                                &focus_handle,
                            ))
                            .key_binding(KeyBinding::for_action_in(&GoToHunk, &focus_handle, cx))
                            .disabled(!button_states.prev_next)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.next_hunk(window, cx)),
                            ),
                    ),
            )
            .child(vertical_divider())
            .child(
                h_group_sm()
                    .child(
                        Button::new("stage", "Stage")
                            .disabled(!button_states.stage)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.dispatch_action(&StageAndNext, window, cx)
                            }))
                            .tooltip({
                                let focus_handle = focus_handle.clone();
                                move |_, cx| {
                                    Tooltip::for_action_in(
                                        format!("Stage {}", tooltip_suffix),
                                        &StageAndNext,
                                        &focus_handle,
                                        cx,
                                    )
                                }
                            }),
                    )
                    .child(
                        Button::new("unstage", "Unstage")
                            .disabled(!button_states.unstage)
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.dispatch_action(&UnstageAndNext, window, cx)
                            }))
                            .tooltip({
                                let focus_handle = focus_handle.clone();
                                move |_, cx| {
                                    Tooltip::for_action_in(
                                        format!("Unstage {}", tooltip_suffix),
                                        &UnstageAndNext,
                                        &focus_handle,
                                        cx,
                                    )
                                }
                            }),
                    )
                    .child(
                        Button::new("toggle_stage", "Toggle Stage")
                            .key_binding(KeyBinding::for_action_in(
                                &ToggleStaged,
                                &focus_handle,
                                cx,
                            ))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.toggle_staged(window, cx)),
                            )
                            .tooltip({
                                let focus_handle = focus_handle.clone();
                                move |_, cx| {
                                    Tooltip::for_action_in(
                                        format!("Toggle Stage for {}", tooltip_suffix),
                                        &ToggleStaged,
                                        &focus_handle,
                                        cx,
                                    )
                                }
                            }),
                    ),
            )
            .child(vertical_divider())
            .child(
                h_group_sm()
                    .child(
                        Button::new("stage_all", "Stage All")
                            .disabled(!button_states.stage_all)
                            .on_click(cx.listener(|this, _, window, cx| this.stage_all(window, cx)))
                            .tooltip({
                                let focus_handle = focus_handle.clone();
                                move |_, cx| {
                                    Tooltip::for_action_in(
                                        "Stage All Changes",
                                        &StageAll,
                                        &focus_handle,
                                        cx,
                                    )
                                }
                            }),
                    )
                    .child(
                        Button::new("unstage_all", "Unstage All")
                            .disabled(!button_states.unstage_all)
                            .on_click(
                                cx.listener(|this, _, window, cx| this.unstage_all(window, cx)),
                            )
                            .tooltip({
                                let focus_handle = focus_handle.clone();
                                move |_, cx| {
                                    Tooltip::for_action_in(
                                        "Unstage All Changes",
                                        &UnstageAll,
                                        &focus_handle,
                                        cx,
                                    )
                                }
                            }),
                    ),
            )
    }
}

struct BranchDiffAddon {
    branch_diff: Entity<branch_diff::BranchDiff>,
}

impl Addon for BranchDiffAddon {
    fn to_any(&self) -> &dyn Any {
        self
    }
}

impl EventEmitter<ItemEvent> for BranchDiffAddon {}

// impl ItemHandle for BranchDiffAddon {
//    fn item_id(&self) -> EntityId {
//        self.branch_diff.entity_id()
//    }
//
//    fn to_any(&self) -> AnyElement {
//        gpui::Empty.into_any_element()
//    }
// }

impl workspace::SerializableItem for ProjectDiff {
    fn serialized_item_kind() -> &'static str {
        "ProjectDiff"
    }

    fn should_serialize(&self, _event: &Self::Event) -> bool {
        false
    }

    fn cleanup(
        _workspace_id: workspace::WorkspaceId,
        _alive_items: Vec<workspace::ItemId>,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<anyhow::Result<()>> {
        Task::ready(Ok(()))
    }

    fn serialize(
        &mut self,
        _workspace: &mut Workspace,
        _item_id: workspace::ItemId,
        _closing: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Task<anyhow::Result<()>>> {
        None
    }

    fn deserialize(
        _project: Entity<Project>,
        _workspace: WeakEntity<Workspace>,
        _workspace_id: workspace::WorkspaceId,
        _item_id: workspace::ItemId,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Task<anyhow::Result<Entity<Self>>> {
        Task::ready(Err(anyhow::anyhow!("Not implemented")))
    }
}
