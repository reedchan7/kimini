use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use kimini::api::{EventSocket, KimiClient, SocketEvent};
use kimini::daemon::discover_connection;
use kimini::protocol::{MessageRole, PromptOptions};

const MUTATING_LIVE_TEST_ENV: &str = "KIMINI_RUN_MUTATING_LIVE_TEST";

struct ArchivedSession {
    client: KimiClient,
    session_id: String,
    model: String,
}

impl ArchivedSession {
    fn create(client: KimiClient) -> Self {
        let auth_model = client
            .auth_summary()
            .expect("read live-test auth summary")
            .default_model;
        let models = client.list_models().expect("read live-test model catalog");
        let model = auth_model
            .filter(|model| !model.trim().is_empty())
            .or_else(|| models.items.first().map(|model| model.model.clone()))
            .expect("configured model");
        let session = client
            .create_session(env!("CARGO_MANIFEST_DIR"), Some(&model))
            .expect("create isolated live-test session");
        Self {
            client,
            session_id: session.id,
            model,
        }
    }
}

impl Drop for ArchivedSession {
    fn drop(&mut self) {
        if self
            .client
            .session_status(&self.session_id)
            .is_ok_and(|status| status.busy)
        {
            let _ = self.client.abort_session(&self.session_id);
        }
        let _ = self.client.archive_session(&self.session_id);
    }
}

#[test]
#[ignore = "requires the locally installed Kimi Code daemon"]
fn loads_sessions_and_an_atomic_snapshot_from_the_real_daemon() {
    let stop = AtomicBool::new(false);
    let connection = discover_connection(&stop, &|_| {}).expect("local Kimi Code daemon");
    let client = KimiClient::new(connection.clone());
    let auth = client.auth_summary().expect("auth summary");
    let models = client.list_models().expect("model catalog");
    let sessions = client.list_sessions().expect("session list");

    if auth.ready {
        assert!(auth.providers_count > 0);
        assert!(!models.items.is_empty());
    }

    if let Some(session) = sessions.items.first() {
        let snapshot = client.snapshot(&session.id).expect("session snapshot");
        let status = client.session_status(&session.id).expect("session status");
        let prompts = client.list_prompts(&session.id).expect("prompt queue");
        let tasks = client.list_tasks(&session.id).expect("task list");
        let skills = client.list_skills(&session.id).expect("skill catalog");
        let goal = client.session_goal(&session.id).expect("session goal");
        assert_eq!(snapshot.session.id, session.id);
        assert!(!snapshot.epoch.is_empty());
        assert_eq!(snapshot.cursor().seq, snapshot.as_of_seq);
        assert!(status.context_usage <= 1.0);
        assert!(
            prompts
                .queued
                .iter()
                .all(|prompt| !prompt.prompt_id.is_empty())
        );
        assert!(tasks.items.iter().all(|task| task.session_id == session.id));
        assert!(skills.skills.iter().all(|skill| !skill.name.is_empty()));
        if let Some(goal) = goal {
            assert!(!goal.goal_id.is_empty());
            assert!(!goal.objective.is_empty());
            assert!(!goal.is_complete());
        }

        let root = client
            .list_files(&session.id, ".")
            .expect("workspace root listing");
        assert!(root.items.len() <= 500);
        let file = root
            .items
            .iter()
            .find(|entry| !entry.is_directory())
            .cloned()
            .or_else(|| {
                let directory = root.items.iter().find(|entry| entry.is_directory())?;
                client
                    .list_files(&session.id, &directory.path)
                    .ok()?
                    .items
                    .into_iter()
                    .find(|entry| !entry.is_directory())
            });
        if let Some(file) = file {
            let preview = client
                .read_workspace_file(&session.id, &file.path)
                .expect("workspace file preview");
            assert_eq!(preview.path, file.path);
            assert!(preview.size >= preview.content.len() as u64 || preview.encoding == "base64");
        }
        if let Ok(git) = client.workspace_git_status(&session.id) {
            assert!(git.entries.keys().all(|path| !path.is_empty()));
        }

        let socket = EventSocket::connect(
            connection,
            "kimini-live-test",
            [(session.id.clone(), snapshot.cursor())].into(),
        );
        let events = socket.events();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match events.try_recv() {
                Ok(SocketEvent::Connected) => break,
                Ok(SocketEvent::Error { message, .. }) => {
                    panic!("WebSocket connection: {message}")
                }
                Ok(_) | Err(async_channel::TryRecvError::Empty) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                }
                Ok(_) | Err(async_channel::TryRecvError::Empty) => {
                    panic!("WebSocket handshake timed out")
                }
                Err(async_channel::TryRecvError::Closed) => {
                    panic!("WebSocket closed during handshake")
                }
            }
        }
    }
}

#[test]
#[ignore = "creates an isolated daemon session; set KIMINI_RUN_MUTATING_LIVE_TEST=1"]
fn completes_a_real_prompt_round_trip_and_archives_the_fixture() {
    assert_eq!(
        std::env::var(MUTATING_LIVE_TEST_ENV).as_deref(),
        Ok("1"),
        "set {MUTATING_LIVE_TEST_ENV}=1 to acknowledge the live prompt"
    );

    let stop = AtomicBool::new(false);
    let connection = discover_connection(&stop, &|_| {}).expect("local Kimi Code daemon");
    let fixture = ArchivedSession::create(KimiClient::new(connection));
    let status = fixture
        .client
        .session_status(&fixture.session_id)
        .expect("read live-test runtime");
    let thinking = if status.thinking_level.trim().is_empty() {
        fixture
            .client
            .list_models()
            .expect("read live-test model effort")
            .items
            .into_iter()
            .find(|model| model.model == fixture.model)
            .and_then(|model| model.default_effort)
            .unwrap_or_else(|| "off".into())
    } else {
        status.thinking_level.clone()
    };
    let options = PromptOptions {
        agent_id: None,
        model: Some(fixture.model.clone()),
        thinking: Some(thinking),
        permission_mode: Some(status.permission),
        plan_mode: Some(status.plan_mode),
        swarm_mode: Some(status.swarm_mode),
    };
    let content = [kimini::protocol::PromptPart::text(
        "Reply with exactly KIMINI_E2E_OK. Do not call tools.",
    )];
    let result = fixture
        .client
        .submit_prompt_with_options(&fixture.session_id, &content, &options)
        .expect("submit live prompt");
    assert!(!result.prompt_id.is_empty());
    let deadline = Instant::now() + Duration::from_secs(90);

    loop {
        let snapshot = fixture
            .client
            .snapshot(&fixture.session_id)
            .expect("poll live prompt snapshot");
        let reply = snapshot
            .messages
            .items
            .iter()
            .find(|message| message.role == MessageRole::Assistant);
        if reply.is_some_and(|message| message.plain_text().contains("KIMINI_E2E_OK"))
            && fixture
                .client
                .session_status(&fixture.session_id)
                .is_ok_and(|status| !status.busy)
        {
            break;
        }
        assert!(
            snapshot.pending_approvals.is_empty() && snapshot.pending_questions.is_empty(),
            "the no-tool fixture unexpectedly requested interaction"
        );
        assert_ne!(
            snapshot.session.last_turn_reason.as_deref(),
            Some("failed"),
            "live prompt failed before producing an assistant reply"
        );
        assert!(Instant::now() < deadline, "live prompt timed out");
        std::thread::sleep(Duration::from_millis(100));
    }
}
