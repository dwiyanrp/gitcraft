//! Review, patch, branch and link entries of the flat commit menu.

use super::*;
use crate::forge::links::Target;
use crate::repo::sidebar::remote_short;
use crate::settings::Settings;

pub(super) fn review_item(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let branch = ctx.name.clone();
    let base = tab
        .base_ref()
        .map(|b| b.name)
        .filter(|name| *name != branch)
        .or_else(|| tab.refs.current.clone().filter(|name| *name != branch));
    let no_base = crate::i18n::text("repo-core-3396");
    let response = MenuItem::new(crate::i18n::text("repo-core-4247"))
        .icon("eye")
        .enabled(idle)
        .refused(base.is_none().then_some(no_base))
        .show(ui);
    let tip = if Settings::get(ui.ctx()).ai.enabled()
        && let Some(base) = &base
    {
        crate::i18n::message(
            "repo-core-3398",
            &[
                ("name", branch.clone().into()),
                ("base", base.clone().into()),
            ],
        )
    } else {
        crate::i18n::text("repo-core-3399").to_owned()
    };
    let response = response.on_hover_text(tip);
    if response.clicked()
        && let Some(base) = base
    {
        acts.push(Action::ReviewBranch { branch, base });
    }
}

pub(super) fn patch_items(ui: &mut Ui, idle: bool, acts: &mut Vec<Action>) {
    super::super::apply_patch_item(ui, idle, acts);
}

pub(super) fn create_patch_item(
    ui: &mut Ui,
    hash: &str,
    summary: &str,
    merge: bool,
    acts: &mut Vec<Action>,
) {
    let no_patch = crate::i18n::text("repo-core-2628");
    let response = MenuItem::new(crate::i18n::text("repo-core-2627"))
        .icon("file-diff")
        .refused(merge.then_some(no_patch))
        .show(ui)
        .on_hover_text(crate::i18n::text("repo-core-2629"));
    if response.clicked() {
        acts.push(Action::SavePatch {
            hash: hash.to_owned(),
            summary: summary.to_owned(),
        });
    }
}

pub(super) fn branch_items(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let no_branch = crate::i18n::text("repo-core-4259");
    let rename = MenuItem::new(crate::i18n::message(
        "repo-core-4248",
        &[("name", ctx.name.clone().into())],
    ))
    .icon("wrench")
    .enabled(idle)
    .refused(ctx.local.is_none().then_some(no_branch))
    .show(ui);
    if rename.clicked()
        && let Some(b) = ctx.local
    {
        acts.push(Action::Dialog(Dialog::rename_branch(&b.name)));
    }

    let refusal = ctx.local.and_then(|b| delete_refusal(tab, b));
    let delete = MenuItem::new(crate::i18n::message(
        "repo-core-4256",
        &[("name", ctx.name.clone().into())],
    ))
    .icon("minus")
    .enabled(idle)
    .refused(refusal)
    .show(ui);
    if delete.clicked()
        && let Some(b) = ctx.local
    {
        acts.push(local_delete_action(tab, b));
    }

    let no_remote = crate::i18n::text("repo-core-4260");
    let remote_name = ctx.remote.map(|b| b.name.as_str()).unwrap_or_default();
    let delete_remote = MenuItem::new(crate::i18n::message(
        "repo-core-4256",
        &[("name", remote_name.to_string().into())],
    ))
    .icon("minus")
    .enabled(idle)
    .refused(ctx.remote.is_none().then_some(no_remote))
    .show(ui);
    if delete_remote.clicked()
        && let Some(b) = ctx.remote
        && let Some(remote) = ctx.remote_name.as_deref()
    {
        let short = remote_short(&b.name, remote).unwrap_or(&b.name);
        acts.push(remote_delete_action(tab, remote, short, b));
    }

    let combined = ctx
        .local
        .and_then(|b| both_delete_action(tab, b))
        .filter(|_| refusal.is_none());
    let both_name = ctx.remote.map(|b| b.name.as_str()).unwrap_or_default();
    let delete_both = MenuItem::new(crate::i18n::message(
        "repo-core-4257",
        &[
            ("name", ctx.name.clone().into()),
            ("name2", both_name.to_string().into()),
        ],
    ))
    .icon("minus")
    .enabled(idle)
    .refused(combined.is_none().then_some(no_remote))
    .show(ui);
    if delete_both.clicked()
        && let Some(action) = combined
    {
        acts.push(action);
    }
}

pub(super) fn copy_items(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    hash: &str,
    acts: &mut Vec<Action>,
) {
    if MenuItem::new(crate::i18n::text("repo-core-4249"))
        .icon("copy")
        .show(ui)
        .clicked()
    {
        ui.ctx().copy_text(ctx.name.clone());
    }
    if MenuItem::new(crate::i18n::text("repo-core-4250"))
        .icon("copy")
        .show(ui)
        .clicked()
    {
        ui.ctx().copy_text(hash.to_owned());
    }
    if let Some(full) = ctx.full.as_deref()
        && let Some((menu, target)) = tab.branch_link_menu(full)
    {
        let link_name = ctx
            .remote
            .map(|b| b.name.clone())
            .unwrap_or_else(|| ctx.name.clone());
        let what = crate::i18n::message("repo-core-4251", &[("name", link_name.into())]);
        menu.copy_items(ui, &what, &target, acts);
    }
    if let Some(menu) = tab.link_menu(ctx.full.as_deref()) {
        let remote = ctx
            .remote_name
            .clone()
            .or_else(|| tab.default_remote())
            .unwrap_or_default();
        let what = crate::i18n::message("repo-core-4252", &[("remote", remote.into())]);
        let target = Target::Commit {
            rev: hash.to_owned(),
        };
        menu.copy_items(ui, &what, &target, acts);
    }
}
