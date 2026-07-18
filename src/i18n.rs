//! Host UI language (English / Simplified Chinese).
//!
//! Resolution order:
//! 1. `$KIMINI_LANG` (`en`, `zh`, `zh-CN`, `zh-Hans`, …)
//! 2. Saved preference (`lang` under the app config dir)
//! 3. System locale (`LANG` / `LC_ALL` / macOS `AppleLocale`)
//! 4. English (default)
//!
//! Chinese copy lives only in [`Strings::zh`]. Product docs stay English.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Supported UI languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub const fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Zh => "zh",
        }
    }

    /// Parse a BCP-47 / locale-ish tag into a supported language.
    pub fn parse_tag(raw: &str) -> Option<Self> {
        let s = raw.trim();
        if s.is_empty() {
            return None;
        }
        let primary = s
            .split(['_', '-', '.'])
            .next()
            .unwrap_or(s)
            .to_ascii_lowercase();
        match primary.as_str() {
            "en" => Some(Self::En),
            "zh" | "cn" => Some(Self::Zh),
            _ => None,
        }
    }

    pub fn from_env() -> Option<Self> {
        env::var("KIMINI_LANG")
            .ok()
            .as_deref()
            .and_then(Self::parse_tag)
    }

    pub fn from_system() -> Self {
        for key in ["LC_ALL", "LC_MESSAGES", "LANG"] {
            if let Ok(v) = env::var(key)
                && let Some(lang) = Self::parse_tag(&v)
            {
                return lang;
            }
        }

        #[cfg(target_os = "macos")]
        if let Some(lang) = apple_locale() {
            return lang;
        }

        Self::En
    }

    /// Resolve active language: env → preference → system → English.
    pub fn resolve() -> Self {
        if let Some(lang) = Self::from_env() {
            return lang;
        }
        if let Some(lang) = load_preference() {
            return lang;
        }
        Self::from_system()
    }

    pub fn strings(self) -> Strings {
        match self {
            Self::En => Strings::en(),
            Self::Zh => Strings::zh(),
        }
    }
}

/// Host-owned UI copy (menus + Settings window). Web content is not translated.
#[derive(Debug, Clone, Copy)]
pub struct Strings {
    pub native: NativeStrings,
    pub about: &'static str,
    pub settings: &'static str,
    pub settings_title: &'static str,
    pub edit: &'static str,
    pub navigate: &'static str,
    pub window: &'static str,
    pub reload: &'static str,
    pub back: &'static str,
    pub forward: &'static str,
    pub language: &'static str,
    pub language_hint: &'static str,
    pub lang_en: &'static str,
    pub lang_zh: &'static str,
    pub launch_probing: &'static str,
    pub launch_starting: &'static str,
    pub launch_waiting: &'static str,
    pub launch_no_kimi: &'static str,
    pub launch_invalid_url: &'static str,
    pub launch_manual: &'static str,
    pub launch_manual_hint: &'static str,
    pub launch_connect: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct NativeStrings {
    pub sessions: &'static str,
    pub sessions_list: &'static str,
    pub new_session: &'static str,
    pub search_sessions: &'static str,
    pub load_more_sessions: &'static str,
    pub show_more_conversations: &'static str,
    pub show_less_conversations: &'static str,
    pub loading_sessions: &'static str,
    pub untitled_session: &'static str,
    pub archived_sessions: &'static str,
    pub active_sessions: &'static str,
    pub restore_session: &'static str,
    pub no_archived_sessions: &'static str,
    pub choose_folder: &'static str,
    pub start_session: &'static str,
    pub start_session_hint: &'static str,
    pub load_earlier: &'static str,
    pub loading_earlier: &'static str,
    pub ask_placeholder: &'static str,
    pub attach_file: &'static str,
    pub uploading_file: &'static str,
    pub upload_failed: &'static str,
    pub remove_attachment: &'static str,
    pub attachment_image: &'static str,
    pub attachment_video: &'static str,
    pub attachment_file: &'static str,
    pub browser_address: &'static str,
    pub browser_content: &'static str,
    pub conversation: &'static str,
    pub message_composer: &'static str,
    pub session_runtime: &'static str,
    pub default_model: &'static str,
    pub context_label: &'static str,
    pub enter_hint: &'static str,
    pub send: &'static str,
    pub queue: &'static str,
    pub queued_prompts: &'static str,
    pub queued_attachments: &'static str,
    pub steer: &'static str,
    pub steering: &'static str,
    pub remove_from_queue: &'static str,
    pub stop: &'static str,
    pub model: &'static str,
    pub model_required: &'static str,
    pub thinking: &'static str,
    pub preview_thinking: &'static str,
    pub thinking_preview_hint: &'static str,
    pub close_thinking: &'static str,
    pub permission: &'static str,
    pub plan_on: &'static str,
    pub plan_off: &'static str,
    pub swarm_on: &'static str,
    pub swarm_off: &'static str,
    pub goal_mode_on: &'static str,
    pub goal_mode_off: &'static str,
    pub side_chat: &'static str,
    pub side_chat_panel: &'static str,
    pub side_chat_subtitle: &'static str,
    pub side_chat_placeholder: &'static str,
    pub side_chat_empty: &'static str,
    pub side_chat_send: &'static str,
    pub side_chat_close: &'static str,
    pub side_chat_opening: &'static str,
    pub side_chat_thinking: &'static str,
    pub side_chat_stop: &'static str,
    pub terminal: &'static str,
    pub terminal_panel: &'static str,
    pub terminal_placeholder: &'static str,
    pub terminal_empty: &'static str,
    pub terminal_loading: &'static str,
    pub terminal_new: &'static str,
    pub terminal_send: &'static str,
    pub terminal_close: &'static str,
    pub terminal_close_tab: &'static str,
    pub terminal_running: &'static str,
    pub terminal_exited: &'static str,
    pub terminal_local: &'static str,
    pub terminal_local_fallback: &'static str,
    pub goal_label: &'static str,
    pub goal_active: &'static str,
    pub goal_paused: &'static str,
    pub goal_blocked: &'static str,
    pub start_goal: &'static str,
    pub starting_goal: &'static str,
    pub goal_start_question: &'static str,
    pub pause_goal: &'static str,
    pub resume_goal: &'static str,
    pub cancel_goal: &'static str,
    pub keep_goal: &'static str,
    pub goal_cancel_question: &'static str,
    pub goal_cancel_detail: &'static str,
    pub done_when: &'static str,
    pub turns: &'static str,
    pub tokens: &'static str,
    pub fork: &'static str,
    pub session_actions: &'static str,
    pub rename_session: &'static str,
    pub save: &'static str,
    pub compact: &'static str,
    pub compact_question: &'static str,
    pub compact_detail: &'static str,
    pub undo: &'static str,
    pub undo_question: &'static str,
    pub undo_detail: &'static str,
    pub export_session: &'static str,
    pub exporting: &'static str,
    pub archive: &'static str,
    pub archive_question: &'static str,
    pub archive_detail: &'static str,
    pub cancel: &'static str,
    pub browser: &'static str,
    pub close_browser: &'static str,
    pub connected: &'static str,
    pub connecting: &'static str,
    pub working: &'static str,
    pub retry_connection: &'static str,
    pub open_web_fallback: &'static str,
    pub you: &'static str,
    pub kimi: &'static str,
    pub tool: &'static str,
    pub system: &'static str,
    pub tool_result: &'static str,
    pub tool_read: &'static str,
    pub tool_edit: &'static str,
    pub tool_write: &'static str,
    pub tool_command: &'static str,
    pub tool_search: &'static str,
    pub show_tool_details: &'static str,
    pub hide_tool_details: &'static str,
    pub approve: &'static str,
    pub approve_once: &'static str,
    pub approve_session: &'static str,
    pub reject: &'static str,
    pub approval_required: &'static str,
    pub question_required: &'static str,
    pub submit_answers: &'static str,
    pub dismiss_question: &'static str,
    pub other_answer: &'static str,
    pub recommended: &'static str,
    pub switch_language: &'static str,
    pub tasks_panel: &'static str,
    pub tasks: &'static str,
    pub close_tasks: &'static str,
    pub refresh_tasks: &'static str,
    pub tasks_loading: &'static str,
    pub no_tasks: &'static str,
    pub cancel_task: &'static str,
    pub subagent_task: &'static str,
    pub shell_task: &'static str,
    pub tool_task: &'static str,
    pub task_running: &'static str,
    pub task_completed: &'static str,
    pub task_failed: &'static str,
    pub task_cancelled: &'static str,
    pub files_panel: &'static str,
    pub files: &'static str,
    pub close_files: &'static str,
    pub refresh_files: &'static str,
    pub search_files: &'static str,
    pub workspace_files: &'static str,
    pub loading_files: &'static str,
    pub no_files: &'static str,
    pub file_preview: &'static str,
    pub loading_file: &'static str,
    pub select_file: &'static str,
    pub source: &'static str,
    pub diff: &'static str,
    pub binary_file: &'static str,
    pub skills_panel: &'static str,
    pub workspace_tools: &'static str,
    pub skills: &'static str,
    pub close_skills: &'static str,
    pub refresh_skills: &'static str,
    pub skills_loading: &'static str,
    pub no_skills: &'static str,
    pub activate_skill: &'static str,
    pub activating_skill: &'static str,
    pub skill_activated: &'static str,
    pub slash_only: &'static str,
    pub skill_activation_unacknowledged: &'static str,
    pub skill_attachments_unsupported: &'static str,
    pub command_attachments_unsupported: &'static str,
    pub built_in_command: &'static str,
    pub slash_commands: &'static str,
    pub auth_panel: &'static str,
    pub authentication: &'static str,
    pub close_auth: &'static str,
    pub refresh_auth: &'static str,
    pub auth_status: &'static str,
    pub auth_ready: &'static str,
    pub auth_required: &'static str,
    pub auth_working: &'static str,
    pub auth_failed: &'static str,
    pub auth_denied: &'static str,
    pub auth_expired: &'static str,
    pub auth_cancelled: &'static str,
    pub providers: &'static str,
    pub sign_in: &'static str,
    pub sign_in_hint: &'static str,
    pub sign_out: &'static str,
    pub finish_sign_in: &'static str,
    pub device_code: &'static str,
    pub open_sign_in: &'static str,
    pub cancel_sign_in: &'static str,
}

impl Strings {
    pub const fn en() -> Self {
        Self {
            native: NativeStrings {
                sessions: "Sessions",
                sessions_list: "Kimi Code sessions",
                new_session: "+ New",
                search_sessions: "Search sessions",
                load_more_sessions: "Load more sessions",
                show_more_conversations: "Show {count} more conversations",
                show_less_conversations: "Show fewer conversations",
                loading_sessions: "Loading sessions…",
                untitled_session: "Untitled session",
                archived_sessions: "Archived",
                active_sessions: "Active",
                restore_session: "Restore",
                no_archived_sessions: "No archived sessions",
                choose_folder: "Create Kimi session in this folder",
                start_session: "Start a Kimi Code session",
                start_session_hint: "Select a session or send a prompt to begin.",
                load_earlier: "Load earlier messages",
                loading_earlier: "Loading earlier messages…",
                ask_placeholder: "Ask Kimi to work on this project…",
                attach_file: "Attach files",
                uploading_file: "Uploading",
                upload_failed: "Upload failed",
                remove_attachment: "Remove attachment",
                attachment_image: "Image",
                attachment_video: "Video",
                attachment_file: "File",
                browser_address: "Enter a web address",
                browser_content: "Browser content",
                conversation: "Conversation",
                message_composer: "Message composer",
                session_runtime: "Session runtime",
                default_model: "Default model",
                context_label: "context",
                enter_hint: "Enter to send · Shift+Enter for a new line",
                send: "Send",
                queue: "Queue",
                queued_prompts: "Queued prompts",
                queued_attachments: "attachments",
                steer: "Steer now",
                steering: "Steering",
                remove_from_queue: "Remove from queue",
                stop: "Stop",
                model: "Model",
                model_required: "Choose a model before sending.",
                thinking: "Thinking",
                preview_thinking: "Preview Thinking",
                thinking_preview_hint: "Select a thinking trace to inspect it here.",
                close_thinking: "Close thinking preview",
                permission: "Permission",
                plan_on: "Plan · On",
                plan_off: "Plan · Off",
                swarm_on: "Swarm · On",
                swarm_off: "Swarm · Off",
                goal_mode_on: "Goal · Next",
                goal_mode_off: "Goal · Off",
                side_chat: "BTW",
                side_chat_panel: "Side chat",
                side_chat_subtitle: "Ask without interrupting the main turn",
                side_chat_placeholder: "Ask a quick side question…",
                side_chat_empty: "Side questions and answers stay out of the main conversation.",
                side_chat_send: "Send",
                side_chat_close: "Close",
                side_chat_opening: "Opening side chat…",
                side_chat_thinking: "Thinking",
                side_chat_stop: "Stop",
                terminal: "Terminal",
                terminal_panel: "Session terminals",
                terminal_placeholder: "Run a shell command…",
                terminal_empty: "No terminal for this session",
                terminal_loading: "Opening terminal…",
                terminal_new: "New",
                terminal_send: "Run",
                terminal_close: "Close",
                terminal_close_tab: "Close terminal",
                terminal_running: "Running",
                terminal_exited: "Exited",
                terminal_local: "Local",
                terminal_local_fallback: "Kimi daemon PTY is unavailable; using Kimini's local Rust terminal.",
                goal_label: "Goal",
                goal_active: "Active",
                goal_paused: "Paused",
                goal_blocked: "Blocked",
                start_goal: "Start goal",
                starting_goal: "Starting goal",
                goal_start_question: "Start this goal? Kimi will work autonomously toward it.",
                pause_goal: "Pause",
                resume_goal: "Resume",
                cancel_goal: "Cancel goal",
                keep_goal: "Keep goal",
                goal_cancel_question: "Cancel this goal?",
                goal_cancel_detail: "A cancelled goal cannot be resumed.",
                done_when: "Done when",
                turns: "turns",
                tokens: "tokens",
                fork: "Fork",
                session_actions: "Session actions",
                rename_session: "Rename",
                save: "Save",
                compact: "Compact context",
                compact_question: "Compact this session context?",
                compact_detail: "Kimi will summarize older context before continuing.",
                undo: "Undo last turn",
                undo_question: "Undo the last turn?",
                undo_detail: "The latest user and assistant turn will be removed from this session.",
                export_session: "Export diagnostics",
                exporting: "Exporting session…",
                archive: "Archive",
                archive_question: "Archive this session?",
                archive_detail: "You can restore it later in Kimi Code Web.",
                cancel: "Cancel",
                browser: "Browser",
                close_browser: "Close Browser",
                connected: "Connected",
                connecting: "Connecting to Kimi Code…",
                working: "Working",
                retry_connection: "Reconnect",
                open_web_fallback: "Open Kimini Web",
                you: "You",
                kimi: "Kimi",
                tool: "Tool",
                system: "System",
                tool_result: "Tool result",
                tool_read: "Read",
                tool_edit: "Edit",
                tool_write: "Write",
                tool_command: "Command",
                tool_search: "Search",
                show_tool_details: "Show details",
                hide_tool_details: "Hide details",
                approve: "Approve",
                approve_once: "Approve once",
                approve_session: "Allow for session",
                reject: "Reject",
                approval_required: "Approval required",
                question_required: "Answer required",
                submit_answers: "Submit answers",
                dismiss_question: "Dismiss",
                other_answer: "Other",
                recommended: "Recommended",
                switch_language: "中文",
                tasks_panel: "Background tasks and subagents",
                tasks: "Tasks",
                close_tasks: "Close",
                refresh_tasks: "Refresh",
                tasks_loading: "Loading tasks…",
                no_tasks: "No background tasks in this session",
                cancel_task: "Cancel",
                subagent_task: "Subagent",
                shell_task: "Shell",
                tool_task: "Tool",
                task_running: "Running",
                task_completed: "Completed",
                task_failed: "Failed",
                task_cancelled: "Cancelled",
                files_panel: "Workspace files and preview",
                files: "Files",
                close_files: "Close",
                refresh_files: "Refresh",
                search_files: "Search files, then press Enter",
                workspace_files: "Workspace files",
                loading_files: "Loading files…",
                no_files: "No files",
                file_preview: "File preview",
                loading_file: "Loading file…",
                select_file: "Select a file to preview it.",
                source: "Source",
                diff: "Diff",
                binary_file: "Binary file",
                skills_panel: "Session skills",
                workspace_tools: "Workspace",
                skills: "Skills",
                close_skills: "Close",
                refresh_skills: "Refresh",
                skills_loading: "Loading skills…",
                no_skills: "No skills available in this session",
                activate_skill: "Activate",
                activating_skill: "Activating…",
                skill_activated: "Activated",
                slash_only: "Slash only",
                skill_activation_unacknowledged: "Skill activation was not acknowledged",
                skill_attachments_unsupported: "Remove attachments before activating a skill",
                command_attachments_unsupported: "Remove attachments before running a command",
                built_in_command: "Built-in command",
                slash_commands: "Commands and skills",
                auth_panel: "Kimi authentication",
                authentication: "Auth",
                close_auth: "Close",
                refresh_auth: "Refresh",
                auth_status: "Authentication",
                auth_ready: "Ready",
                auth_required: "Sign in",
                auth_working: "Working…",
                auth_failed: "Sign-in failed",
                auth_denied: "Sign-in was denied",
                auth_expired: "The sign-in code expired",
                auth_cancelled: "Sign-in was cancelled",
                providers: "Providers",
                sign_in: "Sign in with Kimi",
                sign_in_hint: "Connect a provider before starting a coding session.",
                sign_out: "Sign out",
                finish_sign_in: "Finish signing in",
                device_code: "Device code",
                open_sign_in: "Open sign-in page",
                cancel_sign_in: "Cancel",
            },
            about: "About Kimini",
            settings: "Settings…",
            settings_title: "Kimini Settings",
            edit: "Edit",
            navigate: "Navigate",
            window: "Window",
            reload: "Reload",
            back: "Back",
            forward: "Forward",
            language: "Language",
            language_hint: "Language for menus and Settings. Web content follows Kimi Code Web.",
            lang_en: "English",
            lang_zh: "Chinese (Simplified)",
            launch_probing: "Looking for the local Kimi server…",
            launch_starting: "Starting the local Kimi server…",
            launch_waiting: "Still waiting for the Kimi server — try running `kimi` in your terminal.",
            launch_no_kimi: "Kimi CLI not found. Install it and this page will connect automatically:",
            launch_invalid_url: "Only local (loopback) addresses are allowed.",
            launch_manual: "Connect manually",
            launch_manual_hint: "Paste the URL printed by `kimi web`.",
            launch_connect: "Connect",
        }
    }

    pub const fn zh() -> Self {
        Self {
            native: NativeStrings {
                sessions: "会话",
                sessions_list: "Kimi Code 会话列表",
                new_session: "+ 新建",
                search_sessions: "搜索会话",
                load_more_sessions: "加载更多会话",
                show_more_conversations: "显示另外 {count} 个对话",
                show_less_conversations: "收起较早对话",
                loading_sessions: "正在加载会话…",
                untitled_session: "未命名会话",
                archived_sessions: "已归档",
                active_sessions: "进行中",
                restore_session: "恢复",
                no_archived_sessions: "没有已归档会话",
                choose_folder: "在此文件夹中创建 Kimi 会话",
                start_session: "开始一个 Kimi Code 会话",
                start_session_hint: "选择会话，或发送消息开始工作。",
                load_earlier: "加载更早消息",
                loading_earlier: "正在加载更早消息…",
                ask_placeholder: "让 Kimi 在这个项目中工作…",
                attach_file: "添加附件",
                uploading_file: "上传中",
                upload_failed: "上传失败",
                remove_attachment: "移除附件",
                attachment_image: "图片",
                attachment_video: "视频",
                attachment_file: "文件",
                browser_address: "输入网页地址",
                browser_content: "浏览器内容",
                conversation: "对话",
                message_composer: "消息输入框",
                session_runtime: "会话运行状态",
                default_model: "默认模型",
                context_label: "上下文",
                enter_hint: "Enter 发送 · Shift+Enter 换行",
                send: "发送",
                queue: "排队",
                queued_prompts: "待处理消息",
                queued_attachments: "个附件",
                steer: "立即转向",
                steering: "正在转向",
                remove_from_queue: "移出队列",
                stop: "停止",
                model: "模型",
                model_required: "发送前请选择模型。",
                thinking: "思考",
                preview_thinking: "思考预览",
                thinking_preview_hint: "选择一段思考轨迹后在此查看。",
                close_thinking: "关闭思考预览",
                permission: "权限",
                plan_on: "计划 · 开",
                plan_off: "计划 · 关",
                swarm_on: "集群 · 开",
                swarm_off: "集群 · 关",
                goal_mode_on: "目标 · 下一条",
                goal_mode_off: "目标 · 关",
                side_chat: "顺便问",
                side_chat_panel: "侧聊",
                side_chat_subtitle: "提问时不打断主任务",
                side_chat_placeholder: "快速问一个旁支问题…",
                side_chat_empty: "侧聊问答不会进入主对话。",
                side_chat_send: "发送",
                side_chat_close: "关闭",
                side_chat_opening: "正在打开侧聊…",
                side_chat_thinking: "思考",
                side_chat_stop: "停止",
                terminal: "终端",
                terminal_panel: "会话终端",
                terminal_placeholder: "运行 Shell 命令…",
                terminal_empty: "此会话没有终端",
                terminal_loading: "正在打开终端…",
                terminal_new: "新建",
                terminal_send: "运行",
                terminal_close: "关闭",
                terminal_close_tab: "关闭终端",
                terminal_running: "运行中",
                terminal_exited: "已退出",
                terminal_local: "本地",
                terminal_local_fallback: "Kimi daemon 的 PTY 当前不可用，已切换到 Kimini 本地 Rust 终端。",
                goal_label: "目标",
                goal_active: "进行中",
                goal_paused: "已暂停",
                goal_blocked: "受阻",
                start_goal: "启动目标",
                starting_goal: "正在启动目标",
                goal_start_question: "启动这个目标？Kimi 将自主朝该目标推进。",
                pause_goal: "暂停",
                resume_goal: "继续",
                cancel_goal: "取消目标",
                keep_goal: "保留目标",
                goal_cancel_question: "取消这个目标？",
                goal_cancel_detail: "目标取消后无法恢复。",
                done_when: "完成条件",
                turns: "轮",
                tokens: "tokens",
                fork: "分叉",
                session_actions: "会话操作",
                rename_session: "重命名",
                save: "保存",
                compact: "压缩上下文",
                compact_question: "压缩这个会话的上下文？",
                compact_detail: "Kimi 会先总结较早的上下文，再继续工作。",
                undo: "撤销上一轮",
                undo_question: "撤销上一轮对话？",
                undo_detail: "此会话中最新一轮用户消息和助手回复将被移除。",
                export_session: "导出诊断包",
                exporting: "正在导出会话…",
                archive: "归档",
                archive_question: "归档这个会话？",
                archive_detail: "之后可在 Kimi Code Web 中恢复。",
                cancel: "取消",
                browser: "浏览器",
                close_browser: "关闭浏览器",
                connected: "已连接",
                connecting: "正在连接 Kimi Code…",
                working: "工作中",
                retry_connection: "重新连接",
                open_web_fallback: "打开 Kimini Web",
                you: "你",
                kimi: "Kimi",
                tool: "工具",
                system: "系统",
                tool_result: "工具结果",
                tool_read: "读取",
                tool_edit: "编辑",
                tool_write: "写入",
                tool_command: "命令",
                tool_search: "搜索",
                show_tool_details: "展开详情",
                hide_tool_details: "收起详情",
                approve: "允许",
                approve_once: "仅允许一次",
                approve_session: "本会话内允许",
                reject: "拒绝",
                approval_required: "需要授权",
                question_required: "需要回答",
                submit_answers: "提交答案",
                dismiss_question: "忽略",
                other_answer: "其他",
                recommended: "推荐",
                switch_language: "EN",
                tasks_panel: "后台任务与子代理",
                tasks: "任务",
                close_tasks: "关闭",
                refresh_tasks: "刷新",
                tasks_loading: "正在加载任务…",
                no_tasks: "此会话没有后台任务",
                cancel_task: "取消任务",
                subagent_task: "子代理",
                shell_task: "命令",
                tool_task: "工具",
                task_running: "运行中",
                task_completed: "已完成",
                task_failed: "失败",
                task_cancelled: "已取消",
                files_panel: "工作区文件与预览",
                files: "文件",
                close_files: "关闭",
                refresh_files: "刷新",
                search_files: "搜索文件，按 Enter 执行",
                workspace_files: "工作区文件",
                loading_files: "正在加载文件…",
                no_files: "没有文件",
                file_preview: "文件预览",
                loading_file: "正在加载文件…",
                select_file: "选择文件后可在此预览。",
                source: "源码",
                diff: "差异",
                binary_file: "二进制文件",
                skills_panel: "会话技能",
                workspace_tools: "工作区",
                skills: "技能",
                close_skills: "关闭",
                refresh_skills: "刷新",
                skills_loading: "正在加载技能…",
                no_skills: "此会话没有可用技能",
                activate_skill: "激活",
                activating_skill: "激活中…",
                skill_activated: "已激活",
                slash_only: "仅斜杠命令",
                skill_activation_unacknowledged: "服务端未确认技能激活",
                skill_attachments_unsupported: "激活技能前请先移除附件",
                command_attachments_unsupported: "执行命令前请先移除附件",
                built_in_command: "内置命令",
                slash_commands: "命令与技能",
                auth_panel: "Kimi 认证",
                authentication: "认证",
                close_auth: "关闭",
                refresh_auth: "刷新",
                auth_status: "认证状态",
                auth_ready: "已就绪",
                auth_required: "请登录",
                auth_working: "处理中…",
                auth_failed: "登录失败",
                auth_denied: "登录请求被拒绝",
                auth_expired: "登录验证码已过期",
                auth_cancelled: "登录已取消",
                providers: "服务商",
                sign_in: "登录 Kimi",
                sign_in_hint: "连接服务商后即可开始编程会话。",
                sign_out: "退出登录",
                finish_sign_in: "完成登录",
                device_code: "设备验证码",
                open_sign_in: "打开登录页面",
                cancel_sign_in: "取消",
            },
            about: "关于 Kimini",
            settings: "设置…",
            settings_title: "Kimini 设置",
            edit: "编辑",
            navigate: "导航",
            window: "窗口",
            reload: "重新加载",
            back: "后退",
            forward: "前进",
            language: "语言",
            language_hint: "用于菜单与设置窗口。网页内容仍由 Kimi Code Web 决定。",
            lang_en: "English",
            lang_zh: "简体中文",
            launch_probing: "正在查找本地 Kimi 服务…",
            launch_starting: "正在启动本地 Kimi 服务…",
            launch_waiting: "仍在等待 Kimi 服务就绪，可尝试在终端运行 `kimi`。",
            launch_no_kimi: "未找到 Kimi CLI。安装后本页会自动连接：",
            launch_invalid_url: "仅支持本机 (localhost) 地址。",
            launch_manual: "手动连接",
            launch_manual_hint: "粘贴 `kimi web` 输出的地址。",
            launch_connect: "连接",
        }
    }
}

fn config_dir() -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = env::var_os("HOME")?;
        Some(
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Kimini"),
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
            return Some(PathBuf::from(xdg).join("kimini"));
        }
        let home = env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".config").join("kimini"))
    }
}

fn preference_path() -> Option<PathBuf> {
    Some(config_dir()?.join("lang"))
}

pub fn load_preference() -> Option<Lang> {
    let path = preference_path()?;
    let raw = fs::read_to_string(path).ok()?;
    Lang::parse_tag(raw.trim())
}

pub fn save_preference(lang: Lang) -> std::io::Result<()> {
    let dir = config_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "no config directory"))?;
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("lang"), lang.code())
}

#[cfg(target_os = "macos")]
fn apple_locale() -> Option<Lang> {
    let output = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout);
    Lang::parse_tag(s.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_english_tags() {
        assert_eq!(Lang::parse_tag("en"), Some(Lang::En));
        assert_eq!(Lang::parse_tag("en_US"), Some(Lang::En));
        assert_eq!(Lang::parse_tag("en-US.UTF-8"), Some(Lang::En));
    }

    #[test]
    fn parse_chinese_tags() {
        assert_eq!(Lang::parse_tag("zh"), Some(Lang::Zh));
        assert_eq!(Lang::parse_tag("zh_CN"), Some(Lang::Zh));
        assert_eq!(Lang::parse_tag("zh-Hans"), Some(Lang::Zh));
    }

    #[test]
    fn parse_unknown_is_none() {
        assert_eq!(Lang::parse_tag(""), None);
        assert_eq!(Lang::parse_tag("ja"), None);
    }

    #[test]
    fn zh_table_has_cjk_en_table_does_not() {
        let en = Strings::en().reload;
        let zh = Strings::zh().reload;
        assert!(en.is_ascii());
        assert!(!zh.is_ascii());
    }
}
