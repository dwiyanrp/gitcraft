use super::*;
use egui_kittest::kittest::Queryable;

/// What has the keyboard when a key is pressed with the reword editor open.
#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Nothing,
    /// The terminal, drawn after the editor, which keeps Escape for the shell.
    Terminal,
    /// A plain text field drawn before the editor (the search box).
    FieldBefore,
    /// A plain text field drawn after it (the commit box).
    FieldAfter,
    /// The editor's own summary.
    Summary,
}

/// What the editor answers to Escape (or Ctrl+Enter when `save`) pressed while `focus` has
/// the keyboard.
fn press(focus: Focus, save: bool) -> Option<bool> {
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    let mut reword = Reword {
        hash: "a".repeat(40),
        subject: "Fix the login".to_owned(),
        body: "A long explanation being typed".to_owned(),
        focus: focus == Focus::Summary,
        fill: false,
    };
    let terminal_id = egui::Id::new("terminal");
    let (key, modifiers) = if save {
        (Key::Enter, egui::Modifiers::COMMAND)
    } else {
        (Key::Escape, egui::Modifiers::NONE)
    };
    let event = egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    };
    let mut other = "something typed".to_owned();
    let mut out = None;
    for (pass, events) in [
        Vec::new(),
        Vec::new(),
        vec![egui::Event::ModifiersChanged(modifiers), event],
    ]
    .into_iter()
    .enumerate()
    {
        let input = egui::RawInput {
            events,
            ..Default::default()
        };
        ctx.run_ui(input, |ui| {
            let mut field = |ui: &mut Ui| {
                let r = ui.text_edit_singleline(&mut other);
                if pass == 0 {
                    r.request_focus();
                }
            };
            if focus == Focus::FieldBefore {
                field(ui);
            }
            out = reword_editor(ui, &mut reword);
            if focus == Focus::FieldAfter {
                field(ui);
            }
            let (_, rect) = ui.allocate_space(vec2(200.0, 100.0));
            let terminal = ui.interact(rect, terminal_id, Sense::click());
            if focus == Focus::Terminal {
                terminal.request_focus();
                let keep = egui::EventFilter {
                    escape: true,
                    ..Default::default()
                };
                ui.memory_mut(|m| m.set_focus_lock_filter(terminal_id, keep));
            }
        })
        .drop_without_applying_deltas();
    }
    out
}

/// Escape in the terminal (vim, the shell) or in another field is theirs: the message being
/// reworded isn't thrown away. With nothing else holding the keyboard it cancels.
#[test]
fn escape_elsewhere_does_not_cancel_a_reword() {
    assert_eq!(press(Focus::Terminal, false), None);
    // egui takes the focus from a text field for an Escape before the editor is drawn.
    assert_eq!(press(Focus::FieldBefore, false), None);
    assert_eq!(press(Focus::FieldAfter, false), None);
    assert_eq!(press(Focus::Nothing, false), Some(false));
    assert_eq!(press(Focus::Summary, false), Some(false));
}

/// Nor does Ctrl+Enter there save it.
#[test]
fn ctrl_enter_elsewhere_does_not_save_a_reword() {
    assert_eq!(press(Focus::Terminal, true), None);
    assert_eq!(press(Focus::FieldBefore, true), None);
    assert_eq!(press(Focus::FieldAfter, true), None);
    assert_eq!(press(Focus::Nothing, true), Some(true));
    assert_eq!(press(Focus::Summary, true), Some(true));
}

/// Clicking an editable commit message box enters reword edit mode.
#[test]
fn clicking_commit_message_enters_edit_mode() {
    let detail = crate::git::CommitDetail {
        hash: "a".repeat(40),
        parents: vec!["b".repeat(40)],
        author: "Author".into(),
        email: "author@example.invalid".into(),
        author_date: "2026-11-26 10:00:00 +0000".into(),
        committer: "Author".into(),
        commit_date: "2026-11-26 10:00:00 +0000".into(),
        message: "fix: initial title\n\nBody details".into(),
        files: Vec::new(),
        in_head: true,
        merges_after: false,
    };
    let mut styled = false;
    let mut h = egui_kittest::Harness::builder()
        .with_size([400.0, 400.0])
        .build_ui_state(
            move |ui, (reword, acts): &mut (Option<Reword>, Vec<Action>)| {
                if !styled {
                    crate::theme::install_fonts(ui.ctx());
                    crate::theme::apply(
                        ui.ctx(),
                        &crate::theme::ThemeId::default(),
                        &crate::theme::Tweaks::default(),
                    );
                    styled = true;
                    return;
                }
                message_section(ui, acts, &detail, reword, true, 300.0);
            },
            (None, Vec::new()),
        );
    for _ in 0..3 {
        h.step();
    }
    h.get_by_label_contains("Body details").click();
    h.step();
    let reword = h.state().0.as_ref().expect("click should enter edit mode");
    assert_eq!(reword.subject, "fix: initial title");
    assert_eq!(reword.body, "Body details");
}

#[test]
fn commit_files_support_path_tree_all_files_and_collapse_expand() {
    let ctx = egui::Context::default();
    let mut tab = RepoTab::new_deferred("/224/commit-files-ui".into(), &ctx);
    let hash = "a".repeat(40);
    tab.detail_hash = Some(hash.clone());
    tab.detail = Some(crate::git::CommitDetail {
        hash: hash.clone(),
        parents: vec!["b".repeat(40)],
        author: "Author".into(),
        email: "author@example.invalid".into(),
        author_date: "2026-11-26 10:00:00 +0000".into(),
        committer: "Author".into(),
        commit_date: "2026-11-26 10:00:00 +0000".into(),
        message: "test commit".into(),
        files: vec![crate::git::ChangedFile {
            path: "src/changed.rs".into(),
            old_path: None,
            status: 'M',
            added: Some(1),
            removed: Some(0),
        }],
        in_head: true,
        merges_after: false,
    });
    tab.all_files.of = Some(hash);
    tab.all_files.files = Some(Ok(vec![
        "src/changed.rs".into(),
        "docs/unchanged.md".into(),
    ]));

    let mut styled = false;
    let mut h = egui_kittest::Harness::builder()
        .with_size([400.0, 800.0])
        .build_ui_state(
            move |ui, (tab, acts): &mut (RepoTab, Vec<Action>)| {
                if !styled {
                    crate::theme::install_fonts(ui.ctx());
                    crate::theme::apply(
                        ui.ctx(),
                        &crate::theme::ThemeId::default(),
                        &crate::theme::Tweaks::default(),
                    );
                    egui_extras::install_image_loaders(ui.ctx());
                    styled = true;
                    return;
                }
                egui::CentralPanel::default().show(ui, |ui| tab.commit_panel(ui, acts));
            },
            (tab, Vec::new()),
        );
    for _ in 0..4 {
        h.step();
    }

    // 1. Default Tree view for changed files: folder shown + Collapse all button.
    assert!(h.query_by_label_contains("folder src").is_some());
    assert!(h.query_by_label_contains("changed.rs").is_some());
    h.get_by_label("Collapse all").click();
    h.step();
    assert!(h.query_by_label_contains("changed.rs").is_none());
    h.get_by_label("Expand all").click();
    h.step();
    assert!(h.query_by_label_contains("changed.rs").is_some());

    // 2. Switch to Path view for changed files: flat list, no folder node, no Collapse/Expand.
    h.get_by_label("Path").click();
    h.step();
    h.step();
    assert!(h.query_by_label_contains("folder src").is_none());
    assert!(h.query_by_label("Collapse all").is_none());
    assert!(h.query_by_label("Expand all").is_none());
    assert!(h.query_by_label_contains("src/changed.rs").is_some());

    // 3. View all files always renders in Tree view with Expand all / Collapse all.
    h.state_mut().0.all_files.on = true;
    h.step();
    h.step();
    assert!(h.query_by_label_contains("folder docs").is_some());
    assert!(h.query_by_label_contains("unchanged.md").is_none());
    h.get_by_label("Expand all").click();
    h.step();
    h.step();
    assert!(h.query_by_label_contains("unchanged.md").is_some());
    h.get_by_label("Collapse all").click();
    h.step();
    h.step();
    assert!(h.query_by_label_contains("unchanged.md").is_none());
}
