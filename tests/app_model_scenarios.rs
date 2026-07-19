mod support;

use kimini::model::{AppModel, ApplyOutcome};
use kimini::protocol::{MessagePage, Page, SessionSnapshot, SessionStatus, WireEvent, Workspace};
use support::{event, snapshot};

#[test]
fn snapshot_then_stream_builds_a_single_conversation() {
    let snapshot: SessionSnapshot = snapshot(7, "hello", "partial");
    let mut model = AppModel::default();
    model.seed(snapshot);

    let delta: WireEvent = event(
        "assistant.delta",
        7,
        true,
        Some(7),
        serde_json::json!({ "delta": " answer" }),
    );
    assert_eq!(model.apply(delta), ApplyOutcome::Applied);

    let conversation = model.active_conversation().unwrap();
    assert_eq!(conversation.messages[0].plain_text(), "hello");
    assert_eq!(
        conversation.assistant_stream.as_deref(),
        Some("partial answer")
    );
    assert_eq!(conversation.cursor.seq, 7);
}

#[test]
fn snapshots_update_the_single_session_catalog_without_duplicates() {
    let initial = snapshot(7, "hello", "");
    let mut updated = initial.clone();
    updated.session.title = "Updated title".into();
    let mut model = AppModel::default();

    model.replace_sessions(vec![initial.session]);
    model.seed(updated);

    assert_eq!(model.sessions().len(), 1);
    assert_eq!(model.sessions()[0].title, "Updated title");
}

#[test]
fn workspace_catalog_updates_without_losing_order() {
    let workspace = Workspace {
        id: "wd_project_0123456789ab".into(),
        root: "/tmp/project".into(),
        name: "Project".into(),
        created_at: "2026-07-19T00:00:00Z".into(),
        last_opened_at: "2026-07-19T01:00:00Z".into(),
        session_count: 1,
    };
    let mut model = AppModel::default();
    model.replace_workspaces(vec![workspace.clone()]);

    let mut renamed = workspace;
    renamed.name = "Renamed".into();
    model.upsert_workspace(renamed);
    assert_eq!(model.workspaces()[0].name, "Renamed");

    model.remove_workspace("wd_project_0123456789ab");
    assert!(model.workspaces().is_empty());
}

#[test]
fn removing_the_active_workspace_clears_its_session_selection() {
    let mut model = AppModel::default();
    model.replace_workspaces(vec![Workspace {
        id: "ws_01".into(),
        root: "/tmp/project".into(),
        name: "Project".into(),
        created_at: "2026-07-19T00:00:00Z".into(),
        last_opened_at: "2026-07-19T01:00:00Z".into(),
        session_count: 1,
    }]);
    model.seed(snapshot(7, "hello", ""));

    assert!(model.remove_workspace("ws_01"));
    assert!(model.active_session().is_none());
}

#[test]
fn removing_the_active_session_selects_the_next_available_session() {
    let first = snapshot(7, "hello", "");
    let mut second = first.session.clone();
    second.id = "sess_02".into();
    let mut model = AppModel::default();
    model.replace_sessions(vec![first.session.clone(), second]);
    model.seed(first);

    let selected = model.remove_session("sess_01");

    assert_eq!(selected.as_deref(), Some("sess_02"));
    assert_eq!(model.sessions().len(), 1);
}

#[test]
fn session_pages_append_in_order_without_duplicates() {
    let first = snapshot(7, "hello", "");
    let mut second = first.session.clone();
    second.id = "sess_02".into();
    let mut third = second.clone();
    third.id = "sess_03".into();
    let mut model = AppModel::default();
    model.replace_session_page(Page {
        items: vec![first.session, second.clone()],
        has_more: true,
    });

    let added = model.append_session_page(Page {
        items: vec![second, third],
        has_more: false,
    });

    assert_eq!(added, 1);
    assert_eq!(
        model
            .sessions()
            .iter()
            .map(|session| session.id.as_str())
            .collect::<Vec<_>>(),
        ["sess_01", "sess_02", "sess_03"]
    );
    assert!(!model.has_more_sessions());
}

#[test]
fn empty_overlapping_session_pages_stop_repeated_paging() {
    let first = snapshot(7, "hello", "").session;
    let mut model = AppModel::default();
    model.replace_session_page(Page {
        items: vec![first.clone()],
        has_more: true,
    });

    assert_eq!(
        model.append_session_page(Page {
            items: vec![first],
            has_more: true,
        }),
        0
    );
    assert!(!model.has_more_sessions());
}

#[test]
fn archived_pages_are_isolated_from_the_active_catalog() {
    let active = snapshot(7, "hello", "").session;
    let mut archived = active.clone();
    archived.id = "archived_01".into();
    archived.archived = true;
    let mut model = AppModel::default();
    model.replace_sessions(vec![active]);

    model.replace_archived_session_page(Page {
        items: vec![archived.clone()],
        has_more: true,
    });
    assert!(model.archived_sessions_loaded());
    assert!(model.has_more_archived_sessions());
    assert_eq!(model.sessions().len(), 1);
    assert_eq!(model.archived_sessions(), &[archived]);

    model.remove_archived_session("archived_01");
    assert!(model.archived_sessions().is_empty());
}

#[test]
fn archived_pagination_deduplicates_and_invalidation_forces_a_fresh_catalog() {
    let mut first = snapshot(7, "hello", "").session;
    first.id = "archived_01".into();
    first.archived = true;
    let mut second = first.clone();
    second.id = "archived_02".into();
    let mut model = AppModel::default();
    model.replace_archived_session_page(Page {
        items: vec![first.clone()],
        has_more: true,
    });

    assert_eq!(
        model.append_archived_session_page(Page {
            items: vec![first, second],
            has_more: true,
        }),
        1
    );
    assert_eq!(model.archived_sessions().len(), 2);
    assert!(model.archived_sessions_loaded());
    assert!(model.has_more_archived_sessions());

    model.invalidate_archived_sessions();
    assert!(model.archived_sessions().is_empty());
    assert!(!model.archived_sessions_loaded());
    assert!(!model.has_more_archived_sessions());
}

#[test]
fn runtime_status_is_attached_only_to_its_seeded_conversation() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", ""));
    let runtime = SessionStatus {
        busy: false,
        model: Some("kimi-k2".into()),
        thinking_level: "high".into(),
        permission: "manual".into(),
        plan_mode: false,
        swarm_mode: true,
        context_tokens: 42,
        max_context_tokens: 100,
        context_usage: 0.42,
    };

    assert!(model.active_runtime().is_none());
    model.set_runtime("missing", runtime.clone());
    assert!(model.active_runtime().is_none());
    model.set_runtime("sess_01", runtime);
    assert_eq!(model.active_runtime().unwrap().context_tokens, 42);
}

#[test]
fn goal_snapshots_recover_and_live_completion_clears_the_strip() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", ""));
    let goal = serde_json::from_value(serde_json::json!({
        "goalId": "goal_01",
        "objective": "Ship native GUI",
        "status": "active",
        "turnsUsed": 1,
        "tokensUsed": 42,
        "wallClockMs": 1000,
        "budget": {
            "tokenBudget": null,
            "turnBudget": null,
            "wallClockBudgetMs": null,
            "remainingTokens": null,
            "remainingTurns": null,
            "remainingWallClockMs": null
        }
    }))
    .unwrap();
    model.set_goal("sess_01", Some(goal));
    assert_eq!(model.active_goal().unwrap().objective, "Ship native GUI");

    let completion = event(
        "goal.updated",
        8,
        false,
        None,
        serde_json::json!({
            "snapshot": {
                "goalId": "goal_01",
                "objective": "Ship native GUI",
                "status": "complete",
                "turnsUsed": 2,
                "tokensUsed": 84,
                "wallClockMs": 2000,
                "budget": {
                    "tokenBudget": null,
                    "turnBudget": null,
                    "wallClockBudgetMs": null,
                    "remainingTokens": null,
                    "remainingTurns": null,
                    "remainingWallClockMs": null
                }
            }
        }),
    );
    assert_eq!(model.apply(completion), ApplyOutcome::Applied);
    assert!(model.active_goal().is_none());
}

#[test]
fn history_for_an_unseeded_session_is_rejected_without_creating_state() {
    let mut model = AppModel::default();

    assert!(!model.prepend_messages(
        "missing",
        MessagePage {
            items: Vec::new(),
            has_more: false,
        },
    ));
    assert!(model.active_conversation().is_none());
}

#[test]
fn older_message_pages_are_reversed_and_deduplicated_before_prepend() {
    let mut initial = snapshot(7, "latest", "");
    initial.messages.has_more = true;
    let latest = initial.messages.items[0].clone();
    let mut middle = latest.clone();
    middle.id = "msg_middle".into();
    middle.content = serde_json::from_value(serde_json::json!([
        { "type": "text", "text": "middle" }
    ]))
    .unwrap();
    let mut oldest = middle.clone();
    oldest.id = "msg_oldest".into();
    oldest.content = serde_json::from_value(serde_json::json!([
        { "type": "text", "text": "oldest" }
    ]))
    .unwrap();
    let mut model = AppModel::default();
    model.seed(initial);

    assert!(model.prepend_messages(
        "sess_01",
        MessagePage {
            items: vec![middle, oldest, latest],
            has_more: false,
        },
    ));

    let conversation = model.active_conversation().unwrap();
    assert_eq!(conversation.messages.len(), 3);
    assert_eq!(conversation.messages[0].plain_text(), "oldest");
    assert_eq!(conversation.messages[1].plain_text(), "middle");
    assert_eq!(conversation.messages[2].plain_text(), "latest");
    assert!(!conversation.has_more_messages);
}

#[test]
fn duplicate_durable_events_are_ignored_and_gaps_request_a_resync() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", ""));

    let duplicate = event(
        "event.session.work_changed",
        7,
        false,
        None,
        serde_json::json!({ "busy": false }),
    );
    assert_eq!(model.apply(duplicate), ApplyOutcome::Duplicate);

    let gap = event(
        "event.session.work_changed",
        9,
        false,
        None,
        serde_json::json!({ "busy": false }),
    );
    assert_eq!(model.apply(gap), ApplyOutcome::ResyncRequired);
    assert!(model.active_session().unwrap().busy);
}

#[test]
fn durable_interaction_events_update_pending_cards_and_cursor() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", ""));

    let requested = event(
        "event.approval.requested",
        8,
        false,
        None,
        serde_json::json!({
          "approval_id": "approval_01", "session_id": "sess_01",
          "tool_call_id": "tool_01", "tool_name": "Bash",
          "action": "run command", "expires_at": "2026-07-18T09:00:00.000Z",
          "created_at": "2026-07-18T08:00:00.000Z"
        }),
    );
    assert_eq!(model.apply(requested), ApplyOutcome::Applied);
    assert_eq!(model.active_conversation().unwrap().approvals.len(), 1);

    let resolved = event(
        "event.approval.resolved",
        9,
        false,
        None,
        serde_json::json!({ "approval_id": "approval_01" }),
    );
    assert_eq!(model.apply(resolved), ApplyOutcome::Applied);
    let conversation = model.active_conversation().unwrap();
    assert!(conversation.approvals.is_empty());
    assert_eq!(conversation.cursor.seq, 9);
}

#[test]
fn message_echoes_replace_by_id_and_future_events_still_advance_the_cursor() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", "stale stream"));

    for (seq, text) in [(8, "first"), (9, "corrected")] {
        assert_eq!(
            model.apply(event(
                "event.message.created",
                seq,
                false,
                None,
                serde_json::json!({ "message": {
                  "id": "msg_02", "session_id": "sess_01", "role": "assistant",
                  "content": [{ "type": "text", "text": text }],
                  "created_at": "2026-07-18T08:02:00.000Z"
                } }),
            )),
            ApplyOutcome::Applied
        );
    }
    assert_eq!(model.active_conversation().unwrap().messages.len(), 2);
    assert_eq!(
        model.active_conversation().unwrap().messages[1].plain_text(),
        "corrected"
    );
    assert!(
        model
            .active_conversation()
            .unwrap()
            .assistant_stream
            .is_none()
    );

    assert_eq!(
        model.apply(event(
            "event.future",
            10,
            false,
            None,
            serde_json::json!({})
        )),
        ApplyOutcome::Applied
    );
    assert_eq!(model.active_conversation().unwrap().cursor.seq, 10);
}

#[test]
fn stream_offsets_skip_duplicates_and_detect_missing_text() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", "abc"));

    let duplicate = event(
        "assistant.delta",
        7,
        true,
        Some(1),
        serde_json::json!({ "delta": "bc" }),
    );
    assert_eq!(model.apply(duplicate), ApplyOutcome::Applied);
    assert_eq!(
        model
            .active_conversation()
            .unwrap()
            .assistant_stream
            .as_deref(),
        Some("abc")
    );

    let gap = event(
        "assistant.delta",
        7,
        true,
        Some(5),
        serde_json::json!({ "delta": "later" }),
    );
    assert_eq!(model.apply(gap), ApplyOutcome::ResyncRequired);
}

#[test]
fn stream_offsets_follow_the_daemons_utf16_units() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", "😀"));

    let next = event(
        "assistant.delta",
        7,
        true,
        Some(2),
        serde_json::json!({ "delta": "!" }),
    );
    assert_eq!(model.apply(next), ApplyOutcome::Applied);
    assert_eq!(
        model
            .active_conversation()
            .unwrap()
            .assistant_stream
            .as_deref(),
        Some("😀!")
    );
}

#[test]
fn a_new_turn_step_resets_step_relative_streams() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", "first step"));

    assert_eq!(
        model.apply(event(
            "turn.step.started",
            8,
            false,
            None,
            serde_json::json!({ "agentId": "main", "turnId": 1, "step": 2 }),
        )),
        ApplyOutcome::Applied
    );
    assert_eq!(
        model.apply(event(
            "assistant.delta",
            8,
            true,
            Some(0),
            serde_json::json!({ "agentId": "main", "delta": "second step" }),
        )),
        ApplyOutcome::Applied
    );
    assert_eq!(
        model
            .active_conversation()
            .unwrap()
            .assistant_stream
            .as_deref(),
        Some("second step")
    );
}

#[test]
fn subagent_streams_do_not_enter_the_main_conversation() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", "main"));

    assert_eq!(
        model.apply(event(
            "assistant.delta",
            7,
            true,
            Some(4),
            serde_json::json!({ "agentId": "agent_01", "delta": " side output" }),
        )),
        ApplyOutcome::Applied
    );
    assert_eq!(
        model
            .active_conversation()
            .unwrap()
            .assistant_stream
            .as_deref(),
        Some("main")
    );
}

#[test]
fn thinking_and_question_lifecycles_are_projected() {
    let mut model = AppModel::default();
    model.seed(snapshot(7, "hello", ""));

    assert_eq!(
        model.apply(event(
            "thinking.delta",
            7,
            true,
            Some(0),
            serde_json::json!({ "delta": "considering" }),
        )),
        ApplyOutcome::Applied
    );
    assert_eq!(
        model
            .active_conversation()
            .unwrap()
            .thinking_stream
            .as_deref(),
        Some("considering")
    );

    let request = serde_json::json!({
      "question_id": "question_01", "session_id": "sess_01",
      "questions": [{ "id": "choice", "question": "Continue?", "options": [] }],
      "created_at": "2026-07-18T08:00:00.000Z"
    });
    assert_eq!(
        model.apply(event(
            "event.question.requested",
            8,
            false,
            None,
            request.clone()
        )),
        ApplyOutcome::Applied
    );
    assert_eq!(
        model.apply(event("event.question.requested", 9, false, None, request)),
        ApplyOutcome::Applied
    );
    assert_eq!(model.active_conversation().unwrap().questions.len(), 1);
    assert_eq!(
        model.apply(event(
            "event.question.dismissed",
            10,
            false,
            None,
            serde_json::json!({ "question_id": "question_01" }),
        )),
        ApplyOutcome::Applied
    );
    assert!(model.active_conversation().unwrap().questions.is_empty());
}

#[test]
fn events_without_an_active_subscription_are_ignored() {
    let mut model = AppModel::default();
    let mut global = event(
        "event.config.changed",
        1,
        false,
        None,
        serde_json::json!({}),
    );
    global.session_id = None;
    assert_eq!(model.apply(global), ApplyOutcome::Irrelevant);

    assert_eq!(
        model.apply(event(
            "event.message.created",
            1,
            false,
            None,
            serde_json::json!({})
        )),
        ApplyOutcome::Irrelevant
    );
}

#[test]
fn global_work_events_update_background_sessions_without_resyncing_the_active_one() {
    let active = snapshot(7, "hello", "");
    let mut background = active.session.clone();
    background.id = "sess_02".into();
    background.title = "Background".into();
    background.busy = true;
    let mut model = AppModel::default();
    model.replace_sessions(vec![active.session.clone(), background]);
    model.seed(active);

    let mut work_changed = event(
        "event.session.work_changed",
        3,
        false,
        None,
        serde_json::json!({ "busy": false, "main_turn_active": false }),
    );
    work_changed.session_id = Some("sess_02".into());

    assert_eq!(model.apply(work_changed), ApplyOutcome::Applied);
    assert!(!model.sessions()[1].busy);
    assert_eq!(model.active_conversation().unwrap().cursor.seq, 7);
}
