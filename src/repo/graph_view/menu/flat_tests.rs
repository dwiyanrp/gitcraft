use std::sync::Arc;

use crate::git;
use crate::repo::pull::PullMode;
use egui_kittest::kittest::Queryable;

fn tab() -> crate::repo::RepoTab {
    let ctx = egui::Context::default();
    let mut tab = crate::repo::RepoTab::new_deferred("/nonexistent/flat-menu".into(), &ctx);
    let hash = "a".repeat(40);
    tab.refs.head = hash.clone();
    tab.refs.current = Some("dev".to_owned());
    tab.refs.local = vec![git::Branch {
        name: "dev".to_owned(),
        target: hash.clone(),
        is_head: true,
        upstream: "origin/dev".to_owned(),
        track: "[ahead 0, behind 0]".to_owned(),
    }];
    tab.refs.remotes = vec![(
        "origin".to_owned(),
        vec![git::Branch {
            name: "origin/dev".to_owned(),
            target: hash.clone(),
            is_head: false,
            upstream: String::new(),
            track: String::new(),
        }],
    )];
    tab.remotes = vec![git::Remote {
        name: "origin".to_owned(),
        url: "https://example.invalid/owner/repo.git".to_owned(),
    }];
    tab.commits = vec![
        Arc::new(git::Commit {
            hash,
            summary: "A commit".to_owned(),
            ..Default::default()
        }),
        Arc::new(git::Commit {
            hash: crate::repo::WIP_HASH.into(),
            ..Default::default()
        }),
    ];
    tab
}

#[test]
fn right_click_shows_the_flat_gitkraken_style_list() {
    let mut h = egui_kittest::Harness::builder().build_ui_state(
        |ui, (tab, acts): &mut (crate::repo::RepoTab, Vec<crate::repo::Action>)| {
            let row = ui.button("Commit row");
            ui.data_mut(|d| {
                d.insert_temp(
                    super::pill_menu_id(&"a".repeat(40)),
                    Some("refs/heads/dev".to_owned()),
                )
            });
            tab.commit_menu(&row, 0, true, acts);
        },
        (tab(), Vec::new()),
    );
    h.run();
    h.get_by_label("Commit row").click_secondary();
    h.run();

    for label in [
        "Pull (fast-forward if possible)",
        "Push",
        "Set Upstream",
        "Checkout",
        "Create worktree from",
        "Create branch here…",
        "Reset dev to this commit",
        "Edit commit message",
        "Revert commit",
        "Explain branch changes (Preview)",
        "Apply patch…",
        "Rename dev",
        "Delete dev",
        "Delete origin/dev",
        "Delete dev and origin/dev",
        "Copy branch name",
        "Copy commit SHA",
        "Copy link to branch: origin/dev",
        "Copy link to this commit on remote: origin",
        "Create patch from commit…",
        "Pin to left",
        "Solo",
        "Create tag here…",
        "Create annotated tag here…",
        "More actions",
    ] {
        h.get_by_label(label);
    }
}

#[test]
fn flat_menu_keeps_the_advanced_menu_one_level_down() {
    let mut h = egui_kittest::Harness::builder().build_ui_state(
        |ui, (tab, acts): &mut (crate::repo::RepoTab, Vec<crate::repo::Action>)| {
            let row = ui.button("Commit row");
            tab.commit_menu(&row, 0, true, acts);
        },
        (tab(), Vec::new()),
    );
    h.run();
    h.get_by_label("Commit row").click_secondary();
    h.run();
    h.get_by_label("More actions").hover();
    h.run_steps(12);
    h.get_by_label("Links / Copy ⏵");
    h.get_by_label("Compare ⏵");
}

#[test]
fn flat_pull_entry_pulls_fast_forwarding_when_possible() {
    let mut h = egui_kittest::Harness::builder().build_ui_state(
        |ui, (tab, acts): &mut (crate::repo::RepoTab, Vec<crate::repo::Action>)| {
            let row = ui.button("Commit row");
            tab.commit_menu(&row, 0, true, acts);
        },
        (tab(), Vec::new()),
    );
    h.run();
    h.get_by_label("Commit row").click_secondary();
    h.run();
    h.get_by_label("Pull (fast-forward if possible)").click();
    h.run();
    assert!(
        h.state()
            .1
            .iter()
            .any(|action| matches!(action, crate::repo::Action::PullWith(PullMode::Merge)))
    );
}

#[test]
fn flat_menu_hides_remote_delete_items_when_remote_is_on_another_commit() {
    let mut t = tab();
    let other_hash = "b".repeat(40);
    t.refs.remotes[0].1[0].target = other_hash.clone();
    t.commits.push(Arc::new(git::Commit {
        hash: other_hash,
        summary: "Earlier commit".to_owned(),
        ..Default::default()
    }));
    let mut h = egui_kittest::Harness::builder().build_ui_state(
        |ui, (tab, acts): &mut (crate::repo::RepoTab, Vec<crate::repo::Action>)| {
            let row = ui.button("Commit row");
            ui.data_mut(|d| {
                d.insert_temp(
                    super::pill_menu_id(&"a".repeat(40)),
                    Some("refs/heads/dev".to_owned()),
                )
            });
            tab.commit_menu(&row, 0, true, acts);
        },
        (t, Vec::new()),
    );
    h.run();
    h.get_by_label("Commit row").click_secondary();
    h.run();

    h.get_by_label("Rename dev");
    assert!(h.query_by_label("Delete dev").is_none());
    assert!(h.query_by_label("Delete origin/dev").is_none());
    assert!(h.query_by_label("Delete dev and origin/dev").is_none());
}

#[test]
fn flat_menu_hides_rename_when_local_branch_is_on_another_commit() {
    let mut t = tab();
    let other_hash = "b".repeat(40);
    t.refs.remotes[0].1[0].target = other_hash.clone();
    t.commits.push(Arc::new(git::Commit {
        hash: other_hash.clone(),
        summary: "Remote commit".to_owned(),
        ..Default::default()
    }));
    let remote_row_idx = t.commits.len() - 1;
    let mut h = egui_kittest::Harness::builder().build_ui_state(
        |ui, (tab, acts): &mut (crate::repo::RepoTab, Vec<crate::repo::Action>)| {
            let row = ui.button("Commit row");
            ui.data_mut(|d| {
                d.insert_temp(
                    super::pill_menu_id(&other_hash),
                    Some("refs/remotes/origin/dev".to_owned()),
                )
            });
            tab.commit_menu(&row, remote_row_idx, true, acts);
        },
        (t, Vec::new()),
    );
    h.run();
    h.get_by_label("Commit row").click_secondary();
    h.run();

    assert!(h.query_by_label("Rename dev").is_none());
    h.get_by_label("Delete origin/dev");
}

#[test]
fn flat_menu_on_commit_without_branch_shows_commit_actions() {
    let mut t = tab();
    let other_hash = "b".repeat(40);
    t.commits.push(Arc::new(git::Commit {
        hash: other_hash,
        summary: "Earlier commit".to_owned(),
        ..Default::default()
    }));
    let row_idx = t.commits.len() - 1;
    let mut h = egui_kittest::Harness::builder().build_ui_state(
        |ui, (tab, acts): &mut (crate::repo::RepoTab, Vec<crate::repo::Action>)| {
            let row = ui.button("Commit row");
            tab.commit_menu(&row, row_idx, true, acts);
        },
        (t, Vec::new()),
    );
    h.run();
    h.get_by_label("Commit row").click_secondary();
    h.run();

    for label in [
        "Checkout this commit",
        "Create worktree from this commit…",
        "Create branch here…",
        "Cherry pick commit checking…",
        "Rebase dev onto this commit checking…",
        "Reset dev to this commit ⏵",
        "Revert commit",
    ] {
        h.get_by_label(label);
    }
    for hidden in [
        "Pull (fast-forward if possible)",
        "Push",
        "Set Upstream",
        "Checkout ⏵",
        "Create worktree from ⏵",
        "Edit commit message",
        "Explain branch changes (Preview)",
        "Rename dev",
        "Delete dev",
        "Copy branch name",
    ] {
        assert!(
            h.query_by_label(hidden).is_none(),
            "unexpected item: {hidden}"
        );
    }
}
