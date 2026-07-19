use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use kimini::api::KimiClient;
use kimini::daemon::Connection;

#[test]
fn unwraps_the_daemon_envelope_and_sends_native_identity() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[],"has_more":false},"request_id":"req_01"}"#,
    );
    let client = KimiClient::new(Connection::new(origin, Some("secret".into())));

    let page = client.list_sessions().unwrap();
    let request = request.join().unwrap();

    assert!(page.items.is_empty());
    assert!(request.starts_with("GET /api/v1/sessions?page_size=100&include_archive=false"));
    assert!(request.contains("Authorization: Bearer secret"));
    assert!(request.contains("X-Kimi-Client-Ui-Mode: native"));
}

#[test]
fn pages_sessions_from_the_oldest_loaded_session() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[],"has_more":false},"request_id":"req_01"}"#,
    );
    let client = KimiClient::new(Connection::new(origin, None));

    let page = client.list_sessions_before("sess old/1").unwrap();
    let request = request.join().unwrap();

    assert!(page.items.is_empty());
    assert!(request.starts_with(
        "GET /api/v1/sessions?before_id=sess%20old%2F1&page_size=100&include_archive=false"
    ));
}

#[test]
fn drains_every_remaining_session_page_for_global_search() {
    let first = serde_json::json!({
        "code": 0,
        "msg": "",
        "data": {
            "items": [session_json("older 1")],
            "has_more": true
        },
        "request_id": "req_01"
    })
    .to_string();
    let second = serde_json::json!({
        "code": 0,
        "msg": "",
        "data": {
            "items": [session_json("older/2")],
            "has_more": false
        },
        "request_id": "req_02"
    })
    .to_string();
    let (origin, requests) = many_responses(vec![first, second]);
    let client = KimiClient::new(Connection::new(origin, None));

    let pages = client.list_session_pages_before("loaded oldest").unwrap();
    let requests = requests.join().unwrap();

    assert_eq!(pages.len(), 2);
    assert_eq!(pages[0].items[0].id, "older 1");
    assert_eq!(pages[1].items[0].id, "older/2");
    assert!(requests[0].starts_with(
        "GET /api/v1/sessions?before_id=loaded%20oldest&page_size=100&include_archive=false"
    ));
    assert!(requests[1].starts_with(
        "GET /api/v1/sessions?before_id=older%201&page_size=100&include_archive=false"
    ));
}

#[test]
fn lists_archived_sessions_with_an_independent_cursor() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[],"has_more":false},"request_id":"req_01"}"#,
    );
    KimiClient::new(Connection::new(origin, None))
        .list_archived_sessions()
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("GET /api/v1/sessions?page_size=100&archived_only=true")
    );

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[],"has_more":false},"request_id":"req_02"}"#,
    );
    KimiClient::new(Connection::new(origin, None))
        .list_archived_sessions_before("old/session")
        .unwrap();
    assert!(request.join().unwrap().starts_with(
        "GET /api/v1/sessions?before_id=old%2Fsession&page_size=100&archived_only=true"
    ));
}

#[test]
fn creates_sessions_with_the_selected_model_in_the_initial_profile() {
    let session = r#"{"id":"sess_01","workspace_id":"ws_01","title":"","created_at":"2026-07-18T08:00:00.000Z","updated_at":"2026-07-18T08:00:00.000Z","busy":false,"metadata":{"cwd":"/tmp/project"},"agent_config":{"model":"k3"},"usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_creation_tokens":0,"total_cost_usd":0,"context_tokens":0,"context_limit":100000,"turn_count":0},"permission_rules":[],"message_count":0,"last_seq":0}"#;
    let (origin, request) = one_response(format!(r#"{{"code":0,"msg":"","data":{session}}}"#));

    let created = KimiClient::new(Connection::new(origin, None))
        .create_session("/tmp/project", Some("k3"))
        .unwrap();
    let request = request.join().unwrap();

    assert_eq!(created.agent_config.model, "k3");
    assert!(request.starts_with("POST /api/v1/sessions"));
    assert!(request.contains(r#""agent_config":{"model":"k3"}"#));
    assert!(request.contains(r#""metadata":{"cwd":"/tmp/project"}"#));
}

#[test]
fn manages_workspaces_through_the_daemon_catalog() {
    let workspace = r#"{"id":"wd_project_0123456789ab","root":"/tmp/project","name":"Project","created_at":"2026-07-19T00:00:00Z","last_opened_at":"2026-07-19T01:00:00Z","session_count":0}"#;
    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{{"items":[{workspace}]}},"request_id":"req_list"}}"#
    ));
    let listed = KimiClient::new(Connection::new(origin, None))
        .list_workspaces()
        .unwrap();
    assert_eq!(listed.items[0].name, "Project");
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("GET /api/v1/workspaces")
    );

    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{workspace},"request_id":"req_post"}}"#
    ));
    KimiClient::new(Connection::new(origin, None))
        .register_workspace("/tmp/project")
        .unwrap();
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /api/v1/workspaces"));
    assert!(request.contains(r#"{"root":"/tmp/project"}"#));

    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{workspace},"request_id":"req_patch"}}"#
    ));
    KimiClient::new(Connection::new(origin, None))
        .rename_workspace("wd/project", "Renamed")
        .unwrap();
    let request = request.join().unwrap();
    assert!(request.starts_with("PATCH /api/v1/workspaces/wd%2Fproject"));
    assert!(request.contains(r#"{"name":"Renamed"}"#));

    let (origin, request) =
        one_response(r#"{"code":0,"msg":"","data":{"deleted":true},"request_id":"req_delete"}"#);
    KimiClient::new(Connection::new(origin, None))
        .remove_workspace("wd/project")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("DELETE /api/v1/workspaces/wd%2Fproject")
    );
}

#[test]
fn posts_prompt_content_to_the_session_route() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"prompt_id":"prompt_01","user_message_id":"msg_01","status":"running"},"request_id":"req_01"}"#,
    );
    let client = KimiClient::new(Connection::new(origin, None));

    let result = client.submit_prompt("sess/a", "hello").unwrap();
    let request = request.join().unwrap();

    assert_eq!(result.prompt_id, "prompt_01");
    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/prompts"));
    assert!(request.contains(r#"{"content":[{"type":"text","text":"hello"}]}"#));
}

#[test]
fn starts_and_prompts_a_side_channel_agent_without_polluting_the_main_turn() {
    let (origin, request) =
        one_response(r#"{"code":0,"msg":"","data":{"agent_id":"btw_01"},"request_id":"req_btw"}"#);
    let started = KimiClient::new(Connection::new(origin, None))
        .start_side_chat("sess/a")
        .unwrap();
    assert_eq!(started.agent_id, "btw_01");
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa:btw")
    );

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"prompt_id":"prompt_btw","user_message_id":"msg_btw","status":"running"}}"#,
    );
    let options = kimini::protocol::PromptOptions {
        agent_id: Some("btw_01".into()),
        model: Some("kimi-k2".into()),
        thinking: Some("high".into()),
        permission_mode: Some("manual".into()),
        plan_mode: Some(true),
        swarm_mode: Some(false),
    };
    KimiClient::new(Connection::new(origin, None))
        .submit_prompt_with_options(
            "sess/a",
            &[kimini::protocol::PromptPart::text("quick question")],
            &options,
        )
        .unwrap();
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/prompts"));
    assert!(request.contains(r#""agent_id":"btw_01""#));
    assert!(request.contains(r#""permission_mode":"manual""#));
    assert!(request.contains(r#""content":[{"type":"text","text":"quick question"}]"#));
}

#[test]
fn creates_and_closes_session_terminals_through_encoded_routes() {
    let terminal = r#"{"code":0,"msg":"","data":{"id":"term_01","session_id":"sess/a","cwd":"/workspace","shell":"/bin/zsh","cols":120,"rows":36,"status":"running","created_at":"2026-07-18T08:00:00.000Z"}}"#;
    let (origin, request) = one_response(terminal);
    let created = KimiClient::new(Connection::new(origin, None))
        .create_terminal(
            "sess/a",
            &kimini::protocol::CreateTerminal {
                cols: 120,
                rows: 36,
            },
        )
        .unwrap();
    assert_eq!(created.id, "term_01");
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/terminals"));
    assert!(request.contains(r#"{"cols":120,"rows":36}"#));

    let (origin, request) =
        one_response(r#"{"code":0,"msg":"","data":{"closed":true},"request_id":"req_close"}"#);
    KimiClient::new(Connection::new(origin, None))
        .close_terminal("sess/a", "term/01")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa/terminals/term%2F01:close")
    );
}

#[test]
fn lists_steers_and_removes_server_queued_prompts() {
    let queue = r#"{"code":0,"msg":"","data":{"active":null,"queued":[{"prompt_id":"prompt_01","user_message_id":"msg_01","status":"queued","content":[{"type":"text","text":"follow up"}],"created_at":"2026-07-18T08:00:00.000Z"}]},"request_id":"req_01"}"#;
    let (origin, request) = one_response(queue);
    let prompts = KimiClient::new(Connection::new(origin, None))
        .list_prompts("sess/a")
        .unwrap();
    assert_eq!(prompts.queued[0].prompt_id, "prompt_01");
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("GET /api/v1/sessions/sess%2Fa/prompts")
    );

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"steered":true,"prompt_ids":["prompt_01"]},"request_id":"req_02"}"#,
    );
    KimiClient::new(Connection::new(origin, None))
        .steer_prompts("sess/a", &["prompt_01".into()])
        .unwrap();
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/prompts::steer"));
    assert!(request.contains(r#"{"prompt_ids":["prompt_01"]}"#));

    let (origin, request) = one_response(
        r#"{"code":40903,"msg":"already completed","data":{"aborted":false,"at_seq":42},"request_id":"req_03"}"#,
    );
    let result = KimiClient::new(Connection::new(origin, None))
        .abort_prompt("sess/a", "prompt/1")
        .unwrap();
    assert!(!result.aborted);
    assert_eq!(result.at_seq, Some(42));
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa/prompts/prompt%2F1:abort")
    );
}

#[test]
fn reads_live_runtime_status_from_the_session_status_route() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"busy":true,"model":"kimi-k2","thinking_level":"high","permission":"manual","plan_mode":false,"swarm_mode":true,"context_tokens":42600,"max_context_tokens":100000,"context_usage":0.426},"request_id":"req_01"}"#,
    );
    let client = KimiClient::new(Connection::new(origin, None));

    let status = client.session_status("sess/a").unwrap();
    let request = request.join().unwrap();

    assert_eq!(status.model.as_deref(), Some("kimi-k2"));
    assert_eq!(status.context_percent(), 43);
    assert!(request.starts_with("GET /api/v1/sessions/sess%2Fa/status"));
}

#[test]
fn reads_and_controls_goals_through_the_session_contract() {
    let goal = r#"{"goalId":"goal_01","objective":"Ship native GUI","status":"active","turnsUsed":2,"tokensUsed":1200,"wallClockMs":45000,"budget":{"tokenBudget":10000,"turnBudget":null,"wallClockBudgetMs":null,"remainingTokens":8800,"remainingTurns":null,"remainingWallClockMs":null,"tokenBudgetReached":false,"turnBudgetReached":false,"wallClockBudgetReached":false,"overBudget":false}}"#;
    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{goal},"request_id":"req_goal"}}"#
    ));
    let current = KimiClient::new(Connection::new(origin, None))
        .session_goal("sess/a")
        .unwrap()
        .unwrap();
    assert_eq!(current.goal_id, "goal_01");
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("GET /api/v1/sessions/sess%2Fa/goal")
    );

    let session = r#"{"id":"sess_01","workspace_id":"ws_01","title":"Test","created_at":"2026-07-18T08:00:00.000Z","updated_at":"2026-07-18T08:00:00.000Z","busy":false,"archived":false,"metadata":{"cwd":"/tmp/project"},"agent_config":{"model":"k3"},"usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_creation_tokens":0,"total_cost_usd":0,"context_tokens":0,"context_limit":100000,"turn_count":0},"permission_rules":[],"message_count":0,"last_seq":0}"#;
    let response = format!(r#"{{"code":0,"msg":"","data":{session}}}"#);
    let (origin, request) = one_response(response.clone());
    KimiClient::new(Connection::new(origin, None))
        .set_goal_objective("sess/a", "Ship native GUI")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .contains(r#"{"agent_config":{"goal_objective":"Ship native GUI"}}"#)
    );

    let (origin, request) = one_response(response);
    KimiClient::new(Connection::new(origin, None))
        .control_goal("sess/a", kimini::protocol::GoalControl::Pause)
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .contains(r#"{"agent_config":{"goal_control":"pause"}}"#)
    );
}

#[test]
fn manages_oauth_readiness_and_device_flows() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"ready":false,"providers_count":0,"default_model":null,"managed_provider":{"name":"kimi","status":"unauthenticated"}},"request_id":"req_01"}"#,
    );
    let auth = KimiClient::new(Connection::new(origin, None))
        .auth_summary()
        .unwrap();
    assert!(!auth.ready);
    assert!(request.join().unwrap().starts_with("GET /api/v1/auth"));

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"flow_id":"flow_01","provider":"kimi","status":"pending","verification_uri":"https://example.test/device","verification_uri_complete":"https://example.test/device?code=ABCD","user_code":"ABCD","expires_in":600,"interval":5,"expires_at":"2026-07-18T09:00:00.000Z"},"request_id":"req_02"}"#,
    );
    let flow = KimiClient::new(Connection::new(origin, None))
        .start_oauth_login()
        .unwrap();
    assert_eq!(
        flow.pending_details().map(|(_, code, _)| code),
        Some("ABCD")
    );
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/oauth/login")
    );

    let (origin, request) =
        one_response(r#"{"code":0,"msg":"","data":null,"request_id":"req_03"}"#);
    assert!(
        KimiClient::new(Connection::new(origin, None))
            .oauth_login_status()
            .unwrap()
            .is_none()
    );
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("GET /api/v1/oauth/login")
    );

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"cancelled":true,"status":"cancelled"},"request_id":"req_04"}"#,
    );
    KimiClient::new(Connection::new(origin, None))
        .cancel_oauth_login()
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("DELETE /api/v1/oauth/login")
    );

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"logged_out":true,"provider":"kimi"},"request_id":"req_05"}"#,
    );
    KimiClient::new(Connection::new(origin, None))
        .logout_oauth()
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/oauth/logout")
    );
}

#[test]
fn reads_and_patches_the_daemon_settings_contract() {
    let config = r#"{"default_model":"kimi-code/k3","thinking":{"enabled":true,"effort":"max"},"default_permission_mode":"yolo","default_plan_mode":false,"merge_all_available_skills":true,"telemetry":false}"#;
    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{config},"request_id":"req_config_get"}}"#
    ));
    let loaded = KimiClient::new(Connection::new(origin, None))
        .get_config()
        .unwrap();
    assert_eq!(loaded.default_model.as_deref(), Some("kimi-code/k3"));
    assert_eq!(loaded.default_permission_mode.as_deref(), Some("yolo"));
    assert!(request.join().unwrap().starts_with("GET /api/v1/config"));

    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{config},"request_id":"req_config_post"}}"#
    ));
    KimiClient::new(Connection::new(origin, None))
        .patch_config(&serde_json::json!({
            "default_permission_mode": "auto",
            "thinking": { "enabled": false }
        }))
        .unwrap();
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /api/v1/config"));
    assert!(request.contains(r#""default_permission_mode":"auto""#));
    assert!(request.contains(r#""thinking":{"enabled":false}"#));
}

#[test]
fn lists_reads_and_cancels_background_tasks() {
    let task = r#"{"id":"task_01","session_id":"sess/a","kind":"bash","description":"Run tests","status":"running","command":"cargo test","created_at":"2026-07-18T08:00:00.000Z"}"#;
    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{{"items":[{task}]}},"request_id":"req_01"}}"#
    ));
    let list = KimiClient::new(Connection::new(origin, None))
        .list_tasks("sess/a")
        .unwrap();
    assert_eq!(list.items[0].description, "Run tests");
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("GET /api/v1/sessions/sess%2Fa/tasks")
    );

    let detail = task.replace(
        r#""created_at":"2026-07-18T08:00:00.000Z""#,
        r#""created_at":"2026-07-18T08:00:00.000Z","output_preview":"ok","output_bytes":2"#,
    );
    let (origin, request) = one_response(format!(
        r#"{{"code":0,"msg":"","data":{detail},"request_id":"req_02"}}"#
    ));
    let detail = KimiClient::new(Connection::new(origin, None))
        .task_with_output("sess/a", "task/01", 4096)
        .unwrap();
    assert_eq!(detail.output_preview.as_deref(), Some("ok"));
    assert!(request.join().unwrap().starts_with(
        "GET /api/v1/sessions/sess%2Fa/tasks/task%2F01?with_output=true&output_bytes=4096"
    ));

    let (origin, request) =
        one_response(r#"{"code":0,"msg":"","data":{"cancelled":true},"request_id":"req_03"}"#);
    KimiClient::new(Connection::new(origin, None))
        .cancel_task("sess/a", "task/01")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa/tasks/task%2F01:cancel")
    );
}

#[test]
fn pages_older_messages_from_the_oldest_visible_message() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[],"has_more":false},"request_id":"req_01"}"#,
    );
    let client = KimiClient::new(Connection::new(origin, None));

    let page = client.list_messages_before("sess/a", "msg old/1").unwrap();
    let request = request.join().unwrap();

    assert!(page.items.is_empty());
    assert!(request.starts_with(
        "GET /api/v1/sessions/sess%2Fa/messages?before_id=msg%20old%2F1&page_size=100"
    ));
}

#[test]
fn uploads_files_as_authenticated_multipart_data() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"id":"f_01","name":"asset.png","media_type":"image/png","size":4,"created_at":"2026-07-18T08:00:00.000Z"},"request_id":"req_01"}"#,
    );
    let path = std::env::temp_dir().join(format!("kimini-upload-{}.png", std::process::id()));
    fs::write(&path, [0_u8, 1, 2, 3]).unwrap();
    let client = KimiClient::new(Connection::new(origin, Some("secret".into())));

    let file = client.upload_file(&path).unwrap();
    let request = request.join().unwrap();
    fs::remove_file(path).unwrap();

    assert_eq!(file.id, "f_01");
    assert!(request.starts_with("POST /api/v1/files"));
    assert!(request.contains("Content-Type: multipart/form-data; boundary=kimini-"));
    assert!(request.contains("Authorization: Bearer secret"));
    assert!(request.contains("filename=\"kimini-upload-"));
    assert!(request.contains("Content-Type: image/png"));
}

#[test]
fn posts_runtime_controls_to_the_session_profile_route() {
    let session = r#"{"id":"sess_01","workspace_id":"ws_01","title":"Test","created_at":"2026-07-18T08:00:00.000Z","updated_at":"2026-07-18T08:00:00.000Z","busy":false,"archived":false,"metadata":{"cwd":"/tmp/project"},"agent_config":{"model":"k3"},"usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_creation_tokens":0,"total_cost_usd":0,"context_tokens":0,"context_limit":100000,"turn_count":0},"permission_rules":[],"message_count":0,"last_seq":0}"#;
    let response = format!(r#"{{"code":0,"msg":"","data":{session},"request_id":"req_01"}}"#);
    let (origin, request) = one_response(response);
    let client = KimiClient::new(Connection::new(origin, None));

    client
        .update_session_config("sess/a", serde_json::json!({ "permission_mode": "auto" }))
        .unwrap();
    let request = request.join().unwrap();

    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/profile"));
    assert!(request.contains(r#"{"agent_config":{"permission_mode":"auto"}}"#));
}

#[test]
fn posts_swarm_mode_to_the_authoritative_agent_config() {
    let session = r#"{"id":"sess_01","workspace_id":"ws_01","title":"Test","created_at":"2026-07-18T08:00:00.000Z","updated_at":"2026-07-18T08:00:00.000Z","busy":false,"archived":false,"metadata":{"cwd":"/tmp/project"},"agent_config":{"model":"k3"},"usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_creation_tokens":0,"total_cost_usd":0,"context_tokens":0,"context_limit":100000,"turn_count":0},"permission_rules":[],"message_count":0,"last_seq":0}"#;
    let response = format!(r#"{{"code":0,"msg":"","data":{session},"request_id":"req_01"}}"#);
    let (origin, request) = one_response(response);

    KimiClient::new(Connection::new(origin, None))
        .update_session_config("sess/a", serde_json::json!({ "swarm_mode": true }))
        .unwrap();

    assert!(
        request
            .join()
            .unwrap()
            .contains(r#"{"agent_config":{"swarm_mode":true}}"#)
    );
}

#[test]
fn renames_sessions_through_the_shared_profile_route() {
    let session = r#"{"id":"sess_01","workspace_id":"ws_01","title":"Renamed","created_at":"2026-07-18T08:00:00.000Z","updated_at":"2026-07-18T08:00:00.000Z","busy":false,"archived":false,"metadata":{"cwd":"/tmp/project"},"agent_config":{"model":"k3"},"usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_creation_tokens":0,"total_cost_usd":0,"context_tokens":0,"context_limit":100000,"turn_count":0},"permission_rules":[],"message_count":0,"last_seq":0}"#;
    let response = format!(r#"{{"code":0,"msg":"","data":{session},"request_id":"req_01"}}"#);
    let (origin, request) = one_response(response);

    let renamed = KimiClient::new(Connection::new(origin, None))
        .rename_session("sess/a", "Renamed")
        .unwrap();
    let request = request.join().unwrap();

    assert_eq!(renamed.title, "Renamed");
    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/profile"));
    assert!(request.contains(r#"{"title":"Renamed"}"#));
}

#[test]
fn compact_and_undo_use_session_action_routes() {
    let (origin, request) = one_response(r#"{"code":0,"msg":"","data":{},"request_id":"req_01"}"#);
    KimiClient::new(Connection::new(origin, None))
        .compact_session("sess/a")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa:compact")
    );

    let (origin, request) = one_response(r#"{"code":0,"msg":"","data":{},"request_id":"req_01b"}"#);
    KimiClient::new(Connection::new(origin, None))
        .compact_session_with_instruction("sess/a", Some("focus on decisions"))
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .contains(r#"{"instruction":"focus on decisions"}"#)
    );

    let (origin, request) = one_response(r#"{"code":0,"msg":"","data":{},"request_id":"req_02"}"#);
    KimiClient::new(Connection::new(origin, None))
        .undo_session("sess/a")
        .unwrap();
    let request = request.join().unwrap();
    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa:undo"));
    assert!(request.contains(r#"{"count":1,"page_size":100}"#));
}

#[test]
fn forks_and_archives_through_action_suffix_routes() {
    let session = r#"{"id":"fork_01","workspace_id":"ws_01","title":"Fork","created_at":"2026-07-18T08:00:00.000Z","updated_at":"2026-07-18T08:00:00.000Z","busy":false,"archived":false,"metadata":{"cwd":"/tmp/project"},"agent_config":{"model":"k3"},"usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_creation_tokens":0,"total_cost_usd":0,"context_tokens":0,"context_limit":100000,"turn_count":0},"permission_rules":[],"message_count":0,"last_seq":0}"#;
    let response = format!(r#"{{"code":0,"msg":"","data":{session},"request_id":"req_01"}}"#);
    let (origin, request) = one_response(response);
    KimiClient::new(Connection::new(origin, None))
        .fork_session("sess/a")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa:fork")
    );

    let (origin, request) =
        one_response(r#"{"code":0,"msg":"","data":{"archived":true},"request_id":"req_02"}"#);
    KimiClient::new(Connection::new(origin, None))
        .archive_session("sess/a")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa:archive")
    );
}

#[test]
fn restores_archived_sessions_through_the_action_route() {
    let session = r#"{"id":"restored_01","workspace_id":"ws_01","title":"Restored","created_at":"2026-07-18T08:00:00.000Z","updated_at":"2026-07-18T08:00:00.000Z","busy":false,"archived":false,"metadata":{"cwd":"/tmp/project"},"agent_config":{"model":"k3"},"usage":{"input_tokens":0,"output_tokens":0,"cache_read_tokens":0,"cache_creation_tokens":0,"total_cost_usd":0,"context_tokens":0,"context_limit":100000,"turn_count":0},"permission_rules":[],"message_count":0,"last_seq":0}"#;
    let response = format!(r#"{{"code":0,"msg":"","data":{session},"request_id":"req_01"}}"#);
    let (origin, request) = one_response(response);

    let restored = KimiClient::new(Connection::new(origin, None))
        .restore_session("archived/1")
        .unwrap();

    assert_eq!(restored.id, "restored_01");
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/archived%2F1:restore")
    );
}

#[test]
fn streams_session_exports_to_the_selected_path() {
    let archive = b"PK\x03\x04native-session";
    let (origin, request) = one_binary_response("application/zip", archive);
    let path = std::env::temp_dir().join(format!(
        "kimini-export-{}-{}.zip",
        std::process::id(),
        archive.len()
    ));

    let exported = KimiClient::new(Connection::new(origin, Some("secret".into())))
        .export_session_to("sess/a", &path)
        .unwrap();
    let captured = request.join().unwrap();

    assert_eq!(exported.bytes, archive.len() as u64);
    assert_eq!(fs::read(&path).unwrap(), archive);
    assert!(captured.starts_with("POST /api/v1/sessions/sess%2Fa/export"));
    assert!(captured.contains("Authorization: Bearer secret"));
    fs::remove_file(path).unwrap();
}

#[test]
fn workspace_file_routes_keep_paths_in_json_and_decode_native_surfaces() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[{"path":"src","name":"src","kind":"directory","modified_at":"2026-07-18T08:00:00.000Z","child_count":1}],"truncated":false},"request_id":"req_01"}"#,
    );
    let list = KimiClient::new(Connection::new(origin, None))
        .list_files("sess/a", "src/safe path")
        .unwrap();
    assert_eq!(list.items[0].name, "src");
    let captured = request.join().unwrap();
    assert!(captured.starts_with("POST /api/v1/sessions/sess%2Fa/fs:list"));
    assert!(captured.contains(r#""path":"src/safe path""#));
    assert!(captured.contains(r#""include_git_status":true"#));

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"path":"src/main.rs","content":"fn main() {}","encoding":"utf-8","size":12,"truncated":false,"etag":"v1","mime":"text/x-rust","language_id":"rust","line_count":1,"is_binary":false},"request_id":"req_02"}"#,
    );
    let file = KimiClient::new(Connection::new(origin, None))
        .read_workspace_file("sess/a", "src/main.rs")
        .unwrap();
    assert_eq!(file.language_id.as_deref(), Some("rust"));
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa/fs:read")
    );

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"path":"src/main.rs","diff":"@@ -1 +1 @@","truncated":false},"request_id":"req_03"}"#,
    );
    let diff = KimiClient::new(Connection::new(origin, None))
        .workspace_file_diff("sess/a", "src/main.rs")
        .unwrap();
    assert!(diff.diff.starts_with("@@"));
    assert!(request.join().unwrap().contains(r#""path":"src/main.rs""#));
}

#[test]
fn workspace_search_and_git_status_follow_the_daemon_contract() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"items":[{"path":"src/main.rs","name":"main.rs","kind":"file","score":0.95,"match_positions":[0,1]}],"truncated":false},"request_id":"req_01"}"#,
    );
    let results = KimiClient::new(Connection::new(origin, None))
        .search_workspace_files("sess/a", "main")
        .unwrap();
    assert_eq!(results.items[0].path, "src/main.rs");
    let captured = request.join().unwrap();
    assert!(captured.starts_with("POST /api/v1/sessions/sess%2Fa/fs:search"));
    assert!(captured.contains(r#""query":"main""#));

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"branch":"main","ahead":1,"behind":0,"entries":{"src/main.rs":"modified"},"additions":4,"deletions":2,"pullRequest":null},"request_id":"req_02"}"#,
    );
    let status = KimiClient::new(Connection::new(origin, None))
        .workspace_git_status("sess/a")
        .unwrap();
    assert_eq!(status.branch, "main");
    assert_eq!(status.additions, 4);
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("POST /api/v1/sessions/sess%2Fa/fs:git_status")
    );
}

#[test]
fn lists_and_activates_session_skills_through_encoded_routes() {
    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"skills":[{"name":"review/code","description":"Review code","path":"/skills/review","source":"project","disable_model_invocation":true}]},"request_id":"req_01"}"#,
    );
    let skills = KimiClient::new(Connection::new(origin, None))
        .list_skills("sess/a")
        .unwrap();
    assert_eq!(skills.skills[0].name, "review/code");
    assert!(skills.skills[0].disable_model_invocation);
    assert!(
        request
            .join()
            .unwrap()
            .starts_with("GET /api/v1/sessions/sess%2Fa/skills")
    );

    let (origin, request) = one_response(
        r#"{"code":0,"msg":"","data":{"activated":true,"skill_name":"review/code"},"request_id":"req_02"}"#,
    );
    let result = KimiClient::new(Connection::new(origin, None))
        .activate_skill("sess/a", "review/code", Some("--fix"))
        .unwrap();
    assert!(result.activated);
    let captured = request.join().unwrap();
    assert!(captured.starts_with("POST /api/v1/sessions/sess%2Fa/skills/review%2Fcode:activate"));
    assert!(captured.contains(r#"{"args":"--fix"}"#));
}

#[test]
fn surfaces_daemon_and_shape_errors_without_credentials() {
    let (origin, _) =
        one_response(r#"{"code":40901,"msg":"busy","data":null,"request_id":"req_01"}"#);
    let error = KimiClient::new(Connection::new(origin, None))
        .list_sessions()
        .unwrap_err();
    assert_eq!(error.to_string(), "Kimi daemon error 40901: busy");

    let (origin, _) = one_response(r#"{"code":0,"msg":"","data":null,"request_id":"req_02"}"#);
    let error = KimiClient::new(Connection::new(origin, None))
        .list_sessions()
        .unwrap_err();
    assert_eq!(error.to_string(), "Kimi daemon returned no data");

    let (origin, _) = one_response("not-json");
    let error = KimiClient::new(Connection::new(origin, None))
        .list_sessions()
        .unwrap_err();
    assert!(
        error
            .to_string()
            .starts_with("Invalid Kimi daemon response:")
    );
}

#[test]
fn question_answers_use_the_real_item_identifier_as_the_json_key() {
    let (origin, request) = one_response(r#"{"code":0,"msg":"","data":{},"request_id":"req_01"}"#);
    let client = KimiClient::new(Connection::new(origin, None));

    client
        .resolve_question("sess_01", "question_01", "item/1", "option_01")
        .unwrap();
    let request = request.join().unwrap();

    assert!(request.contains(
        r#"{"answers":{"item/1":{"kind":"single","option_id":"option_01"}},"method":"click"}"#
    ));
}

#[test]
fn approval_scope_and_multi_question_answers_match_the_interaction_contract() {
    let (origin, request) = one_response(r#"{"code":0,"msg":"","data":{},"request_id":"req_01"}"#);
    KimiClient::new(Connection::new(origin, None))
        .resolve_approval_for_session("sess_01", "approval_01")
        .unwrap();
    assert!(
        request
            .join()
            .unwrap()
            .contains(r#"{"decision":"approved","scope":"session"}"#)
    );

    let (origin, request) = one_response(r#"{"code":0,"msg":"","data":{},"request_id":"req_02"}"#);
    let answers = kimini::protocol::QuestionAnswers::from([
        (
            "single".into(),
            kimini::protocol::QuestionAnswer::Single {
                option_id: "a".into(),
            },
        ),
        (
            "multi".into(),
            kimini::protocol::QuestionAnswer::Multi {
                option_ids: vec!["x".into(), "y".into()],
            },
        ),
    ]);
    KimiClient::new(Connection::new(origin, None))
        .resolve_question_answers("sess_01", "question_01", &answers)
        .unwrap();
    let request = request.join().unwrap();
    assert!(request.contains(r#""single":{"kind":"single","option_id":"a"}"#));
    assert!(request.contains(r#""multi":{"kind":"multi","option_ids":["x","y"]}"#));
}

#[test]
fn question_dismiss_accepts_the_daemons_terminal_success_code() {
    let (origin, request) = one_response(
        r#"{"code":40909,"msg":"dismissed","data":{"dismissed":true,"dismissed_at":"now"},"request_id":"req_01"}"#,
    );

    KimiClient::new(Connection::new(origin, None))
        .dismiss_question("sess/a", "question/1")
        .unwrap();
    let request = request.join().unwrap();

    assert!(request.starts_with("POST /api/v1/sessions/sess%2Fa/questions/question%2F1:dismiss"));
}

fn one_response(body: impl Into<String>) -> (String, thread::JoinHandle<String>) {
    let body = body.into();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        loop {
            let mut buffer = [0_u8; 4096];
            let length = stream.read(&mut buffer).unwrap();
            if length == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..length]);
            if request_complete(&request) {
                break;
            }
        }
        let request = String::from_utf8_lossy(&request).into_owned();
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
        request
    });
    (format!("http://{address}"), handle)
}

fn many_responses(bodies: Vec<String>) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        bodies
            .into_iter()
            .map(|body| {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = Vec::new();
                loop {
                    let mut buffer = [0_u8; 4096];
                    let length = stream.read(&mut buffer).unwrap();
                    if length == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..length]);
                    if request_complete(&request) {
                        break;
                    }
                }
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).unwrap();
                String::from_utf8_lossy(&request).into_owned()
            })
            .collect()
    });
    (format!("http://{address}"), handle)
}

fn session_json(id: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "workspace_id": "ws_01",
        "title": id,
        "created_at": "2026-07-18T08:00:00.000Z",
        "updated_at": "2026-07-18T08:00:00.000Z",
        "busy": false,
        "archived": false,
        "metadata": { "cwd": "/tmp/project" },
        "agent_config": { "model": "k3" },
        "usage": {
            "input_tokens": 0,
            "output_tokens": 0,
            "cache_read_tokens": 0,
            "cache_creation_tokens": 0,
            "total_cost_usd": 0,
            "context_tokens": 0,
            "context_limit": 100000,
            "turn_count": 0
        },
        "permission_rules": [],
        "message_count": 0,
        "last_seq": 0
    })
}

fn one_binary_response(
    content_type: &'static str,
    body: &'static [u8],
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        loop {
            let mut buffer = [0_u8; 4096];
            let length = stream.read(&mut buffer).unwrap();
            if length == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..length]);
            if request_complete(&request) {
                break;
            }
        }
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(body).unwrap();
        String::from_utf8_lossy(&request).into_owned()
    });
    (format!("http://{address}"), handle)
}

fn request_complete(request: &[u8]) -> bool {
    let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
        return false;
    };
    let headers = String::from_utf8_lossy(&request[..header_end]).to_ascii_lowercase();
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length:"))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0);
    request.len() >= header_end + 4 + content_length
}
