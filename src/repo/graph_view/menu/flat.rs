//! The flat, GitKraken-style right-click list of a commit row.
//!
//! The screenshot-like entries stay at the root. Less common actions remain available in the
//! final _More actions_ submenu, which holds the grouped menu without changing its behaviour.

use egui::{Response, Ui};

use super::*;

mod actions;
mod branch;
mod create;
mod review;
mod sync;
mod visibility;

use actions::*;
use branch::*;
use create::*;
use review::{
    branch_items, copy_items as link_copy_items, create_patch_item, patch_items, review_item,
};
use sync::*;
use visibility::*;

/// The branch a flat-menu action acts on: the pill that was right-clicked, the first local
/// branch at the commit, or the checked-out one.
pub(super) struct Context<'a> {
    pub(super) local: Option<&'a git::Branch>,
    pub(super) remote: Option<&'a git::Branch>,
    /// How the branch is named in a menu entry (`dev`, or `origin/dev` for a remote pill).
    pub(super) name: String,
    /// Full ref for links and graph visibility, when there is one.
    pub(super) full: Option<String>,
    pub(super) remote_name: Option<String>,
}

pub(super) fn show(
    tab: &RepoTab,
    ui: &mut Ui,
    i: usize,
    pill: Option<String>,
    idle: bool,
    acts: &mut Vec<Action>,
) {
    let Some(commit) = tab.commits.get(i) else {
        return;
    };
    let hash = commit.hash.clone();
    let short = crate::repo::short(&hash).to_owned();
    let summary = commit.summary.clone();
    let merge = commit.parents.len() > 1;
    let is_head = hash == tab.refs.head;
    let ctx = context(tab, i, pill.as_deref());
    let target = tab
        .refs
        .current
        .clone()
        .unwrap_or_else(|| "HEAD".to_owned());

    sync_items(tab, ui, &ctx, idle, acts);
    ui.separator();
    checkout_items(ui, &ctx, &hash, &target, is_head, idle, acts);
    worktree_items(tab, ui, &ctx, &hash, idle, acts);
    ui.separator();
    create_items(
        tab, ui, &hash, &short, &summary, &target, merge, is_head, idle, acts,
    );
    ui.separator();
    review_item(tab, ui, &ctx, idle, acts);
    ui.separator();
    patch_items(ui, is_head, idle, acts);
    branch_items(tab, ui, &ctx, &hash, idle, acts);
    ui.separator();
    link_copy_items(tab, ui, &ctx, &hash, acts);
    create_patch_item(ui, &hash, &summary, merge, acts);
    ui.separator();
    visibility_items(tab, ui, &ctx, &hash, acts);
    ui.separator();
    tag_items(tab, ui, &hash, &short, idle, acts);
    ui.separator();

    submenu(
        ui,
        &hash,
        "more",
        crate::i18n::text("repo-core-4255"),
        "command",
        |ui| {
            tab.grouped_menu_contents(ui, i, pill, idle, acts);
        },
    );
}

fn context<'a>(tab: &'a RepoTab, i: usize, pill: Option<&'a str>) -> Context<'a> {
    let mut remote = None;
    let mut remote_name = None;
    if let Some(short) = pill.and_then(|p| p.strip_prefix("refs/remotes/")) {
        remote = tab
            .refs
            .remotes
            .iter()
            .flat_map(|(_, bs)| bs)
            .find(|b| b.name == short);
        remote_name = tab
            .remotes
            .iter()
            .map(|r| r.name.as_str())
            .find(|r| short == *r || short.starts_with(&format!("{r}/")))
            .map(str::to_owned)
            .or_else(|| short.split('/').next().map(str::to_owned));
    }
    let mut local = pill
        .and_then(|p| p.strip_prefix("refs/heads/"))
        .and_then(|n| tab.refs.local.iter().find(|b| b.name == n));
    if local.is_none()
        && let Some(short) = remote.map(|b| b.name.as_str())
    {
        local = tab.refs.local.iter().find(|b| b.upstream == short);
    }
    if local.is_none()
        && let Some(full) = tab.copy_branch_ref(i)
        && let Some(n) = full.strip_prefix("refs/heads/")
    {
        local = tab.refs.local.iter().find(|b| b.name == n);
    }
    if local.is_none()
        && let Some(current) = tab.refs.current.as_deref()
    {
        local = tab.refs.local.iter().find(|b| b.name == current);
    }
    if remote.is_none()
        && let Some(b) = local
        && !b.upstream.is_empty()
        && let Some((name, short)) = tab.split_upstream(&b.upstream)
    {
        remote_name = Some(name.clone());
        remote = tab
            .refs
            .remotes
            .iter()
            .flat_map(|(_, bs)| bs)
            .find(|r| r.name == b.upstream || r.name == short);
    }
    let full = local
        .map(|b| format!("refs/heads/{}", b.name))
        .or_else(|| remote.map(|b| format!("refs/remotes/{}", b.name)));
    let name = local
        .map(|b| b.name.clone())
        .or_else(|| remote.map(|b| b.name.clone()))
        .unwrap_or_else(|| "HEAD".to_owned());
    Context {
        local,
        remote,
        name,
        full,
        remote_name,
    }
}

/// A simple flat entry: show it, then push `action` when clicked.
pub(super) fn action_item(
    ui: &mut Ui,
    label: impl Into<egui::WidgetText>,
    icon: &'static str,
    enabled: bool,
    refused: Option<&str>,
    acts: &mut Vec<Action>,
    action: Action,
) -> Response {
    let response = MenuItem::new(label)
        .icon(icon)
        .enabled(enabled)
        .refused(refused)
        .show(ui);
    if response.clicked() {
        acts.push(action);
    }
    response
}

/// Submenus of the flat menu need a stable id: refs can be reordered while they are open.
pub(super) fn submenu(
    ui: &mut Ui,
    hash: &str,
    salt: &str,
    label: impl Into<egui::WidgetText>,
    icon: &'static str,
    content: impl FnOnce(&mut Ui),
) {
    let id = ui.make_persistent_id(("flat-commit-menu", hash, salt));
    ui.scope_builder(egui::UiBuilder::new().id(id), |ui| {
        MenuItem::new(label).icon(icon).submenu(ui, content);
    });
}

#[cfg(test)]
#[path = "flat_tests.rs"]
mod tests;
