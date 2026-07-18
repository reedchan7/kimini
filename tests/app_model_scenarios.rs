mod support;

use kimini::model::{AppModel, ApplyOutcome};
use kimini::protocol::{SessionSnapshot, WireEvent};
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
