//! Pull, push and upstream entries of the flat commit menu.

use super::*;
use crate::repo::pull::PullMode;

pub(super) fn sync_items(
    tab: &RepoTab,
    ui: &mut Ui,
    ctx: &Context<'_>,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let pull = MenuItem::new(crate::i18n::text("repo-core-3093"))
        .icon("cloud-download")
        .enabled(idle)
        .show(ui)
        .on_hover_text(crate::repo::messages::display_operation(
            PullMode::Merge.label(),
        ));
    if pull.clicked() {
        acts.push(Action::PullWith(PullMode::Merge));
    }

    let no_branch = crate::i18n::text("repo-core-4259");
    let push = ctx.local.and_then(|b| tab.push_branch(b));
    let destination = push.as_ref().map(|(to, _)| to.clone());
    let response = MenuItem::new(crate::i18n::text("repo-core-0032"))
        .icon("arrow-up-from-line")
        .enabled(idle)
        .refused(push.is_none().then_some(no_branch))
        .show(ui);
    let response = if let Some(to) = destination {
        response.on_hover_text(crate::i18n::message("repo-core-3408", &[("to", to.into())]))
    } else {
        response
    };
    if response.clicked()
        && let Some((_, action)) = push
    {
        acts.push(action);
    }

    let response = MenuItem::new(crate::i18n::text("repo-core-4261"))
        .icon("git-branch")
        .enabled(idle)
        .refused(ctx.local.is_none().then_some(no_branch))
        .show(ui);
    if response.clicked()
        && let Some(b) = ctx.local
    {
        acts.push(Action::Dialog(tab.tracking_dialog(&b.name)));
    }
}
