#[test]
fn hovering_a_commit_row_keeps_message_tooltip_hidden() -> Result<(), String> {
    let path = std::env::temp_dir().join(format!("gitcraft-graph-message-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    let git = |args: &[&str]| git::run(Some(&path), args);
    git(&["init", "-q"])?;
    git(&["config", "user.name", "Graph test"])?;
    git(&["config", "user.email", "graph@example.invalid"])?;
    git(&["config", "commit.gpgSign", "false"])?;
    git(&["commit", "--allow-empty", "-qm", "Base"])?;
    let other = "Previous subject\n\nThe previous commit has its own full body.";
    git(&["commit", "--allow-empty", "-qm", other])?;
    let message = format!(
        "{}\n\n{}\n\nFinal paragraph: TAIL THAT MUST BE VISIBLE.",
        "Long Unicode café subject that will be elided in the graph ".repeat(3),
        "A detailed explanation with spacing and Unicode café. ".repeat(20)
    );
    let message_path = path.join("message.txt");
    std::fs::write(&message_path, &message).map_err(|e| e.to_string())?;
    git(&[
        "commit",
        "--allow-empty",
        "-q",
        "--cleanup=verbatim",
        "-F",
        &message_path.to_string_lossy(),
    ])?;
    // Use production Git and parse_record, rather than injecting a synthetic Commit.body.
    let commits = git::log(&path, None, Default::default(), 0, 10)?;
    assert_eq!(commits[0].body.chars().count(), 400);
    assert!(!commits[0].body.contains("TAIL THAT MUST BE VISIBLE"));
    let first = commits[0].hash.clone();
    let second = commits[1].hash.clone();
    let ctx = egui::Context::default();
    theme::install_fonts(&ctx);
    let mut tab = RepoTab::new_deferred(path.clone(), &ctx);
    tab.commits = commits.into_iter().map(Arc::new).collect();
    tab.graph = crate::graph::layout(&tab.commits);
    tab.rebuild_graph_labels();
    tab.selected = Some(2);
    let size = vec2(900.0, 220.0);
    frame(&ctx, &mut tab, size, vec![], 0.0);
    let (shapes, _) = frame(&ctx, &mut tab, size, vec![], 0.1);
    let first_point = painted_text_center(&shapes, &tab.commits[0].summary);
    let second_point = painted_text_center(&shapes, &tab.commits[1].summary);
    for (point, time) in [(first_point, 0.2), (first_point, 1.0), (second_point, 2.0)] {
        let (shapes, _) = frame(&ctx, &mut tab, size, vec![Event::PointerMoved(point)], time);
        assert!(
            !texts(&shapes).iter().any(|s| {
                s.galley.job.text == message
                    || s.galley.job.text == other
                    || s.galley.text().contains("Loading full commit message")
            }),
            "hovering a commit row must not show a message tooltip"
        );
    }
    assert!(tab.graph_messages.message(&path, &first).is_none());
    assert!(tab.graph_messages.message(&path, &second).is_none());
    assert_eq!(
        tab.selected,
        Some(2),
        "hover must not change the selected commit"
    );
    frame(
        &ctx,
        &mut tab,
        size,
        vec![Event::PointerMoved(first_point)],
        3.6,
    );
    frame(
        &ctx,
        &mut tab,
        size,
        vec![Event::PointerButton {
            pos: first_point,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
        3.61,
    );
    let (_, acts) = frame(
        &ctx,
        &mut tab,
        size,
        vec![Event::PointerButton {
            pos: first_point,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
        3.62,
    );
    assert!(acts.iter().any(|a| matches!(a, Action::Select(0))));
    drop(tab);
    assert!(git::commit_message(&path, "--output=written").is_err());
    std::fs::write(
        &message_path,
        "x".repeat(git::MAX_TOOLTIP_MESSAGE_BYTES + 1),
    )
    .map_err(|e| e.to_string())?;
    git(&[
        "commit",
        "--allow-empty",
        "-q",
        "--cleanup=verbatim",
        "-F",
        &message_path.to_string_lossy(),
    ])?;
    let oversized = git(&["rev-parse", "HEAD"])?;
    assert!(
        git::commit_message(&path, oversized.trim()).is_err_and(|error| error.contains("256 KiB")),
        "oversized objects must be refused before retaining full message text"
    );
    let _ = std::fs::remove_dir_all(&path);
    Ok(())
}
