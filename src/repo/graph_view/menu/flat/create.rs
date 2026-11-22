//! Create, reset, edit and revert entries of the flat commit menu.

use super::*;
use crate::repo::quick_edit::{self, Edit};

#[allow(clippy::too_many_arguments)]
pub(super) fn create_items(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    hash: &str,
    short: &str,
    summary: &str,
    target: &str,
    merge: bool,
    is_head: bool,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    action_item(
        ui,
        crate::i18n::text("repo-core-2641"),
        "git-branch-plus",
        true,
        None,
        acts,
        Action::Dialog(Dialog::create_branch(hash, short)),
    );
    reset_item(
        tab, ui, ctx, hash, short, summary, target, merge, is_head, idle, acts,
    );
    edit_message_item(tab, ui, hash, idle, acts);
    action_item(
        ui,
        crate::i18n::text("repo-core-2655"),
        "undo-2",
        idle,
        None,
        acts,
        crate::repo::commit_actions::revert_action(hash, short),
    );
}

#[allow(clippy::too_many_arguments)]
fn reset_item(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    hash: &str,
    short: &str,
    summary: &str,
    target: &str,
    merge: bool,
    is_head: bool,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let upstream = tab
        .refs
        .current
        .as_deref()
        .and_then(|b| tab.upstream_ref(b));
    let rewrite = Rewrite {
        tab,
        merge,
        hash,
        short,
        summary,
        target,
        head: &tab.refs.head,
        upstream: upstream.as_deref(),
    };
    let label = crate::i18n::message("repo-core-4246", &[("name", ctx.name.clone().into())]);
    let checked_out = crate::i18n::text("repo-core-0796");
    submenu(ui, hash, "reset", label, "undo-2", |ui| {
        if !idle {
            ui.label(crate::i18n::text("dev-integration-ui-busy-menu"));
        }
        if is_head {
            ui.label(checked_out);
        } else {
            reset_items(ui, &rewrite, acts);
        }
    });
}

fn edit_message_item(tab: &RepoTab, ui: &mut Ui, hash: &str, idle: bool, acts: &mut Vec<Action>) {
    let line = quick_edit::first_parents(&tab.history, &tab.history_index, &tab.refs.head);
    let blocked = if tab.refs.current.is_none() {
        Some(crate::i18n::text("repo-core-0842").to_owned())
    } else {
        tab.work.in_progress.map(|s| {
            crate::i18n::message(
                "repo-core-3192",
                &[("to-lowercase", s.label().to_lowercase().into())],
            )
        })
    };
    let refused =
        blocked.or_else(|| quick_edit::refusal(Edit::Message, &line, hash).map(str::to_owned));
    let response = MenuItem::new(crate::i18n::text("repo-core-3172"))
        .icon("wrench")
        .enabled(idle)
        .refused(refused.as_deref())
        .show(ui)
        .on_hover_text(crate::i18n::text("repo-core-3176"));
    if response.clicked() {
        acts.push(Action::EditMessage(hash.to_owned()));
    }
}
