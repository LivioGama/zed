mod git_modals;
mod graph;
mod graph_rendering;

use git_modals::{GitModal, ModalAction};
use graph::format_timestamp;

use git::repository::{CommitDiff, CommitFile, LogOrder, LogSource};
use git::{
    BuildCommitPermalinkParams, GitHostingProviderRegistry, GitRemote, ParsedGitRemote,
    parse_git_remote_url,
};
use git_ui::commit_tooltip::CommitAvatar;
use gpui::{
    AnyElement, App, AppContext as _, ClipboardItem, Corner, DismissEvent, Entity, EventEmitter,
    FocusHandle, Focusable, FontWeight, Point, Render, ScrollWheelEvent, Subscription, Task,
    WeakEntity, actions, anchored, deferred, px,
};
use graph_rendering::accent_colors_count;
use project::git_store::CommitDataState;
use project::{
    Project,
    git_store::{GitStoreEvent, Repository, RepositoryEvent},
};
use settings::Settings;
use std::{collections::HashMap, ops::Range};
use theme::ThemeSettings;
use time::{OffsetDateTime, UtcOffset};
use ui::{ContextMenu, ScrollableHandle, Table, TableInteractionState, Tooltip, prelude::*};
use util::TryFutureExt;

#[derive(Debug)]
struct TreeNode {
    children: HashMap<String, TreeNode>,
    file: Option<CommitFile>,
}
use workspace::{
    Workspace,
    item::{Item, ItemEvent, SerializableItem},
};

use crate::{graph::AllCommitCount, graph_rendering::render_graph};

/// Action to set the context branch before performing branch operations.
#[derive(Clone, PartialEq, Debug, serde::Deserialize, schemars::JsonSchema, gpui::Action)]
pub struct SetContextBranch {
    pub branch_name: SharedString,
    pub is_remote: bool,
}

actions!(
    git_graph,
    [
        /// Opens the Git Graph panel.
        OpenGitGraph,
        /// Opens the commit view for the selected commit.
        OpenCommitView,
        /// Checkout/switch to a branch.
        CheckoutBranch,
        /// Checkout a specific commit revision (detached HEAD).
        CheckoutRevision,
        /// Pull changes from remote with smart stash/unstash.
        PullWithStash,
        /// Merge a branch into current.
        MergeBranch,
        /// Rebase current branch onto another.
        RebaseOnto,
        /// Squash selected commits.
        SquashCommits,
        /// Drop selected commits.
        DropCommits,
        /// Reword selected commits.
        RewordCommits,
        /// Edit/amend the selected commit.
        EditAmendCommit,
        /// Cherry-pick selected commits.
        CherryPick,
        /// Revert selected commits.
        RevertCommits,
        /// Delete a branch.
        DeleteBranch,
        /// Navigate to previous commit.
        SelectPreviousCommit,
        /// Navigate to next commit.
        SelectNextCommit,
        /// Extend selection to previous commit.
        SelectPreviousCommitExtend,
        /// Extend selection to next commit.
        SelectNextCommitExtend,
    ]
);

pub fn init(cx: &mut App) {
    workspace::register_serializable_item::<GitGraph>(cx);

    cx.observe_new(|workspace: &mut workspace::Workspace, _, _| {
        workspace.register_action(|workspace, _: &OpenGitGraph, window, cx| {
            let project = workspace.project().clone();
            let git_graph = cx.new(|cx| GitGraph::new(project, window, cx));
            workspace.add_item_to_active_pane(Box::new(git_graph), None, true, window, cx);
        });
    })
    .detach();

    // Register keyboard navigation actions
    cx.observe_new(|workspace: &mut workspace::Workspace, _, _| {
        workspace.register_action(|workspace, _: &SquashCommits, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.squash_commits(window, cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &DropCommits, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.drop_commits(window, cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &RewordCommits, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.reword_commits(window, cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &EditAmendCommit, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.edit_amend_commit(window, cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &SelectNextCommit, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.select_next_commit(cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &SelectPreviousCommitExtend, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.select_previous_commit_extend(cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &SelectNextCommitExtend, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.select_next_commit_extend(cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &CherryPick, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.cherry_pick(window, cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &CherryPick, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.cherry_pick(window, cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &RevertCommits, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| this.revert_commits(window, cx))
                    });
                }
            });
        });
        workspace.register_action(|workspace, action: &SetContextBranch, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| {
                            this.context_branch_name = Some(action.branch_name.clone());
                            this.context_is_remote = action.is_remote;
                        })
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &CheckoutBranch, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| {
                            if let Some(branch_name) = this.context_branch_name.clone() {
                                this.checkout_branch(branch_name, window, cx);
                            }
                        })
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &MergeBranch, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| {
                            if let Some(branch_name) = this.context_branch_name.clone() {
                                this.merge_branch(branch_name, window, cx);
                            }
                        })
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &RebaseOnto, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| {
                            if let Some(branch_name) = this.context_branch_name.clone() {
                                this.rebase_onto(branch_name, window, cx);
                            }
                        })
                    });
                }
            });
        });
        workspace.register_action(|workspace, _: &CheckoutRevision, window, cx| {
            let pane = workspace.active_pane();
            pane.update(cx, |pane, cx| {
                if let Some(item) = pane.active_item() {
                    item.downcast::<GitGraph>().map(|git_graph| {
                        git_graph.update(cx, |this, cx| {
                            this.checkout_revision(window, cx);
                        })
                    });
                }
            });
        });
    })
    .detach();
}

pub struct GitGraph {
    focus_handle: FocusHandle,
    graph: crate::graph::GitGraph,
    project: Entity<Project>,
    loading: bool,
    error: Option<SharedString>,
    context_menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    context_branch_name: Option<SharedString>,
    context_is_remote: bool,
    row_height: Pixels,
    table_interaction_state: Entity<TableInteractionState>,
    horizontal_scroll_offset: Pixels,
    graph_viewport_width: Pixels,
    selected_entry_idx: Option<usize>,
    selected_entry_indices: Vec<usize>,
    selected_branches: Vec<SharedString>,
    modal: Option<Entity<GitModal>>,
    log_source: LogSource,
    log_order: LogOrder,
    selected_commit_diff: Option<CommitDiff>,
    _commit_diff_task: Option<Task<()>>,
    _load_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl GitGraph {
    pub fn new(project: Entity<Project>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        cx.on_focus(&focus_handle, window, |_, _, cx| cx.notify())
            .detach();

        let git_store = project.read(cx).git_store().clone();
        let accent_colors = cx.theme().accents();
        let mut graph = crate::graph::GitGraph::new(accent_colors_count(accent_colors));
        let log_source = LogSource::default();
        let log_order = LogOrder::default();

        cx.subscribe(&git_store, |this, _, event, cx| match event {
            GitStoreEvent::RepositoryUpdated(_, RepositoryEvent::BranchChanged, true) => {
                // todo! only call load data from render, we should set a bool here
                // todo! We should check that the repo actually has a change that would affect the graph
                this.graph.clear();
                cx.notify();
            }
            GitStoreEvent::ActiveRepositoryChanged(repo_id) => {
                this.graph.clear();
                this._subscriptions.clear();
                cx.notify();

                if let Some(repository) = this.project.read(cx).active_repository(cx) {
                    // todo! we can merge the repository event handler with the git store events
                    this._subscriptions =
                        vec![cx.subscribe(&repository, Self::on_repository_event)];
                }
            }
            // todo! active repository has changed we should invalidate the graph state and reset our repo subscription
            _ => {}
        })
        .detach();

        let _subscriptions = if let Some(repository) = project.read(cx).active_repository(cx) {
            repository.update(cx, |repository, cx| {
                let commits =
                    repository.graph_data(log_source.clone(), log_order, 0..usize::MAX, cx);
                graph.add_commits(commits);
            });

            vec![cx.subscribe(&repository, Self::on_repository_event)]
        } else {
            vec![]
        };

        let settings = ThemeSettings::get_global(cx);
        let font_size = settings.buffer_font_size(cx);
        let row_height = font_size + px(12.0);

        let table_interaction_state = cx.new(|cx| TableInteractionState::new(cx));

        GitGraph {
            focus_handle,
            project,
            graph,
            loading: true,
            error: None,
            _load_task: None,
            _commit_diff_task: None,
            context_menu: None,
            context_branch_name: None,
            context_is_remote: false,
            row_height,
            table_interaction_state,
            horizontal_scroll_offset: px(0.),
            graph_viewport_width: px(88.),
            selected_entry_idx: None,
            selected_entry_indices: Vec::new(),
            selected_branches: Vec::new(),
            modal: None,
            selected_commit_diff: None,
            log_source,
            log_order,
            _subscriptions,
        }
    }

    fn on_repository_event(
        &mut self,
        repository: Entity<Repository>,
        event: &RepositoryEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            RepositoryEvent::GitGraphCountUpdated(_, commit_count) => {
                let old_count = self.graph.commits.len();

                repository.update(cx, |repository, cx| {
                    let commits = repository.graph_data(
                        self.log_source.clone(),
                        self.log_order,
                        old_count..*commit_count,
                        cx,
                    );
                    self.graph.add_commits(commits);
                });

                self.graph.max_commit_count = AllCommitCount::Loaded(*commit_count);
            }
            _ => {}
        }
    }

    fn render_badge(
        &self,
        name: &SharedString,
        accent_color: gpui::Hsla,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let branch_name = name.clone();
        // Strip "HEAD -> " prefix for display
        let display_name: SharedString = if let Some(stripped) = name.strip_prefix("HEAD -> ") {
            stripped.to_string().into()
        } else {
            name.clone()
        };
        let is_remote = name.starts_with("remotes/") || name.contains("origin/");
        let is_selected = self.selected_branches.contains(&branch_name);

        div()
            .px_1p5()
            .py_0p5()
            // todo! height should probably be based off of font size
            .h(px(22.0))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .bg(if is_selected {
                accent_color.opacity(0.35)
            } else {
                accent_color.opacity(0.18)
            })
            .border_1()
            .border_color(if is_selected {
                accent_color.opacity(0.75)
            } else {
                accent_color.opacity(0.55)
            })
            .child(
                Label::new(display_name)
                    .size(LabelSize::Small)
                    .color(Color::Default)
                    .single_line(),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener({
                    let value = branch_name.clone();
                    move |this, event: &gpui::MouseDownEvent, window, cx| {
                        if event.modifiers.secondary() {
                            // Multi-select mode: toggle selection
                            if let Some(pos) =
                                this.selected_branches.iter().position(|b| b == &value)
                            {
                                this.selected_branches.remove(pos);
                            } else {
                                this.selected_branches.push(value.clone());
                            }
                        } else {
                            // Single-select mode: clear and select only this branch
                            this.selected_branches.clear();
                            this.selected_branches.push(value.clone());
                        }
                        cx.notify();
                    }
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Right,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    this.show_context_menu_for_branch(
                        branch_name.clone(),
                        is_remote,
                        event.position,
                        window,
                        cx,
                    );
                }),
            )
    }

    fn render_table_rows(
        &mut self,
        range: Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<Vec<AnyElement>> {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let row_height = self.row_height;

        // We fetch data outside the visible viewport to avoid loading entries when
        // users scroll through the git graph
        if let Some(repository) = repository.as_ref() {
            const FETCH_RANGE: usize = 100;
            repository.update(cx, |repository, cx| {
                self.graph.commits[range.start.saturating_sub(FETCH_RANGE)
                    ..(range.end + FETCH_RANGE).min(self.graph.commits.len().saturating_sub(1))]
                    .iter()
                    .for_each(|commit| {
                        repository.fetch_commit_data(commit.data.sha, cx);
                    });
            });
        }

        range
            .map(|idx| {
                let Some((commit, repository)) =
                    self.graph.commits.get(idx).zip(repository.as_ref())
                else {
                    return vec![
                        div().h(row_height).into_any_element(),
                        div().h(row_height).into_any_element(),
                        div().h(row_height).into_any_element(),
                        div().h(row_height).into_any_element(),
                    ];
                };

                let commit_data = repository.update(cx, |repository, cx| {
                    repository.fetch_commit_data(commit.data.sha, cx).clone()
                });

                let short_sha = commit.data.sha.display_short();
                let mut formatted_time = String::new();
                let subject;
                let author_name;

                if let CommitDataState::Loaded(data) = commit_data {
                    subject = data.subject.clone();
                    author_name = data.author_name.clone();
                    formatted_time = format_timestamp(data.commit_timestamp);
                } else {
                    subject = "Loading...".into();
                    author_name = "".into();
                }

                let accent_colors = cx.theme().accents();
                let accent_color = accent_colors
                    .0
                    .get(commit.color_idx)
                    .copied()
                    .unwrap_or_else(|| accent_colors.0.first().copied().unwrap_or_default());
                let is_selected = self.selected_entry_indices.contains(&idx);
                let text_color = if is_selected {
                    Color::Default
                } else {
                    Color::Muted
                };

                vec![
                    div()
                        .id(ElementId::NamedInteger("commit-subject".into(), idx as u64))
                        .overflow_hidden()
                        .tooltip(Tooltip::text(subject.clone()))
                        .child(
                            h_flex()
                                .gap_1()
                                .items_center()
                                .overflow_hidden()
                                .children((!commit.data.ref_names.is_empty()).then(|| {
                                    h_flex().flex_shrink().gap_2().items_center().children(
                                        commit
                                            .data
                                            .ref_names
                                            .iter()
                                            .map(|name| self.render_badge(name, accent_color, cx))
                                            .collect::<Vec<_>>(),
                                    )
                                }))
                                .child(
                                    Label::new(subject)
                                        .color(text_color)
                                        .truncate()
                                        .single_line(),
                                ),
                        )
                        .into_any_element(),
                    Label::new(formatted_time)
                        .color(text_color)
                        .single_line()
                        .into_any_element(),
                    Label::new(author_name)
                        .color(text_color)
                        .single_line()
                        .into_any_element(),
                    Label::new(short_sha)
                        .color(text_color)
                        .single_line()
                        .into_any_element(),
                ]
            })
            .collect()
    }

    fn select_entry(&mut self, idx: usize, cx: &mut Context<Self>) {
        if self.selected_entry_idx == Some(idx) {
            return;
        }

        self.selected_entry_idx = Some(idx);
        // Don't clear selected_entry_indices here since it's handled by the caller
        self.selected_commit_diff = None;

        let Some(commit) = self.graph.commits.get(idx) else {
            return;
        };

        let sha = commit.data.sha.to_string();
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let diff_receiver = repository.update(cx, |repo, _| repo.load_commit_diff(sha));

        self._commit_diff_task = Some(cx.spawn(async move |this, cx| {
            if let Ok(Ok(diff)) = diff_receiver.await {
                this.update(cx, |this, cx| {
                    this.selected_commit_diff = Some(diff);
                    cx.notify();
                })
                .ok();
            }
        }));

        cx.notify();
    }

    fn select_previous_commit(&mut self, cx: &mut Context<Self>) {
        if self.graph.commits.is_empty() {
            return;
        }

        let current_idx = self.selected_entry_idx.unwrap_or(0);
        let new_idx = if current_idx == 0 {
            self.graph.commits.len() - 1
        } else {
            current_idx - 1
        };

        self.selected_entry_indices.clear();
        self.selected_entry_indices.push(new_idx);
        self.select_entry(new_idx, cx);
    }

    fn select_next_commit(&mut self, cx: &mut Context<Self>) {
        if self.graph.commits.is_empty() {
            return;
        }

        let current_idx = self.selected_entry_idx.unwrap_or(0);
        let new_idx = if current_idx >= self.graph.commits.len() - 1 {
            0
        } else {
            current_idx + 1
        };

        self.selected_entry_indices.clear();
        self.selected_entry_indices.push(new_idx);
        self.select_entry(new_idx, cx);
    }

    fn select_previous_commit_extend(&mut self, cx: &mut Context<Self>) {
        if self.graph.commits.is_empty() {
            return;
        }

        let current_idx = self.selected_entry_idx.unwrap_or(0);
        let new_idx = if current_idx == 0 {
            self.graph.commits.len() - 1
        } else {
            current_idx - 1
        };

        // Add to selection if not already selected
        if !self.selected_entry_indices.contains(&new_idx) {
            self.selected_entry_indices.push(new_idx);
            self.selected_entry_idx = Some(new_idx);
            self.selected_commit_diff = None;
            self._commit_diff_task = None;
            cx.notify();
        }
    }

    fn select_next_commit_extend(&mut self, cx: &mut Context<Self>) {
        if self.graph.commits.is_empty() {
            return;
        }

        let current_idx = self.selected_entry_idx.unwrap_or(0);
        let new_idx = if current_idx >= self.graph.commits.len() - 1 {
            0
        } else {
            current_idx + 1
        };

        // Add to selection if not already selected
        if !self.selected_entry_indices.contains(&new_idx) {
            self.selected_entry_indices.push(new_idx);
            self.selected_entry_idx = Some(new_idx);
            self.selected_commit_diff = None;
            self._commit_diff_task = None;
            cx.notify();
        }
    }

    fn show_context_menu_for_branch(
        &mut self,
        branch_name: SharedString,
        is_remote: bool,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.context_branch_name = Some(branch_name.clone());
        self.context_is_remote = is_remote;

        let selected_count = self.selected_branches.len();
        let has_multiple = selected_count > 1;
        let is_selected = self.selected_branches.contains(&branch_name);

        let menu = ContextMenu::build(window, cx, |mut menu, _focus_handle, cx| {
            if has_multiple {
                // Multi-branch operations
                menu = menu.entry(
                    format!("Checkout {} Branches", selected_count),
                    None,
                    |_window, _cx| {
                        // TODO: Implement bulk checkout
                    },
                );

                if !is_remote {
                    menu = menu.separator().entry(
                        format!("Delete {} Branches...", selected_count),
                        None,
                        |_window, cx| cx.dispatch_action(&DeleteBranch),
                    );
                }

                if is_remote {
                    menu = menu.separator().entry(
                        format!("Delete {} Remote Branches...", selected_count),
                        None,
                        |_window, cx| cx.dispatch_action(&DeleteBranch),
                    );
                }
            } else {
                // Single branch operations
                menu = menu.entry("Checkout", None, |_window, cx| {
                    cx.dispatch_action(&CheckoutBranch)
                });

                if !is_remote {
                    menu = menu
                        .separator()
                        .entry("Merge into Current", None, |_window, cx| {
                            cx.dispatch_action(&MergeBranch)
                        })
                        .entry("Rebase Current Onto", None, |_window, cx| {
                            cx.dispatch_action(&RebaseOnto)
                        })
                        .separator()
                        .entry("Delete Branch...", None, |_window, cx| {
                            cx.dispatch_action(&DeleteBranch)
                        });
                }

                if is_remote {
                    menu =
                        menu.separator()
                            .entry("Delete Remote Branch...", None, |_window, cx| {
                                cx.dispatch_action(&DeleteBranch)
                            });
                }
            }

            menu
        });

        let subscription = cx.subscribe(&menu, |this, _, _: &DismissEvent, cx| {
            this.context_menu = None;
            this.context_branch_name = None;
            cx.notify();
        });

        self.context_menu = Some((menu, position, subscription));
        cx.notify();
    }

    /// Prettify a branch name for display:
    /// - Strips "HEAD -> " prefix
    /// - Remote branches (refs/remotes/origin/foo) -> "origin/foo"
    /// - Local branches keep just the branch name
    fn prettify_branch_name(name: &str) -> (String, bool) {
        // Strip "HEAD -> " prefix if present
        let name = name.strip_prefix("HEAD -> ").unwrap_or(name);

        // Check for remote branch patterns
        let is_remote = name.starts_with("origin/")
            || name.starts_with("upstream/")
            || name.starts_with("refs/remotes/")
            || (name.contains('/') && !name.starts_with("refs/heads/"));

        let pretty_name = if name.starts_with("refs/remotes/") {
            // refs/remotes/origin/main -> origin/main
            name.strip_prefix("refs/remotes/")
                .unwrap_or(name)
                .to_string()
        } else if name.starts_with("refs/heads/") {
            // refs/heads/main -> main
            name.strip_prefix("refs/heads/").unwrap_or(name).to_string()
        } else {
            name.to_string()
        };

        (pretty_name, is_remote)
    }

    fn show_context_menu_for_commits(
        &mut self,
        position: Point<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let selected_count = self.selected_entry_indices.len().max(1);

        // Collect branch info for all selected commits
        let mut all_branches: Vec<(usize, Vec<SharedString>)> = Vec::new();
        for &idx in &self.selected_entry_indices {
            if let Some(commit) = self.graph.commits.get(idx) {
                let branches: Vec<SharedString> = commit
                    .data
                    .ref_names
                    .iter()
                    .filter(|name| name.as_ref() != "HEAD")
                    .cloned()
                    .collect();
                if !branches.is_empty() {
                    all_branches.push((idx, branches));
                }
            }
        }

        // Check if all selected commits share at least one common branch
        // This is required for squash and drop operations
        let commits_on_same_branch = if selected_count >= 2 {
            // Get branches for all selected commits
            let mut branch_sets: Vec<std::collections::HashSet<&str>> = Vec::new();
            for &idx in &self.selected_entry_indices {
                if let Some(commit) = self.graph.commits.get(idx) {
                    let branches: std::collections::HashSet<&str> = commit
                        .data
                        .ref_names
                        .iter()
                        .filter(|name| name.as_ref() != "HEAD")
                        .map(|s| s.as_ref())
                        .collect();
                    branch_sets.push(branches);
                }
            }

            // Check if there's at least one branch common to all commits
            if branch_sets.is_empty() {
                false
            } else {
                let first_set = &branch_sets[0];
                first_set
                    .iter()
                    .any(|branch| branch_sets.iter().skip(1).all(|set| set.contains(branch)))
            }
        } else {
            true // Single commit is always "on same branch"
        };

        let can_squash = selected_count >= 2 && commits_on_same_branch;
        let can_drop = selected_count >= 1 && (selected_count == 1 || commits_on_same_branch);

        // Get commit data and ref_names for single selection
        let (sha, message, author, ref_names) = if selected_count == 1 {
            if let Some(idx) = self.selected_entry_idx {
                if let Some(commit) = self.graph.commits.get(idx) {
                    let repository = self
                        .project
                        .read_with(cx, |project, cx| project.active_repository(cx));

                    let ref_names = commit.data.ref_names.clone();

                    if let Some(repository) = repository {
                        let data = repository.update(cx, |repository, cx| {
                            repository.fetch_commit_data(commit.data.sha, cx).clone()
                        });

                        let sha = commit.data.sha.to_string();
                        if let CommitDataState::Loaded(data) = data {
                            let message = data.subject.to_string();
                            let author = format!("{} <{}>", data.author_name, data.author_email);
                            (Some(sha), Some(message), Some(author), ref_names)
                        } else {
                            (Some(sha), None, None, ref_names)
                        }
                    } else {
                        (Some(commit.data.sha.to_string()), None, None, ref_names)
                    }
                } else {
                    (None, None, None, Vec::new())
                }
            } else {
                (None, None, None, Vec::new())
            }
        } else {
            (None, None, None, Vec::new())
        };

        // Get the current branch name to avoid showing merge/rebase for current branch
        let current_branch_name: Option<String> = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx))
            .and_then(|repo| repo.read(cx).branch.as_ref().map(|b| b.name().to_string()));

        let menu = ContextMenu::build(window, cx, |mut menu, _focus_handle, cx| {
            // Copy options for single selection
            if let Some(sha) = sha {
                menu = menu.entry("Copy SHA", None, {
                    move |_window, cx| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(sha.clone()));
                    }
                });

                if let Some(message) = message {
                    menu = menu.entry("Copy Message", None, {
                        move |_window, cx| {
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(message.clone()));
                        }
                    });
                }

                if let Some(author) = author {
                    menu = menu.entry("Copy Author", None, {
                        move |_window, cx| {
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(author.clone()));
                        }
                    });
                }
            }
            menu = menu.separator();

            // Checkout revision (single selection only)
            if selected_count == 1 {
                menu = menu.entry("Checkout Revision", None, |_window, cx| {
                    cx.dispatch_action(&CheckoutRevision)
                });
            }

            if can_squash {
                menu = menu.entry(
                    format!("Squash {} Commits...", selected_count),
                    None,
                    |_window, cx| cx.dispatch_action(&SquashCommits),
                );
            }

            // Edit/amend commit (only for single selection)
            if selected_count == 1 {
                menu = menu.entry("Edit/Amend Commit...", None, |_window, cx| {
                    cx.dispatch_action(&EditAmendCommit)
                });
            }

            // Drop commits (dangerous operation) - only if commits are on the same branch
            if can_drop {
                menu = menu.separator().entry(
                    if selected_count == 1 {
                        "Drop Commit...".to_string()
                    } else {
                        format!("Drop {} Commits...", selected_count)
                    },
                    None,
                    |_window, cx| cx.dispatch_action(&DropCommits),
                );
            }

            // Reword commits
            if selected_count >= 1 {
                menu = menu.entry(
                    if selected_count == 1 {
                        "Reword Commit...".to_string()
                    } else {
                        format!("Reword {} Commits...", selected_count)
                    },
                    None,
                    |_window, cx| cx.dispatch_action(&RewordCommits),
                );
            }

            // Cherry-pick commits
            if selected_count >= 1 {
                menu = menu.entry(
                    if selected_count == 1 {
                        "Cherry-pick Commit".to_string()
                    } else {
                        format!("Cherry-pick {} Commits", selected_count)
                    },
                    None,
                    |_window, cx| cx.dispatch_action(&CherryPick),
                );
            }

            // Revert commits
            if selected_count >= 1 {
                menu = menu.entry(
                    if selected_count == 1 {
                        "Revert Commit...".to_string()
                    } else {
                        format!("Revert {} Commits...", selected_count)
                    },
                    None,
                    |_window, cx| cx.dispatch_action(&RevertCommits),
                );
            }

            // Branch operations for commits with branches/tags
            if !ref_names.is_empty() {
                menu = menu.separator();

                for ref_name in ref_names.iter() {
                    // Skip HEAD pointer
                    if ref_name.as_ref() == "HEAD" {
                        continue;
                    }

                    let (pretty_name, is_remote) = Self::prettify_branch_name(ref_name.as_ref());
                    let branch_name: SharedString = ref_name.clone().into();
                    let display_name = pretty_name.clone();

                    // Check if this branch is the current branch
                    let is_current_branch = current_branch_name
                        .as_ref()
                        .map(|current| current == &pretty_name)
                        .unwrap_or(false);

                    menu = menu.custom_entry(
                        move |_window, _cx| {
                            Label::new(format!("Branch: {}", display_name))
                                .color(Color::Muted)
                                .size(LabelSize::Small)
                                .into_any_element()
                        },
                        |_, _| {},
                    );

                    // Don't show checkout for current branch
                    if !is_current_branch {
                        let checkout_display = pretty_name.clone();
                        menu = menu.entry(format!("  Checkout '{}'", checkout_display), None, {
                            let branch = branch_name.clone();
                            move |_window, cx| {
                                cx.dispatch_action(&SetContextBranch {
                                    branch_name: branch.clone(),
                                    is_remote,
                                });
                                cx.dispatch_action(&CheckoutBranch);
                            }
                        });
                    }

                    // Don't show merge/rebase for current branch (can't merge/rebase current into itself)
                    if !is_remote && !is_current_branch {
                        let merge_display = pretty_name.clone();
                        menu = menu.entry(
                            format!("  Merge '{}' into Current", merge_display),
                            None,
                            {
                                let branch = branch_name.clone();
                                move |_window, cx| {
                                    cx.dispatch_action(&SetContextBranch {
                                        branch_name: branch.clone(),
                                        is_remote: false,
                                    });
                                    cx.dispatch_action(&MergeBranch);
                                }
                            },
                        );

                        let rebase_display = pretty_name.clone();
                        menu = menu.entry(
                            format!("  Rebase Current Onto '{}'", rebase_display),
                            None,
                            {
                                let branch = branch_name.clone();
                                move |_window, cx| {
                                    cx.dispatch_action(&SetContextBranch {
                                        branch_name: branch.clone(),
                                        is_remote: false,
                                    });
                                    cx.dispatch_action(&RebaseOnto);
                                }
                            },
                        );
                    }
                }
            }

            // Multi-commit branch operations (when 2+ commits with branches are selected)
            if selected_count >= 2 && all_branches.len() >= 2 {
                // Collect branch pairs with their pretty names and remote status
                // (source_raw, source_pretty, is_source_remote, target_raw, target_pretty, is_target_remote)
                let mut branch_pairs: Vec<(
                    SharedString,
                    String,
                    bool,
                    SharedString,
                    String,
                    bool,
                )> = Vec::new();

                // Get branches from first and second commits with branches
                let first_branches = &all_branches[0].1;
                let second_branches = &all_branches[1].1;

                for first_branch in first_branches {
                    let (first_pretty, is_first_remote) =
                        Self::prettify_branch_name(first_branch.as_ref());

                    for second_branch in second_branches {
                        let (second_pretty, is_second_remote) =
                            Self::prettify_branch_name(second_branch.as_ref());

                        // Skip if both branches are the same
                        if first_pretty == second_pretty {
                            continue;
                        }

                        // Only allow operations between local branches or from remote to local
                        if !is_first_remote || !is_second_remote {
                            branch_pairs.push((
                                first_branch.clone(),
                                first_pretty.clone(),
                                is_first_remote,
                                second_branch.clone(),
                                second_pretty.clone(),
                                is_second_remote,
                            ));
                        }
                    }
                }

                if !branch_pairs.is_empty() {
                    menu = menu.separator();
                    menu = menu.custom_entry(
                        move |_window, _cx| {
                            Label::new("Branch Operations")
                                .color(Color::Muted)
                                .size(LabelSize::Small)
                                .into_any_element()
                        },
                        |_, _| {},
                    );

                    for (
                        source_raw,
                        source_pretty,
                        is_source_remote,
                        target_raw,
                        target_pretty,
                        is_target_remote,
                    ) in branch_pairs
                    {
                        // Merge source into target (target must be local)
                        if !is_target_remote {
                            menu = menu.entry(
                                format!("Merge '{}' into '{}'", source_pretty, target_pretty),
                                None,
                                {
                                    let src = source_raw.clone();
                                    let tgt = target_raw.clone();
                                    move |_window, cx| {
                                        // First checkout target, then merge source
                                        cx.dispatch_action(&SetContextBranch {
                                            branch_name: tgt.clone(),
                                            is_remote: false,
                                        });
                                        cx.dispatch_action(&CheckoutBranch);
                                        cx.dispatch_action(&SetContextBranch {
                                            branch_name: src.clone(),
                                            is_remote: is_source_remote,
                                        });
                                        cx.dispatch_action(&MergeBranch);
                                    }
                                },
                            );
                        }

                        // Rebase source onto target (source must be local)
                        if !is_source_remote {
                            menu = menu.entry(
                                format!("Rebase '{}' onto '{}'", source_pretty, target_pretty),
                                None,
                                {
                                    let src = source_raw.clone();
                                    let tgt = target_raw.clone();
                                    move |_window, cx| {
                                        // First checkout source, then rebase onto target
                                        cx.dispatch_action(&SetContextBranch {
                                            branch_name: src.clone(),
                                            is_remote: false,
                                        });
                                        cx.dispatch_action(&CheckoutBranch);
                                        cx.dispatch_action(&SetContextBranch {
                                            branch_name: tgt.clone(),
                                            is_remote: is_target_remote,
                                        });
                                        cx.dispatch_action(&RebaseOnto);
                                    }
                                },
                            );
                        }
                    }
                }
            }

            menu
        });

        let subscription = cx.subscribe(&menu, |this, _, _: &DismissEvent, cx| {
            this.context_menu = None;
            cx.notify();
        });

        self.context_menu = Some((menu, position, subscription));
        cx.notify();
    }

    fn checkout_branch(
        &mut self,
        branch_name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.perform_checkout(branch_name, false, window, cx);
    }

    fn checkout_revision(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Get the SHA of the selected commit
        let Some(idx) = self.selected_entry_idx else {
            return;
        };
        let Some(commit) = self.graph.commits.get(idx) else {
            return;
        };

        let sha: SharedString = commit.data.sha.to_string().into();
        self.perform_checkout(sha, false, window, cx);
    }

    fn show_checkout_modal(
        &mut self,
        branch_name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::CheckoutBranch {
                    branch_name: branch_name.clone(),
                    has_uncommitted_changes: true,
                    stash: true,
                },
                |action, window, cx| {
                    if let ModalAction::CheckoutBranch {
                        branch_name, stash, ..
                    } = action
                    {
                        cx.emit(DismissEvent);
                        // Perform checkout will be called from the parent
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_checkout(
        &mut self,
        branch_name: SharedString,
        stash_first: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            // Stash if needed
            if stash_first {
                let receiver = repository
                    .update(cx, |repo, _cx| {
                        repo.stash_all(
                            true,
                            Some(format!("Auto-stash before checkout to {}", branch_name)),
                        )
                    })
                    .unwrap();

                match receiver.await {
                    Ok(()) => {}
                    Err(e) => {
                        // Detached, can't return error
                    }
                }
            }

            // Checkout branch
            let receiver = repository
                .update(cx, |repo, _| repo.change_branch(branch_name.to_string()))
                .unwrap();

            match receiver.await {
                Ok(()) => {}
                Err(e) => {
                    // Detached, can't return error
                }
            }

            // Unstash if we stashed
            if stash_first {
                let task = repository
                    .update(cx, |repo, cx| repo.stash_pop(None, cx))
                    .unwrap();

                // Ignore errors on unstash
                let _ = task.await;
            }

            weak_self
                .update(cx, |this, cx| {
                    this.graph.clear();
                    cx.notify();
                })
                .ok();

            anyhow::Ok(())
        })
        .detach();
    }

    fn pull_with_stash(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            // Stash changes
            let receiver = repository
                .update(cx, |repo, _cx| {
                    repo.stash_all(true, Some("Auto-stash before pull".into()))
                })
                .unwrap();

            match receiver.await {
                Ok(()) => {}
                Err(e) => {
                    // Detached, can't return error
                }
            }

            // Pull with rebase (or merge based on settings)
            // TODO: Get rebase preference from settings
            // TODO: pull() is missing AskPassDelegate parameter - need proper integration
            // let rebase = true;
            // Commented out until AskPassDelegate is properly integrated
            // let receiver = repository
            //     .update(cx, |repo, _cx| {
            //         repo.pull(None, "origin".into(), rebase, cx)
            //     })
            //     .unwrap();
            //
            // match receiver.await {
            //     Ok(()) => {}
            //     Err(e) => return Err(e),
            // }

            // Unstash
            let task = repository
                .update(cx, |repo, cx| repo.stash_pop(None, cx))
                .unwrap();

            // Ignore errors on unstash
            let _ = task.await;

            weak_self
                .update(cx, |this, cx| {
                    this.graph.clear();
                    cx.notify();
                })
                .ok();

            anyhow::Ok(())
        })
        .detach();
    }

    fn merge_branch(
        &mut self,
        branch_name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::MergeBranch {
                    branch_name: branch_name.clone(),
                },
                move |action, window, cx| {
                    if let ModalAction::MergeBranch { branch_name } = action {
                        // Will be handled by parent
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_merge(
        &mut self,
        branch_name: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = repository
                .update(cx, |repo, _| repo.merge_branch(branch_name.to_string()))
                .unwrap();

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // Detached, can't return error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn rebase_onto(
        &mut self,
        target_branch: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::RebaseOnto {
                    target_branch: target_branch.clone(),
                },
                move |action, window, cx| {
                    if let ModalAction::RebaseOnto { target_branch } = action {
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_rebase(
        &mut self,
        target_branch: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = repository
                .update(cx, |repo, _| repo.rebase_onto(target_branch.to_string()))
                .unwrap();

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // Detached, can't return error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn squash_commits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_entry_indices.len() < 2 {
            return;
        }

        let commit_count = self.selected_entry_indices.len();
        let git_graph = cx.weak_entity();
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::SquashCommits {
                    commit_count,
                    message: "Squashed commits".into(),
                },
                move |action, window, cx| {
                    if let ModalAction::SquashCommits { message, .. } = action {
                        if let Some(git_graph) = git_graph.upgrade() {
                            git_graph.update(cx, |git_graph, cx| {
                                git_graph.perform_squash(message.clone(), window, cx);
                            });
                        }
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_squash(
        &mut self,
        message: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let commit_shas: Vec<String> = self
            .selected_entry_indices
            .iter()
            .filter_map(|&idx| self.graph.commits.get(idx).map(|c| c.data.sha.to_string()))
            .collect();

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = repository
                .update(cx, |repo, _| {
                    repo.squash_commits(commit_shas, message.to_string())
                })
                .unwrap();

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            this.selected_entry_indices.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // Detached, can't return error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn drop_commits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_entry_indices.is_empty() {
            return;
        }

        let commit_count = self.selected_entry_indices.len();
        let git_graph = cx.weak_entity();
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::DropCommits { commit_count },
                move |action, window, cx| {
                    if let ModalAction::DropCommits { .. } = action {
                        if let Some(git_graph) = git_graph.upgrade() {
                            git_graph.update(cx, |git_graph, cx| {
                                git_graph.perform_drop_commits(window, cx);
                            });
                        }
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_drop_commits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let commit_shas: Vec<String> = self
            .selected_entry_indices
            .iter()
            .filter_map(|&idx| self.graph.commits.get(idx).map(|c| c.data.sha.to_string()))
            .collect();

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = repository
                .update(cx, |repo, _| repo.drop_commits(commit_shas))
                .unwrap();

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            this.selected_entry_indices.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // Detached, can't return error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn reword_commits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_entry_indices.is_empty() {
            return;
        }

        let commit_count = self.selected_entry_indices.len();
        let current_message: SharedString = "".into();

        let git_graph = cx.weak_entity();
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::RewordCommits {
                    commit_count,
                    message: current_message.clone(),
                },
                move |action, window, cx| {
                    if let ModalAction::RewordCommits { message, .. } = action {
                        if let Some(git_graph) = git_graph.upgrade() {
                            git_graph.update(cx, |git_graph, cx| {
                                git_graph.perform_reword_commits(message.clone(), window, cx);
                            });
                        }
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_reword_commits(
        &mut self,
        new_message: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let commit_shas: Vec<String> = self
            .selected_entry_indices
            .iter()
            .filter_map(|&idx| self.graph.commits.get(idx).map(|c| c.data.sha.to_string()))
            .collect();

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = repository
                .update(cx, |repo, _| {
                    repo.reword_commits(commit_shas, new_message.to_string())
                })
                .unwrap();

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            this.selected_entry_indices.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // Detached, can't return error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn edit_amend_commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_entry_indices.len() != 1 {
            return;
        }

        let idx = self.selected_entry_indices[0];
        let Some(commit) = self.graph.commits.get(idx) else {
            return;
        };

        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let commit_data = repository.update(cx, |repository, cx| {
            repository.fetch_commit_data(commit.data.sha, cx).clone()
        });

        let current_message = match commit_data {
            CommitDataState::Loaded(data) => data.subject.clone(),
            _ => "Loading...".into(),
        };

        let git_graph = cx.weak_entity();
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::EditAmendCommit {
                    current_message: current_message.clone(),
                    amend: true,
                },
                move |action, window, cx| {
                    if let ModalAction::EditAmendCommit {
                        current_message,
                        amend,
                    } = action
                    {
                        if let Some(git_graph) = git_graph.upgrade() {
                            git_graph.update(cx, |git_graph, cx| {
                                git_graph.perform_edit_amend_commit(
                                    current_message.clone(),
                                    amend,
                                    window,
                                    cx,
                                );
                            });
                        }
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_edit_amend_commit(
        &mut self,
        message: SharedString,
        amend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let commit_shas: Vec<String> = self
            .selected_entry_indices
            .iter()
            .filter_map(|&idx| self.graph.commits.get(idx).map(|c| c.data.sha.to_string()))
            .collect();

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            if amend {
                // Edit commits (allows changing files and message)
                let receiver = repository
                    .update(cx, |repo, _| repo.edit_commits(commit_shas))
                    .unwrap();

                match receiver.await {
                    Ok(()) => {
                        weak_self
                            .update(cx, |this, cx| {
                                this.graph.clear();
                                this.selected_entry_indices.clear();
                                cx.notify();
                            })
                            .ok();
                    }
                    Err(e) => {
                        // Detached, can't return error
                    }
                }
            } else {
                // Reword commits (change message only)
                let receiver = repository
                    .update(cx, |repo, _| {
                        repo.reword_commits(commit_shas, message.to_string())
                    })
                    .unwrap();

                match receiver.await {
                    Ok(()) => {
                        weak_self
                            .update(cx, |this, cx| {
                                this.graph.clear();
                                this.selected_entry_indices.clear();
                                cx.notify();
                            })
                            .ok();
                    }
                    Err(e) => {
                        // Detached, can't return error
                    }
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn cherry_pick(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_entry_indices.is_empty() {
            return;
        }

        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let commit_shas: Vec<String> = self
            .selected_entry_indices
            .iter()
            .filter_map(|&idx| self.graph.commits.get(idx).map(|c| c.data.sha.to_string()))
            .collect();

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = repository
                .update(cx, |repo, _| repo.cherry_pick(commit_shas))
                .unwrap();

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            this.selected_entry_indices.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // TODO: Handle cherry-pick conflicts
                    // For now, just ignore the error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn revert_commits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected_entry_indices.is_empty() {
            return;
        }

        let commit_count = self.selected_entry_indices.len();
        let git_graph = cx.weak_entity();
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::RevertCommits { commit_count },
                move |action, window, cx| {
                    if let ModalAction::RevertCommits { .. } = action {
                        if let Some(git_graph) = git_graph.upgrade() {
                            git_graph.update(cx, |git_graph, cx| {
                                git_graph.perform_revert_commits(window, cx);
                            });
                        }
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_revert_commits(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let commit_shas: Vec<String> = self
            .selected_entry_indices
            .iter()
            .filter_map(|&idx| self.graph.commits.get(idx).map(|c| c.data.sha.to_string()))
            .collect();

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = repository
                .update(cx, |repo, _| repo.revert_commits(commit_shas))
                .unwrap();

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            this.selected_entry_indices.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if error_msg.contains("conflicts detected") {
                        // TODO: Handle revert conflicts
                    } else {
                        return Err(e);
                    }
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn delete_branch(
        &mut self,
        branch_name: SharedString,
        is_remote: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let git_graph = cx.weak_entity();
        let modal = cx.new(|cx| {
            GitModal::new(
                ModalAction::DeleteBranch {
                    branch_name: branch_name.clone(),
                    is_remote,
                    delete_remote: false,
                },
                move |action, window, cx| {
                    if let ModalAction::DeleteBranch {
                        branch_name,
                        is_remote,
                        delete_remote,
                    } = action
                    {
                        if let Some(git_graph) = git_graph.upgrade() {
                            git_graph.update(cx, |git_graph, cx| {
                                git_graph.perform_delete_branch(
                                    branch_name,
                                    delete_remote,
                                    window,
                                    cx,
                                );
                            });
                        }
                        cx.emit(DismissEvent);
                    }
                },
                window,
                cx,
            )
        });

        self.modal = Some(modal);
        cx.notify();
    }

    fn perform_delete_branch(
        &mut self,
        branch_name: SharedString,
        delete_remote: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let selected_branches = self.selected_branches.clone();
        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let branches_to_delete = if !selected_branches.is_empty() {
                selected_branches
            } else {
                vec![branch_name.clone()]
            };

            for branch in branches_to_delete {
                // Delete local branch
                let receiver = repository
                    .update(cx, |repo, _| repo.delete_branch(branch.to_string()))
                    .unwrap();

                match receiver.await {
                    Ok(()) => {}
                    Err(e) => {
                        // Detached, can't return error
                    }
                }

                // Delete remote branch if requested
                if delete_remote {
                    // TODO: delete_remote_branch is missing parameters - need proper integration
                    // Commented out until properly integrated
                    // let receiver = repository
                    //     .update(cx, |repo, _cx| {
                    //         repo.delete_remote_branch("origin".into(), branch.to_string())
                    //     })
                    //     .unwrap();
                    //
                    // match receiver.await {
                    //     Ok(()) => {}
                    //     Err(e) => return Err(e),
                    // }
                }
            }

            weak_self
                .update(cx, |this, cx: &mut Context<Self>| {
                    this.graph.clear();
                    this.selected_branches.clear();
                    cx.notify();
                })
                .ok();

            anyhow::Ok(())
        })
        .detach();
    }

    fn get_remote(
        &self,
        repository: &Repository,
        _window: &mut Window,
        cx: &mut App,
    ) -> Option<GitRemote> {
        let remote_url = repository.default_remote_url()?;
        let provider_registry = GitHostingProviderRegistry::default_global(cx);
        let (provider, parsed) = parse_git_remote_url(provider_registry, &remote_url)?;
        Some(GitRemote {
            host: provider,
            owner: parsed.owner.into(),
            repo: parsed.repo.into(),
        })
    }

    fn render_commit_detail_panel(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let Some(selected_idx) = self.selected_entry_idx else {
            return div().into_any_element();
        };

        let Some(commit_entry) = self.graph.commits.get(selected_idx) else {
            return div().into_any_element();
        };

        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return div().into_any_element();
        };

        let commit_data = repository.update(cx, |repository, cx| {
            repository
                .fetch_commit_data(commit_entry.data.sha, cx)
                .clone()
        });

        let full_sha: SharedString = commit_entry.data.sha.to_string().into();
        let truncated_sha: SharedString = {
            let sha_str = full_sha.as_ref();
            if sha_str.len() > 24 {
                format!("{}...", &sha_str[..24]).into()
            } else {
                full_sha.clone()
            }
        };
        let ref_names = commit_entry.data.ref_names.clone();
        let accent_colors = cx.theme().accents();
        let accent_color = accent_colors
            .0
            .get(commit_entry.color_idx)
            .copied()
            .unwrap_or_else(|| accent_colors.0.first().copied().unwrap_or_default());

        let (author_name, author_email, commit_timestamp, subject) =
            if let CommitDataState::Loaded(data) = commit_data {
                (
                    data.author_name.clone(),
                    data.author_email.clone(),
                    Some(data.commit_timestamp),
                    data.subject.clone(),
                )
            } else {
                ("Loading...".into(), "".into(), None, "Loading...".into())
            };

        let date_string = commit_timestamp
            .and_then(|ts| OffsetDateTime::from_unix_timestamp(ts).ok())
            .map(|datetime| {
                let local_offset = UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC);
                let local_datetime = datetime.to_offset(local_offset);
                let format =
                    time::format_description::parse("[month repr:short] [day], [year]").ok();
                format
                    .and_then(|f| local_datetime.format(&f).ok())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let remote = repository.update(cx, |repo, cx| self.get_remote(repo, window, cx));

        let avatar = {
            let avatar = CommitAvatar::new(&full_sha, remote.as_ref());
            v_flex()
                .w(px(64.))
                .h(px(64.))
                .border_1()
                .border_color(cx.theme().colors().border)
                .rounded_full()
                .justify_center()
                .items_center()
                .child(
                    avatar
                        .avatar(window, cx)
                        .map(|a| a.size(px(64.)).into_any_element())
                        .unwrap_or_else(|| {
                            Icon::new(IconName::Person)
                                .color(Color::Muted)
                                .size(IconSize::XLarge)
                                .into_any_element()
                        }),
                )
        };

        let changed_files_count = self
            .selected_commit_diff
            .as_ref()
            .map(|diff| diff.files.len())
            .unwrap_or(0);

        let content = v_flex()
            .w(px(300.))
            .h_full()
            .border_l_1()
            .border_color(cx.theme().colors().border)
            .bg(cx.theme().colors().surface_background)
            .child(
                v_flex()
                    .p_3()
                    .gap_3()
                    .child(
                        h_flex().justify_between().child(avatar).child(
                            IconButton::new("close-detail", IconName::Close)
                                .icon_size(IconSize::Small)
                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                    this.selected_entry_idx = None;
                                    this.selected_commit_diff = None;
                                    this._commit_diff_task = None;
                                    cx.notify();
                                })),
                        ),
                    )
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(Label::new(author_name.clone()).weight(FontWeight::SEMIBOLD))
                            .child(
                                Label::new(date_string)
                                    .color(Color::Muted)
                                    .size(LabelSize::Small),
                            ),
                    )
                    .children((!ref_names.is_empty()).then(|| {
                        h_flex().gap_1().flex_wrap().children(
                            ref_names
                                .iter()
                                .map(|name| self.render_badge(name, accent_color, cx))
                                .collect::<Vec<_>>(),
                        )
                    }))
                    .child(
                        v_flex()
                            .gap_1p5()
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Icon::new(IconName::Person)
                                            .size(IconSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .child(
                                        Label::new(author_name)
                                            .size(LabelSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .when(!author_email.is_empty(), |this| {
                                        this.child(
                                            Label::new(format!("<{}>", author_email))
                                                .size(LabelSize::Small)
                                                .color(Color::Ignored),
                                        )
                                    }),
                            )
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Icon::new(IconName::Hash)
                                            .size(IconSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .child({
                                        let copy_sha = full_sha.clone();
                                        Button::new("sha-button", truncated_sha)
                                            .style(ButtonStyle::Transparent)
                                            .label_size(LabelSize::Small)
                                            .color(Color::Muted)
                                            .tooltip(Tooltip::text(format!(
                                                "Copy SHA: {}",
                                                copy_sha
                                            )))
                                            .on_click(cx.listener(
                                                move |this, _event, _window, cx| {
                                                    cx.write_to_clipboard(
                                                        ClipboardItem::new_string(
                                                            copy_sha.to_string(),
                                                        ),
                                                    );
                                                },
                                            ))
                                    }),
                            )
                            .when_some(remote.clone(), |this, remote| {
                                let provider_name = remote.host.name();
                                let icon = match provider_name.as_str() {
                                    "GitHub" => IconName::Github,
                                    _ => IconName::Link,
                                };
                                let parsed_remote = ParsedGitRemote {
                                    owner: remote.owner.as_ref().into(),
                                    repo: remote.repo.as_ref().into(),
                                };
                                let params = BuildCommitPermalinkParams {
                                    sha: full_sha.as_ref(),
                                };
                                let url = remote
                                    .host
                                    .build_commit_permalink(&parsed_remote, params)
                                    .to_string();
                                this.child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            Icon::new(icon)
                                                .size(IconSize::Small)
                                                .color(Color::Muted),
                                        )
                                        .child(
                                            Button::new(
                                                "view-on-provider",
                                                format!("View on {}", provider_name),
                                            )
                                            .style(ButtonStyle::Transparent)
                                            .label_size(LabelSize::Small)
                                            .color(Color::Muted)
                                            .on_click(
                                                cx.listener(move |this, _event, _window, cx| {
                                                    cx.open_url(&url);
                                                }),
                                            ),
                                        ),
                                )
                            })
                            .child(
                                h_flex()
                                    .gap_1()
                                    .child(
                                        Icon::new(IconName::Undo)
                                            .size(IconSize::Small)
                                            .color(Color::Muted),
                                    )
                                    .child(
                                        Button::new("uncommit", "Uncommit")
                                            .style(ButtonStyle::Transparent)
                                            .label_size(LabelSize::Small)
                                            .color(Color::Muted)
                                            .on_click(cx.listener(
                                                move |this, _event, _window, cx| {
                                                    // TODO: Implement uncommit
                                                },
                                            )),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .border_t_1()
                    .border_color(cx.theme().colors().border)
                    .p_3()
                    .child(
                        v_flex()
                            .gap_2()
                            .child(Label::new(subject).weight(FontWeight::MEDIUM)),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .border_t_1()
                    .border_color(cx.theme().colors().border)
                    .p_3()
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                Label::new(format!("{} Changed Files", changed_files_count))
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            )
                            .child({
                                if let Some(diff) = self.selected_commit_diff.as_ref() {
                                    let tree = self.build_file_tree(&diff.files);
                                    self.render_file_tree(&tree, cx)
                                } else {
                                    div().into_any_element()
                                }
                            }),
                    ),
            )
            .into_any_element();
        return content;
    }

    fn build_file_tree(&self, files: &[CommitFile]) -> TreeNode {
        let mut root = TreeNode {
            children: HashMap::new(),
            file: None,
        };

        for file in files {
            let path = file.path.as_std_path();
            let mut current = &mut root;
            for component in path.components() {
                if let std::path::Component::Normal(name) = component {
                    let name_str = name.to_string_lossy().to_string();
                    current = current.children.entry(name_str).or_insert(TreeNode {
                        children: HashMap::new(),
                        file: None,
                    });
                }
            }
            current.file = Some(CommitFile {
                path: file.path.clone(),
                old_text: file.old_text.clone(),
                new_text: file.new_text.clone(),
                is_binary: file.is_binary,
            });
        }

        root
    }

    fn render_file_tree(&self, node: &TreeNode, cx: &Context<Self>) -> AnyElement {
        v_flex()
            .children(
                node.children
                    .iter()
                    .map(|(name, child)| {
                        if child.file.is_some() {
                            // It's a file
                            let file = child.file.as_ref().unwrap();
                            let file_name = file
                                .path
                                .file_name()
                                .map(|n| n.to_string())
                                .unwrap_or_default();

                            let content_element = if file.is_binary {
                                Label::new("Binary file")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted)
                                    .into_any_element()
                            } else if let Some(content) = &file.new_text {
                                div()
                                    .bg(cx.theme().colors().editor_background)
                                    .border_1()
                                    .border_color(cx.theme().colors().border)
                                    .p_2()
                                    .rounded_sm()
                                    .max_h(px(200.))
                                    .child(
                                        Label::new(content.clone())
                                            .size(LabelSize::Small)
                                            .single_line(),
                                    )
                                    .into_any_element()
                            } else {
                                Label::new("No content")
                                    .size(LabelSize::Small)
                                    .color(Color::Muted)
                                    .into_any_element()
                            };
                            v_flex()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            Icon::new(IconName::File)
                                                .size(IconSize::Small)
                                                .color(Color::Accent),
                                        )
                                        .child(
                                            Label::new(file_name)
                                                .size(LabelSize::Small)
                                                .weight(FontWeight::BOLD),
                                        ),
                                )
                                .child(content_element)
                                .into_any_element()
                        } else {
                            // It's a directory
                            div()
                                .child(
                                    h_flex()
                                        .gap_1()
                                        .child(
                                            Icon::new(IconName::Folder)
                                                .size(IconSize::Small)
                                                .color(Color::Muted),
                                        )
                                        .child(
                                            Label::new(name.clone())
                                                .size(LabelSize::Small)
                                                .weight(FontWeight::BOLD),
                                        ),
                                )
                                .child(div().pl_4().child(self.render_file_tree(child, cx)))
                                .into_any_element()
                        }
                    })
                    .collect::<Vec<_>>(),
            )
            .into_any_element()
    }

    fn handle_graph_scroll(
        &mut self,
        event: &ScrollWheelEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let line_height = window.line_height();
        let delta = event.delta.pixel_delta(line_height);

        let table_state = self.table_interaction_state.read(cx);
        let current_offset = table_state.scroll_offset();

        let viewport_height = table_state.scroll_handle.viewport().size.height;

        let commit_count = match self.graph.max_commit_count {
            AllCommitCount::Loaded(count) => count,
            AllCommitCount::NotLoaded => self.graph.commits.len(),
        };
        let content_height = self.row_height * commit_count;
        let max_vertical_scroll = (viewport_height - content_height).min(px(0.));

        let new_y = (current_offset.y + delta.y).clamp(max_vertical_scroll, px(0.));
        let new_offset = Point::new(current_offset.x, new_y);

        let left_padding = px(12.0);
        let lane_width = px(16.0);
        let max_lanes = self.graph.max_lanes.max(1);
        let graph_content_width = lane_width * max_lanes as f32 + left_padding * 2.0;
        let max_horizontal_scroll = (graph_content_width - self.graph_viewport_width).max(px(0.));

        let new_horizontal_offset =
            (self.horizontal_scroll_offset - delta.x).clamp(px(0.), max_horizontal_scroll);

        let vertical_changed = new_offset != current_offset;
        let horizontal_changed = new_horizontal_offset != self.horizontal_scroll_offset;

        if vertical_changed {
            table_state.set_scroll_offset(new_offset);
        }

        if horizontal_changed {
            self.horizontal_scroll_offset = new_horizontal_offset;
        }

        if vertical_changed || horizontal_changed {
            cx.notify();
        }
    }
}

impl Render for GitGraph {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let description_width_fraction = 0.72;
        let date_width_fraction = 0.12;
        let author_width_fraction = 0.10;
        let commit_width_fraction = 0.06;

        let error_banner = self.error.as_ref().map(|error| {
            h_flex()
                .id("error-banner")
                .w_full()
                .px_2()
                .py_1()
                .bg(cx.theme().colors().surface_background)
                .border_b_1()
                .border_color(cx.theme().colors().border)
                .justify_between()
                .items_center()
                .child(
                    h_flex()
                        .gap_2()
                        .overflow_hidden()
                        .child(Icon::new(IconName::Warning).color(Color::Error))
                        .child(Label::new(error.clone()).color(Color::Error).single_line()),
                )
                .child(
                    IconButton::new("dismiss-error", IconName::Close)
                        .icon_size(IconSize::Small)
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.error = None;
                            cx.notify();
                        })),
                )
        });

        let commit_count = match self.graph.max_commit_count {
            AllCommitCount::Loaded(count) => count,
            AllCommitCount::NotLoaded => {
                self.project.update(cx, |project, cx| {
                    if let Some(repository) = project.active_repository(cx) {
                        repository.update(cx, |repository, cx| {
                            // Start loading the graph data if we haven't started already
                            repository.graph_data(
                                self.log_source.clone(),
                                self.log_order,
                                0..0,
                                cx,
                            );
                        })
                    }
                });

                self.graph.commits.len()
            }
        };

        let content = if self.loading && self.graph.commits.is_empty() && false {
            let message = if self.loading {
                "Loading commits..."
            } else {
                "No commits found"
            };
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .child(Label::new(message).color(Color::Muted))
        } else {
            let graph_viewport_width = self.graph_viewport_width;
            div()
                .size_full()
                .flex()
                .flex_row()
                .child(
                    div()
                        .w(graph_viewport_width)
                        .h_full()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .p_2()
                                .border_b_1()
                                .border_color(cx.theme().colors().border)
                                .child(Label::new("Graph").color(Color::Muted)),
                        )
                        .child(
                            div()
                                .id("graph-canvas")
                                .flex_1()
                                .overflow_hidden()
                                .child(render_graph(&self, cx))
                                .on_scroll_wheel(cx.listener(Self::handle_graph_scroll))
                                .on_mouse_down(gpui::MouseButton::Right, {
                                    let weak_self = cx.weak_entity();
                                    move |event: &gpui::MouseDownEvent, window, cx| {
                                        weak_self
                                            .update(cx, |this, cx| {
                                                this.show_context_menu_for_commits(
                                                    event.position,
                                                    window,
                                                    cx,
                                                );
                                            })
                                            .ok();
                                    }
                                }),
                        ),
                )
                .child({
                    let row_height = self.row_height;
                    let selected_entry_indices = self.selected_entry_indices.clone();
                    let weak_self = cx.weak_entity();
                    div().flex_1().size_full().child(
                        Table::new(4)
                            .interactable(&self.table_interaction_state)
                            .hide_row_borders()
                            .header(vec![
                                Label::new("Description")
                                    .color(Color::Muted)
                                    .into_any_element(),
                                Label::new("Date").color(Color::Muted).into_any_element(),
                                Label::new("Author").color(Color::Muted).into_any_element(),
                                Label::new("Commit").color(Color::Muted).into_any_element(),
                            ])
                            .column_widths(
                                [
                                    DefiniteLength::Fraction(description_width_fraction),
                                    DefiniteLength::Fraction(date_width_fraction),
                                    DefiniteLength::Fraction(author_width_fraction),
                                    DefiniteLength::Fraction(commit_width_fraction),
                                ]
                                .to_vec(),
                            )
                            .map_row({
                                let weak_self = weak_self.clone();
                                let selected_entry_indices = selected_entry_indices.clone();
                                move |(index, row), _window, row_cx| {
                                    let is_selected = selected_entry_indices.contains(&index);
                                    let weak = weak_self.clone();
                                    row.h(row_height)
                                        .when(is_selected, |row| {
                                            row.bg(row_cx.theme().colors().element_selected)
                                        })
                                        .on_mouse_down(gpui::MouseButton::Left, {
                                            move |event: &gpui::MouseDownEvent, window, cx| {
                                                weak.update(cx, |this, cx| {
                                                    if event.modifiers.secondary() {
                                                        // Multi-select mode: toggle selection
                                                        if let Some(pos) = this
                                                            .selected_entry_indices
                                                            .iter()
                                                            .position(|&i| i == index)
                                                        {
                                                            this.selected_entry_indices.remove(pos);
                                                            if this.selected_entry_idx
                                                                == Some(index)
                                                            {
                                                                this.selected_entry_idx = this
                                                                    .selected_entry_indices
                                                                    .last()
                                                                    .copied();
                                                            }
                                                        } else {
                                                            this.selected_entry_indices.push(index);
                                                            this.selected_entry_idx = Some(index);
                                                        }
                                                        this.selected_commit_diff = None;
                                                        this._commit_diff_task = None;
                                                    } else {
                                                        // Single-select mode: clear and select only this one
                                                        this.selected_entry_indices.clear();
                                                        this.selected_entry_indices.push(index);
                                                        this.select_entry(index, cx);
                                                    }
                                                    cx.notify();
                                                })
                                                .ok();
                                            }
                                        })
                                        .on_mouse_down(gpui::MouseButton::Right, {
                                            let weak = weak_self.clone();
                                            move |event: &gpui::MouseDownEvent, window, cx| {
                                                weak.update(cx, |this, cx| {
                                                    // If the clicked row is not already selected, select it exclusively
                                                    if !this.selected_entry_indices.contains(&index)
                                                    {
                                                        this.selected_entry_indices.clear();
                                                        this.selected_entry_indices.push(index);
                                                        this.select_entry(index, cx);
                                                    }

                                                    this.show_context_menu_for_commits(
                                                        event.position,
                                                        window,
                                                        cx,
                                                    );
                                                    cx.notify();
                                                })
                                                .ok();
                                            }
                                        })
                                        .into_any_element()
                                }
                            })
                            .uniform_list(
                                "git-graph-commits",
                                commit_count,
                                cx.processor(Self::render_table_rows),
                            ),
                    )
                })
                .when(self.selected_entry_idx.is_some(), |this| {
                    this.child(self.render_commit_detail_panel(window, cx))
                })
        };

        div()
            .size_full()
            .bg(cx.theme().colors().editor_background)
            .key_context("GitGraph")
            .track_focus(&self.focus_handle)
            .child(v_flex().size_full().children(error_banner).child(content))
            .children(self.context_menu.as_ref().map(|(menu, position, _)| {
                deferred(
                    anchored()
                        .position(*position)
                        .anchor(Corner::TopLeft)
                        .child(menu.clone()),
                )
                .with_priority(1)
            }))
            .children(self.modal.as_ref().map(|modal| {
                deferred(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(gpui::black().opacity(0.5))
                        .child(modal.clone()),
                )
                .with_priority(2)
            }))
    }
}

impl EventEmitter<ItemEvent> for GitGraph {}

impl Focusable for GitGraph {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl GitGraph {
    fn handle_checkout_branch_action(
        &mut self,
        _: &CheckoutBranch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(branch_name) = self.context_branch_name.clone() {
            self.checkout_branch(branch_name, window, cx);
        }
    }

    fn handle_pull_with_stash_action(
        &mut self,
        _: &PullWithStash,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.pull_with_stash(window, cx);
    }

    fn handle_merge_branch_action(
        &mut self,
        _: &MergeBranch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(branch_name) = self.context_branch_name.clone() {
            self.merge_branch(branch_name, window, cx);
        }
    }

    fn handle_rebase_onto_action(
        &mut self,
        _: &RebaseOnto,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(branch_name) = self.context_branch_name.clone() {
            self.rebase_onto(branch_name, window, cx);
        }
    }

    fn handle_squash_commits_action(
        &mut self,
        _: &SquashCommits,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.squash_commits(window, cx);
    }

    fn handle_drop_commits_action(
        &mut self,
        _: &DropCommits,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.drop_commits(window, cx);
    }

    fn handle_reword_commits_action(
        &mut self,
        _: &RewordCommits,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.reword_commits(window, cx);
    }

    fn handle_edit_amend_commit_action(
        &mut self,
        _: &EditAmendCommit,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.edit_amend_commit(window, cx);
    }

    fn handle_delete_branch_action(
        &mut self,
        _: &DeleteBranch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(branch_name) = self.context_branch_name.clone() {
            let is_remote = self.context_is_remote;
            self.delete_branch(branch_name, is_remote, window, cx);
        }
    }

    fn handle_cherry_pick_action(
        &mut self,
        _: &CherryPick,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.cherry_pick(window, cx);
    }

    fn handle_cherry_pick_conflict(
        &mut self,
        abort: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = if abort {
                repository
                    .update(cx, |repo, _| repo.abort_cherry_pick())
                    .unwrap()
            } else {
                repository
                    .update(cx, |repo, _| repo.continue_cherry_pick())
                    .unwrap()
            };

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            this.selected_entry_indices.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // Detached, can't return error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn handle_revert_conflict(&mut self, abort: bool, window: &mut Window, cx: &mut Context<Self>) {
        let repository = self
            .project
            .read_with(cx, |project, cx| project.active_repository(cx));

        let Some(repository) = repository else {
            return;
        };

        let weak_self = cx.weak_entity();
        cx.spawn(async move |_, cx| {
            let receiver = if abort {
                repository
                    .update(cx, |repo, _| repo.abort_revert())
                    .unwrap()
            } else {
                repository
                    .update(cx, |repo, _| repo.continue_revert())
                    .unwrap()
            };

            match receiver.await {
                Ok(()) => {
                    weak_self
                        .update(cx, |this, cx| {
                            this.graph.clear();
                            this.selected_entry_indices.clear();
                            cx.notify();
                        })
                        .ok();
                }
                Err(e) => {
                    // Detached, can't return error
                }
            }

            anyhow::Ok(())
        })
        .detach();
    }

    fn handle_revert_commits_action(
        &mut self,
        _: &RevertCommits,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.revert_commits(window, cx);
    }
}

impl Item for GitGraph {
    type Event = ItemEvent;

    fn tab_content_text(&self, _detail: usize, _cx: &App) -> SharedString {
        "Git Graph".into()
    }

    fn show_toolbar(&self) -> bool {
        false
    }

    fn to_item_events(event: &Self::Event, mut f: impl FnMut(ItemEvent)) {
        f(*event)
    }
}

impl SerializableItem for GitGraph {
    fn serialized_item_kind() -> &'static str {
        "GitGraph"
    }

    fn cleanup(
        workspace_id: workspace::WorkspaceId,
        alive_items: Vec<workspace::ItemId>,
        _window: &mut Window,
        cx: &mut App,
    ) -> Task<gpui::Result<()>> {
        workspace::delete_unloaded_items(
            alive_items,
            workspace_id,
            "git_graphs",
            &persistence::GIT_GRAPHS,
            cx,
        )
    }

    fn deserialize(
        project: Entity<Project>,
        _: WeakEntity<Workspace>,
        workspace_id: workspace::WorkspaceId,
        item_id: workspace::ItemId,
        window: &mut Window,
        cx: &mut App,
    ) -> Task<gpui::Result<Entity<Self>>> {
        if persistence::GIT_GRAPHS
            .get_git_graph(item_id, workspace_id)
            .ok()
            .is_some_and(|is_open| is_open)
        {
            let git_graph = cx.new(|cx| GitGraph::new(project, window, cx));
            Task::ready(Ok(git_graph))
        } else {
            Task::ready(Err(anyhow::anyhow!("No git graph to deserialize")))
        }
    }

    fn serialize(
        &mut self,
        workspace: &mut Workspace,
        item_id: workspace::ItemId,
        _closing: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Task<gpui::Result<()>>> {
        let workspace_id = workspace.database_id()?;
        Some(cx.background_spawn(async move {
            persistence::GIT_GRAPHS
                .save_git_graph(item_id, workspace_id, true)
                .await
        }))
    }

    fn should_serialize(&self, event: &Self::Event) -> bool {
        event == &ItemEvent::UpdateTab
    }
}

mod persistence {
    use db::{
        query,
        sqlez::{domain::Domain, thread_safe_connection::ThreadSafeConnection},
        sqlez_macros::sql,
    };
    use workspace::WorkspaceDb;

    pub struct GitGraphsDb(ThreadSafeConnection);

    impl Domain for GitGraphsDb {
        const NAME: &str = stringify!(GitGraphsDb);

        const MIGRATIONS: &[&str] = (&[sql!(
            CREATE TABLE git_graphs (
                workspace_id INTEGER,
                item_id INTEGER UNIQUE,
                is_open INTEGER DEFAULT FALSE,

                PRIMARY KEY(workspace_id, item_id),
                FOREIGN KEY(workspace_id) REFERENCES workspaces(workspace_id)
                ON DELETE CASCADE
            ) STRICT;
        )]);
    }

    db::static_connection!(GIT_GRAPHS, GitGraphsDb, [WorkspaceDb]);

    impl GitGraphsDb {
        query! {
            pub async fn save_git_graph(
                item_id: workspace::ItemId,
                workspace_id: workspace::WorkspaceId,
                is_open: bool
            ) -> Result<()> {
                INSERT OR REPLACE INTO git_graphs(item_id, workspace_id, is_open)
                VALUES (?, ?, ?)
            }
        }

        query! {
            pub fn get_git_graph(
                item_id: workspace::ItemId,
                workspace_id: workspace::WorkspaceId
            ) -> Result<bool> {
                SELECT is_open
                FROM git_graphs
                WHERE item_id = ? AND workspace_id = ?
            }
        }
    }
}
