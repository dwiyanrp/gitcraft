//! Create, reset, edit and revert entries of the flat commit menu.

use super::*;
use crate::repo::quick_edit::{self, Edit};

#[allow(clippy::too_many_arguments)]
pub(super) fn create_items(
    tab: &RepoTab,
    ui: &mut Ui,
    hash: &str,
    short: &str,
    summary: &str,
    target: &str,
    merge: bool,
    is_head: bool,
    has_branch: bool,
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
    cherry_pick_item(tab, ui, hash, short, merge, is_head, idle, acts);
    if !has_branch {
        rebase_onto_commit_item(tab, ui, hash, short, target, is_head, idle, acts);
    }
    reset_item(
        tab, ui, hash, short, summary, target, merge, is_head, idle, acts,
    );
    if has_branch {
        edit_message_item(tab, ui, hash, idle, acts);
    }
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
fn cherry_pick_item(
    tab: &RepoTab,
    ui: &mut Ui,
    hash: &str,
    short: &str,
    merge: bool,
    is_head: bool,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let checked_out = crate::i18n::text("repo-core-0796");
    let cherry_pick = crate::i18n::text("repo-core-4264").to_owned();
    let picked = if merge {
        MenuItem::new(cherry_pick)
            .icon("command")
            .enabled(idle)
            .refused(is_head.then_some(checked_out))
            .show(ui)
    } else {
        tab.conflict_button(ui, cherry_pick, idle && !is_head, Op::CherryPick, hash)
    };
    if picked.clicked() {
        acts.push(crate::repo::commit_actions::cherry_pick_action(hash, short));
    }
}

fn rebase_onto_commit_item(
    tab: &RepoTab,
    ui: &mut Ui,
    hash: &str,
    short: &str,
    target: &str,
    is_head: bool,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let rebase_label = crate::i18n::message("repo-core-4265", &[("target", target.to_string().into())]);
    let rebase = tab.conflict_button(ui, rebase_label, idle && !is_head, Op::Rebase, hash);
    offer(&rebase, || GhostOp::Rebase {
        onto: hash.to_owned(),
        label: short.to_owned(),
    });
    if rebase.clicked() {
        acts.push(Action::AskRebase {
            onto: hash.to_owned(),
            label: format!("Rebase {target} onto {short}"),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn reset_item(
    tab: &RepoTab,
    ui: &mut Ui,
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
    let label = crate::i18n::message("repo-core-4246", &[("name", target.to_string().into())]);
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
