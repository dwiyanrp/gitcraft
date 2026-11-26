//! Delete dialogs built by the flat commit menu.
//!
//! These are the same guarded operations as the sidebar menus; the flat list only changes how
//! the choices are presented.

use super::*;
use crate::repo::operations::expect_ref_at;
use crate::repo::sidebar::branch_delete::{delete_both, delete_both_undo, redo_delete};

/// Delete `b` here: refuse only at run time if HEAD or a worktree moved since the menu opened.
pub(super) fn local_delete_action(tab: &RepoTab, b: &git::Branch) -> Action {
    let restore = ops(&[&["branch", "--", &b.name, &b.target]]);
    let at_seen = || vec![expect_ref_at(&format!("refs/heads/{}", b.name), &b.target)];
    let guarded = |steps: Vec<Vec<String>>| [at_seen(), steps].concat();
    Action::Dialog(
        Dialog::confirm(
            "Delete branch",
            crate::i18n::message("repo-core-3362", &[("name", b.name.clone().into())]),
            crate::i18n::text("repo-core-1115"),
            guarded(ops(&[&["branch", "-d", "--", &b.name]])),
            Some((
                crate::i18n::text("repo-core-3363").to_owned(),
                guarded(ops(&[&["branch", "-D", "--", &b.name]])),
            )),
        )
        .undo(Undo::Custom {
            undo: restore.clone(),
            redo: redo_delete(&b.name, ops(&[&["branch", "-D", "--", &b.name]])),
        })
        .preview_of(tab.delete_preview(&[&b.target], vec![format!("refs/heads/{}", b.name)])),
    )
}

/// Delete `b` and the branch it tracks on the remote, if that branch is still there.
pub(super) fn both_delete_action(tab: &RepoTab, b: &git::Branch) -> Option<Action> {
    let ((remote, name), tracked) = (!b.upstream_gone())
        .then(|| tab.split_upstream(&b.upstream))
        .flatten()
        .zip(
            tab.refs
                .remotes
                .iter()
                .flat_map(|(_, bs)| bs)
                .find(|r| r.name == b.upstream),
        )?;
    let restore = ops(&[&["branch", "--", &b.name, &b.target]]);
    let local_ref = format!("refs/heads/{}", b.name);
    let at_seen = || vec![expect_ref_at(&local_ref, &b.target)];
    let guarded = |steps: Vec<Vec<String>>| [at_seen(), steps].concat();
    let delete = |flag: &str| guarded(delete_both(flag, &b.name, &remote, name, &tracked.target));
    let undo = delete_both_undo(
        restore,
        &remote,
        &format!("{}:refs/heads/{name}", tracked.target),
        tab.track_ops(&b.name, &b.upstream),
    );
    Some(Action::Dialog(
        Dialog::confirm(
            "Delete branch locally and on the remote",
            crate::i18n::message(
                "repo-core-3368",
                &[
                    ("name", b.name.clone().into()),
                    ("name2", name.to_string().into()),
                    ("remote", remote.clone().into()),
                ],
            ),
            crate::i18n::text("repo-core-3369"),
            delete("-d"),
            Some((crate::i18n::text("repo-core-3363").to_owned(), delete("-D"))),
        )
        .undo(Undo::Custom {
            undo,
            redo: redo_delete(&b.name, delete("-D")),
        })
        .preview_of(tab.delete_preview(
            &[&b.target, &tracked.target],
            vec![local_ref, format!("refs/remotes/{}", tracked.name)],
        )),
    ))
}

/// Delete a remote-tracking branch from its remote, guarded by the last fetched commit.
pub(super) fn remote_delete_action(
    tab: &RepoTab,
    remote: &str,
    short: &str,
    b: &git::Branch,
) -> Action {
    Action::Dialog(
        Dialog::confirm(
            "Delete remote branch",
            crate::i18n::message(
                "repo-core-3492",
                &[
                    ("short", short.to_string().into()),
                    ("remote", remote.to_string().into()),
                ],
            ),
            crate::i18n::text("repo-core-1115"),
            ops(&[&[
                "push",
                &format!("--force-with-lease=refs/heads/{short}:{}", b.target),
                remote,
                "--delete",
                &format!("refs/heads/{short}"),
            ]]),
            None,
        )
        .preview_of(tab.delete_preview(&[&b.target], vec![format!("refs/remotes/{}", b.name)])),
    )
}

/// Why a local branch can't be deleted, if it can't.
pub(super) fn delete_refusal(tab: &RepoTab, b: &git::Branch) -> Option<&'static str> {
    let elsewhere = tab
        .worktrees
        .iter()
        .any(|w| !w.current && w.branch.as_deref() == Some(b.name.as_str()));
    elsewhere.then(|| crate::i18n::text("repo-core-3359"))
}
