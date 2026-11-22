//! Pin and solo entries of the flat commit menu.

use super::*;
use crate::repo::branch_order::BranchOrder;
use crate::repo::branch_visibility::Change;

pub(super) fn visibility_items(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    hash: &str,
    acts: &mut Vec<Action>,
) {
    submenu(
        ui,
        hash,
        "pin",
        crate::i18n::text("repo-core-4253"),
        "pin",
        |ui| {
            let mut order = BranchOrder::load(ui.ctx(), tab.repo_key());
            let pinned = order.is_pinned(&ctx.name);
            let label = crate::i18n::text(if pinned {
                "repo-core-3400"
            } else {
                "repo-core-3401"
            });
            let response = MenuItem::new(label)
                .icon("pin")
                .show(ui)
                .on_hover_text(crate::i18n::text("repo-core-3403"));
            if response.clicked() {
                order.toggle_pin(&ctx.name);
                order.store(ui.ctx(), tab.repo_key());
            }
        },
    );
    submenu(
        ui,
        hash,
        "solo",
        crate::i18n::text("repo-core-3554"),
        "eye",
        |ui| {
            let Some(full) = ctx.full.clone() else {
                return;
            };
            let is_head = ctx.local.is_some_and(|b| b.is_head);
            let shown = tab.branch_shown(&full, is_head);
            let hide = crate::i18n::text(if shown {
                "repo-core-3549"
            } else {
                "repo-core-3550"
            });
            let checked_out = crate::i18n::text("repo-core-3552");
            let response = MenuItem::new(hide)
                .icon("eye-off")
                .refused(is_head.then_some(checked_out))
                .show(ui);
            if response.clicked() {
                acts.push(Action::BranchVisibility(Change::Toggle(full.clone())));
            }
            let solo = if tab.visibility.solo.contains(&full) {
                crate::i18n::text("repo-core-3553")
            } else {
                crate::i18n::text("repo-core-3554")
            };
            let response = MenuItem::new(solo)
                .icon("eye")
                .show(ui)
                .on_hover_text(crate::i18n::text("repo-core-3555"));
            if response.clicked() {
                acts.push(Action::BranchVisibility(Change::Solo(full.clone())));
            }
            if tab.visibility.is_filtering()
                && MenuItem::new(crate::i18n::text("repo-core-3556"))
                    .icon("eye")
                    .show(ui)
                    .clicked()
            {
                acts.push(Action::BranchVisibility(Change::ShowAll));
            }
        },
    );
}

pub(super) fn tag_items(
    tab: &RepoTab,
    ui: &mut Ui,
    hash: &str,
    short: &str,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let response = MenuItem::new(crate::i18n::text("repo-core-2642"))
        .icon("tag")
        .enabled(idle)
        .show(ui);
    if response.clicked() {
        acts.push(Action::Dialog(Dialog::tag(
            hash,
            short,
            tab.default_remote(),
        )));
    }
    let response = MenuItem::new(crate::i18n::text("repo-core-4254"))
        .icon("tag")
        .enabled(idle)
        .show(ui)
        .on_hover_text(crate::i18n::text("repo-core-1222"));
    if response.clicked() {
        let mut dialog = Dialog::tag(hash, short, tab.default_remote());
        if let Dialog::Tag { message, .. } = &mut dialog {
            message.push_str("Tag message");
        }
        acts.push(Action::Dialog(dialog));
    }
}
