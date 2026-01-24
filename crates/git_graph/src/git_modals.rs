use editor::Editor;
use gpui::{
    App, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, ParentElement, Render, SharedString, Styled, Window, actions, div, prelude::*,
};
use ui::{Checkbox, Label, ToggleState, prelude::*};

actions!(git_modals, [Confirm, Cancel]);

#[derive(Clone)]
pub enum ModalAction {
    DeleteBranch {
        branch_name: SharedString,
        is_remote: bool,
        delete_remote: bool,
    },
    SquashCommits {
        commit_count: usize,
        message: SharedString,
    },
    DropCommits {
        commit_count: usize,
    },
    RewordCommits {
        commit_count: usize,
        message: SharedString,
    },
    EditAmendCommit {
        current_message: SharedString,
        amend: bool,
    },
    CheckoutBranch {
        branch_name: SharedString,
        has_uncommitted_changes: bool,
        stash: bool,
    },
    MergeBranch {
        branch_name: SharedString,
    },
    RebaseOnto {
        target_branch: SharedString,
    },
    RevertCommits {
        commit_count: usize,
    },
    CherryPickConflict {
        abort: bool,
    },
    RevertConflict {
        abort: bool,
    },
}

pub struct GitModal {
    focus_handle: FocusHandle,
    pub action: ModalAction,
    pub on_confirm: Box<dyn Fn(ModalAction, &mut Window, &mut Context<Self>)>,
    message_input: Option<Entity<Editor>>,
    delete_remote_checkbox: bool,
    stash_checkbox: bool,
}

impl GitModal {
    pub fn new<F>(
        action: ModalAction,
        on_confirm: F,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self
    where
        F: Fn(ModalAction, &mut Window, &mut Context<Self>) + 'static,
    {
        let focus_handle = cx.focus_handle();

        let message_input = if matches!(
            action,
            ModalAction::SquashCommits { .. }
                | ModalAction::RewordCommits { .. }
                | ModalAction::EditAmendCommit { .. }
        ) {
            let mut editor = cx.new(|cx| {
                let mut editor = Editor::single_line(window, cx);
                editor.set_placeholder_text("Enter commit message...", window, cx);
                editor
            });

            // Pre-fill with current message for RewordCommits and EditAmendCommit
            match &action {
                ModalAction::RewordCommits { message, .. }
                | ModalAction::EditAmendCommit {
                    current_message: message,
                    ..
                } => {
                    if !message.is_empty() {
                        editor.update(cx, |editor, cx| {
                            editor.set_text(message.as_ref(), window, cx);
                        });
                    }
                }
                _ => {}
            };

            Some(editor)
        } else {
            None
        };

        Self {
            focus_handle,
            action,
            on_confirm: Box::new(on_confirm),
            message_input,
            delete_remote_checkbox: false,
            stash_checkbox: true, // Default to stashing
        }
    }

    fn confirm(&mut self, _: &menu::Confirm, window: &mut Window, cx: &mut Context<Self>) {
        let mut action = self.action.clone();

        // Update action based on checkbox states
        match &mut action {
            ModalAction::DeleteBranch { delete_remote, .. } => {
                *delete_remote = self.delete_remote_checkbox;
            }
            ModalAction::CheckoutBranch { stash, .. } => {
                *stash = self.stash_checkbox;
            }
            ModalAction::SquashCommits { message, .. } => {
                if let Some(input) = &self.message_input {
                    let text: String = input.read(cx).text(cx);
                    *message = text.into();
                }
            }
            ModalAction::RewordCommits { message, .. } => {
                if let Some(input) = &self.message_input {
                    let text: String = input.read(cx).text(cx);
                    *message = text.into();
                }
            }
            ModalAction::EditAmendCommit {
                current_message, ..
            } => {
                if let Some(input) = &self.message_input {
                    let text: String = input.read(cx).text(cx);
                    *current_message = text.into();
                }
            }
            _ => {}
        }

        (self.on_confirm)(action, window, cx);
        cx.emit(DismissEvent);
    }

    fn cancel(&mut self, _: &menu::Cancel, _window: &mut Window, cx: &mut Context<Self>) {
        cx.emit(DismissEvent);
    }

    fn render_delete_branch(
        &self,
        branch_name: &str,
        is_remote: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                Label::new(format!(
                    "Delete {} branch '{}'?",
                    if is_remote { "remote" } else { "local" },
                    branch_name
                ))
                .size(LabelSize::Large),
            )
            .child(
                Label::new("This action cannot be undone.")
                    .size(LabelSize::Small)
                    .color(Color::Warning),
            )
            .when(!is_remote, |this| {
                this.child(
                    Checkbox::new(
                        "delete-remote-checkbox",
                        ToggleState::from(self.delete_remote_checkbox),
                    )
                    .label("Also delete remote branch")
                    .on_click(cx.listener(
                        |this, state: &ToggleState, _window, cx| {
                            this.delete_remote_checkbox = state.selected();
                            cx.notify();
                        },
                    )),
                )
            })
    }

    fn render_checkout_branch(
        &self,
        branch_name: &str,
        has_uncommitted: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Label::new(format!("Checkout branch '{}'?", branch_name)).size(LabelSize::Large))
            .when(has_uncommitted, |this| {
                this.child(
                    Label::new("You have uncommitted changes.")
                        .size(LabelSize::Small)
                        .color(Color::Warning),
                )
                .child(
                    Checkbox::new("stash-checkbox", ToggleState::from(self.stash_checkbox))
                        .label("Stash changes before checkout")
                        .on_click(cx.listener(|this, state: &ToggleState, _window, cx| {
                            this.stash_checkbox = state.selected();
                            cx.notify();
                        })),
                )
            })
    }

    fn render_squash_commits(&self, count: usize, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Label::new(format!("Squash {} commits", count)).size(LabelSize::Large))
            .child(
                Label::new("Enter a commit message for the squashed commit:")
                    .size(LabelSize::Small),
            )
            .when_some(self.message_input.clone(), |this, input| this.child(input))
    }

    fn render_merge_branch(&self, branch_name: &str) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                Label::new(format!("Merge '{}' into current branch?", branch_name))
                    .size(LabelSize::Large),
            )
            .child(Label::new("This will create a merge commit.").size(LabelSize::Small))
    }

    fn render_rebase_onto(&self, target_branch: &str) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                Label::new(format!("Rebase current branch onto '{}'?", target_branch))
                    .size(LabelSize::Large),
            )
            .child(
                Label::new("This will rewrite commit history.")
                    .size(LabelSize::Small)
                    .color(Color::Warning),
            )
    }

    fn render_drop_commits(&self, count: usize) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Label::new(format!("Drop {} commits?", count)).size(LabelSize::Large))
            .child(
                Label::new(
                    "This will permanently delete the selected commits and cannot be undone.",
                )
                .size(LabelSize::Small)
                .color(Color::Error),
            )
            .child(
                Label::new("Only drop commits that have not been pushed to a shared repository.")
                    .size(LabelSize::Small)
                    .color(Color::Warning),
            )
    }

    fn render_revert_commits(&self, count: usize) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Label::new(format!("Revert {} commits?", count)).size(LabelSize::Large))
            .child(
                Label::new(
                    "This will create new commits that undo the changes from the selected commits.",
                )
                .size(LabelSize::Small),
            )
            .child(
                Label::new("The original commits will remain in the history.")
                    .size(LabelSize::Small)
                    .color(Color::Muted),
            )
    }

    fn render_cherry_pick_conflict(&self) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Label::new("Cherry-pick conflicts detected").size(LabelSize::Large))
            .child(
                Label::new("Conflicts occurred during cherry-pick. Resolve conflicts manually or choose an action:")
                    .size(LabelSize::Small),
            )
    }

    fn render_revert_conflict(&self) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Label::new("Revert conflicts detected").size(LabelSize::Large))
            .child(
                Label::new("Conflicts occurred during revert. Resolve conflicts manually or choose an action:")
                    .size(LabelSize::Small),
            )
    }

    fn render_reword_commits(
        &self,
        count: usize,
        message: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(Label::new(format!("Reword {} commits", count)).size(LabelSize::Large))
            .child(Label::new("Enter a new commit message:").size(LabelSize::Small))
            .when_some(self.message_input.clone(), |this, input| this.child(input))
    }

    fn render_edit_amend_commit(
        &self,
        current_message: &str,
        amend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .gap_3()
            .child(
                Label::new(if amend { "Amend commit" } else { "Edit commit" })
                    .size(LabelSize::Large),
            )
            .child(Label::new("Enter a new commit message:").size(LabelSize::Small))
            .when_some(self.message_input.clone(), |this, input| this.child(input))
            .child(
                Checkbox::new("amend-checkbox", ToggleState::from(amend))
                    .label("Amend (update commit timestamp and author)")
                    .on_click(cx.listener(|this, state: &ToggleState, _window, cx| {
                        // Update the action's amend field
                        if let ModalAction::EditAmendCommit { amend, .. } = &mut this.action {
                            *amend = state.selected();
                        }
                        cx.notify();
                    })),
            )
    }
}

impl EventEmitter<DismissEvent> for GitModal {}
impl Focusable for GitModal {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GitModal {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match &self.action {
            ModalAction::DeleteBranch {
                branch_name,
                is_remote,
                ..
            } => self
                .render_delete_branch(branch_name, *is_remote, cx)
                .into_any_element(),
            ModalAction::CheckoutBranch {
                branch_name,
                has_uncommitted_changes,
                ..
            } => self
                .render_checkout_branch(branch_name, *has_uncommitted_changes, cx)
                .into_any_element(),
            ModalAction::SquashCommits { commit_count, .. } => self
                .render_squash_commits(*commit_count, cx)
                .into_any_element(),
            ModalAction::DropCommits { commit_count } => {
                self.render_drop_commits(*commit_count).into_any_element()
            }
            ModalAction::RewordCommits {
                commit_count,
                message,
            } => self
                .render_reword_commits(*commit_count, message, cx)
                .into_any_element(),
            ModalAction::EditAmendCommit {
                current_message,
                amend,
            } => self
                .render_edit_amend_commit(current_message, *amend, window, cx)
                .into_any_element(),
            ModalAction::MergeBranch { branch_name } => {
                self.render_merge_branch(branch_name).into_any_element()
            }
            ModalAction::RebaseOnto { target_branch } => {
                self.render_rebase_onto(target_branch).into_any_element()
            }
            ModalAction::RevertCommits { commit_count } => {
                self.render_revert_commits(*commit_count).into_any_element()
            }
            ModalAction::CherryPickConflict { .. } => {
                self.render_cherry_pick_conflict().into_any_element()
            }
            ModalAction::RevertConflict { .. } => self.render_revert_conflict().into_any_element(),
        };

        div()
            .elevation_3(cx)
            .p_4()
            .gap_4()
            .min_w(px(400.))
            .max_w(px(600.))
            .bg(cx.theme().colors().elevated_surface_background)
            .border_1()
            .border_color(cx.theme().colors().border)
            .rounded_lg()
            .shadow_lg()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::cancel))
            .child(content)
            .child(
                h_flex()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("cancel", "Cancel")
                            .style(ButtonStyle::Subtle)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.cancel(&menu::Cancel, window, cx);
                            })),
                    )
                    .child(
                        Button::new("confirm", "Confirm")
                            .style(ButtonStyle::Filled)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.confirm(&menu::Confirm, window, cx);
                            })),
                    )
                    .children(match &self.action {
                        ModalAction::CherryPickConflict { .. }
                        | ModalAction::RevertConflict { .. } => Some(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("abort", "Abort")
                                        .style(ButtonStyle::Subtle)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            match &mut this.action {
                                                ModalAction::CherryPickConflict { abort } => {
                                                    *abort = true
                                                }
                                                ModalAction::RevertConflict { abort } => {
                                                    *abort = true
                                                }
                                                _ => {}
                                            }
                                            this.confirm(&menu::Confirm, window, cx);
                                        })),
                                )
                                .child(
                                    Button::new("continue", "Continue")
                                        .style(ButtonStyle::Filled)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            match &mut this.action {
                                                ModalAction::CherryPickConflict { abort } => {
                                                    *abort = false
                                                }
                                                ModalAction::RevertConflict { abort } => {
                                                    *abort = false
                                                }
                                                _ => {}
                                            }
                                            this.confirm(&menu::Confirm, window, cx);
                                        })),
                                ),
                        ),
                        _ => None,
                    }),
            )
    }
}
