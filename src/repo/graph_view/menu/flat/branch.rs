//! Checkout and worktree submenus of the flat commit menu.

use super::*;

pub(super) fn checkout_items(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    hash: &str,
    target: &str,
    is_head: bool,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    submenu(
        ui,
        hash,
        "checkout",
        crate::i18n::text("repo-core-4258"),
        "log-in",
        |ui| {
            if let Some(b) = ctx.remote {
                action_item(
                    ui,
                    b.name.clone(),
                    "log-in",
                    idle,
                    None,
                    acts,
                    Action::CheckoutRemote(b.name.clone()),
                );
            } else if let Some(b) = ctx.local
                && b.name != target
            {
                action_item(
                    ui,
                    crate::i18n::message("repo-core-2632", &[("branch", b.name.clone().into())]),
                    "log-in",
                    idle,
                    None,
                    acts,
                    Action::Checkout(b.name.clone()),
                );
            }
            let checked_out = crate::i18n::text("repo-core-0796");
            let response = MenuItem::new(crate::i18n::text("repo-core-4244"))
                .icon("log-in")
                .enabled(idle)
                .refused(is_head.then_some(checked_out))
                .show(ui);
            if response.clicked() {
                acts.push(Action::Checkout(hash.to_owned()));
            }
        },
    );
    let _ = tab;
}

pub(super) fn worktree_items(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    hash: &str,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    submenu(
        ui,
        hash,
        "worktree",
        crate::i18n::text("repo-core-4245"),
        "folder-git-2",
        |ui| {
            if let Some(b) = ctx.local {
                let label =
                    crate::i18n::message("repo-core-2639", &[("name", b.name.clone().into())]);
                let response = MenuItem::new(label)
                    .icon("folder-git-2")
                    .enabled(idle)
                    .show(ui);
                if response.clicked() {
                    acts.push(Action::Dialog(Dialog::add_worktree(
                        &tab.path,
                        Some(&b.name),
                        &tab.refs.head,
                    )));
                }
            }
            let response = MenuItem::new(crate::i18n::text("repo-core-4244"))
                .icon("folder-git-2")
                .enabled(idle)
                .show(ui);
            if response.clicked() {
                acts.push(Action::Dialog(Dialog::worktree_at(&tab.path, hash)));
            }
        },
    );
}
