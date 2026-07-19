use gpui::{AnyElement, Context, IntoElement, Role, Window, div, prelude::*, px, relative, rgba};
use gpui_component::{Icon, IconName, Sizable as _, input::Input, scroll::ScrollableElement};

use crate::protocol::Session;

use super::super::app::{Shell, UtilityPanel};
use super::super::session_list::{display_title, relative_time, workspace_label};
use super::super::theme::*;
use super::accessible_input::accessible_input;

const MAX_SEARCH_RESULTS: usize = 200;

#[derive(Clone)]
struct SessionSearchHit {
    id: String,
    title: String,
    cwd: String,
    last_prompt: String,
    updated_at: String,
    active: bool,
}

impl Shell {
    pub(in crate::native) fn open_session_search(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.utility_panel == Some(UtilityPanel::Auth) {
            self.utility_panel = None;
        }
        self.session_search_open = true;
        self.session_search_selected = 0;
        self.session_search.update(cx, |input, cx| {
            input.set_value("", window, cx);
            input.focus(window, cx);
        });
        self.load_remaining_sessions_for_search(cx);
        cx.notify();
    }

    pub(in crate::native) fn close_session_search(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.session_search_open = false;
        self.session_search_selected = 0;
        self.session_search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.composer
            .update(cx, |input, cx| input.focus(window, cx));
        cx.notify();
    }

    pub(in crate::native) fn move_session_search(&mut self, delta: isize, cx: &mut Context<Self>) {
        let count = self.session_search_hits(cx).len();
        if count == 0 {
            self.session_search_selected = 0;
            return;
        }
        self.session_search_selected = (self.session_search_selected as isize + delta)
            .clamp(0, count.saturating_sub(1) as isize)
            as usize;
        cx.notify();
    }

    pub(in crate::native) fn activate_session_search_selection(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(hit) = self
            .session_search_hits(cx)
            .get(self.session_search_selected)
            .cloned()
        else {
            return;
        };
        self.session_search_open = false;
        self.session_search_selected = 0;
        self.session_search
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.select_session(hit.id, window, cx);
    }

    fn session_search_hits(&self, cx: &Context<Self>) -> Vec<SessionSearchHit> {
        let query = self.session_search.read(cx).value().trim().to_owned();
        let normalized_query = query.to_lowercase();
        let active_id = self
            .new_session_draft
            .is_none()
            .then(|| {
                self.model
                    .active_session()
                    .map(|session| session.id.as_str())
            })
            .flatten();
        self.model
            .sessions()
            .iter()
            .filter(|session| session_matches(session, &normalized_query))
            .take(MAX_SEARCH_RESULTS)
            .map(|session| SessionSearchHit {
                id: session.id.clone(),
                title: display_title(&session.title),
                cwd: session.metadata.cwd.clone(),
                last_prompt: session
                    .last_prompt
                    .as_deref()
                    .map(|prompt| search_snippet(prompt, &query))
                    .unwrap_or_default(),
                updated_at: session.updated_at.clone(),
                active: active_id == Some(session.id.as_str()),
            })
            .collect()
    }

    pub(super) fn session_search_overlay(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let hits = self.session_search_hits(cx);
        let result_count = hits.len();
        let selected = self
            .session_search_selected
            .min(result_count.saturating_sub(1));

        div()
            .id("session-search-overlay")
            .key_context("SessionSearch")
            .role(Role::Dialog)
            .aria_label(self.strings.native.search_sessions)
            .absolute()
            .inset_0()
            .p_4()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000055))
            .child(
                div()
                    .w(px(640.0))
                    .max_w(relative(0.94))
                    .h(px(680.0))
                    .max_h(relative(0.92))
                    .flex()
                    .flex_col()
                    .overflow_hidden()
                    .rounded_xl()
                    .border_1()
                    .border_color(theme_rgb(BORDER_STRONG))
                    .bg(theme_rgb(SURFACE))
                    .shadow_xl()
                    .child(self.session_search_header(cx))
                    .child(
                        div()
                            .id("session-search-results")
                            .role(Role::List)
                            .aria_label(self.strings.native.sessions_list)
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scrollbar()
                            .p_2()
                            .when(hits.is_empty(), |list| {
                                list.child(
                                    div()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(font_px(13.0))
                                        .text_color(theme_rgb(TEXT_MUTED))
                                        .child(self.strings.native.search_no_results),
                                )
                            })
                            .children(hits.into_iter().enumerate().map(|(index, hit)| {
                                self.session_search_result_row(
                                    hit,
                                    index,
                                    result_count,
                                    index == selected,
                                    cx,
                                )
                            })),
                    )
                    .child(
                        div()
                            .h(px(54.0))
                            .flex_none()
                            .flex()
                            .items_center()
                            .justify_end()
                            .px(px(22.0))
                            .border_t_1()
                            .border_color(theme_rgb(BORDER))
                            .text_size(font_px(10.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(self.strings.native.search_navigation_hint),
                    ),
            )
    }

    fn session_search_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(54.0))
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .border_b_1()
            .border_color(theme_rgb(BORDER))
            .child(
                Icon::new(IconName::Search)
                    .small()
                    .text_color(theme_rgb(TEXT_MUTED)),
            )
            .child(
                accessible_input(
                    "session-search-input",
                    &self.session_search,
                    Role::SearchInput,
                    self.strings.native.search_sessions,
                    self.strings.native.search_sessions,
                    Input::new(&self.session_search)
                        .appearance(false)
                        .bordered(false)
                        .focus_bordered(false),
                    cx,
                )
                .flex_1()
                .min_w_0(),
            )
            .child(
                div()
                    .id("close-session-search")
                    .focusable()
                    .tab_stop(true)
                    .role(Role::Button)
                    .aria_label(self.strings.native.close_auth)
                    .cursor_pointer()
                    .size(px(28.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .text_color(theme_rgb(TEXT_MUTED))
                    .hover(|item| {
                        item.bg(theme_rgb(SURFACE_ACTIVE))
                            .text_color(theme_rgb(TEXT))
                    })
                    .on_click(
                        cx.listener(|this, _, window, cx| this.close_session_search(window, cx)),
                    )
                    .child(Icon::new(IconName::Close).xsmall()),
            )
    }

    fn session_search_result_row(
        &self,
        hit: SessionSearchHit,
        index: usize,
        result_count: usize,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let session_id = hit.id.clone();
        let title = if hit.title.is_empty() {
            self.strings.native.untitled_session.to_owned()
        } else {
            hit.title
        };
        let workspace = workspace_label(&hit.cwd);
        div()
            .id(("session-search-result", index))
            .focusable()
            .tab_stop(true)
            .role(Role::ListItem)
            .aria_label(format!("{title} — {workspace}"))
            .aria_selected(selected)
            .aria_position_in_set(index + 1)
            .aria_size_of_set(result_count)
            .cursor_pointer()
            .rounded_lg()
            .px_3()
            .py_2()
            .when(selected, |row| row.bg(theme_rgb(SURFACE_ACTIVE)))
            .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
            .on_click(cx.listener(move |this, _, window, cx| {
                this.session_search_open = false;
                this.session_search_selected = 0;
                this.session_search
                    .update(cx, |input, cx| input.set_value("", window, cx));
                this.select_session(session_id.clone(), window, cx);
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_size(font_px(10.0))
                    .text_color(theme_rgb(TEXT_MUTED))
                    .child(
                        Icon::new(IconName::FolderClosed)
                            .xsmall()
                            .text_color(theme_rgb(TEXT_MUTED)),
                    )
                    .child(div().min_w_0().flex_1().line_clamp(1).child(workspace))
                    .child(div().flex_none().text_color(theme_rgb(TEXT_MUTED)).child(
                        relative_time(&hit.updated_at, self.strings.native.session_just_now),
                    )),
            )
            .child(
                div()
                    .min_w_0()
                    .text_size(font_px(12.0))
                    .text_color(theme_rgb(if hit.active { ACCENT } else { TEXT }))
                    .line_clamp(1)
                    .child(title),
            )
            .when(!hit.last_prompt.is_empty(), |row| {
                row.child(
                    div()
                        .min_w_0()
                        .text_size(font_px(11.0))
                        .text_color(theme_rgb(TEXT_MUTED))
                        .line_clamp(1)
                        .child(hit.last_prompt),
                )
            })
            .into_any_element()
    }
}

fn session_matches(session: &Session, query: &str) -> bool {
    query.is_empty()
        || session.title.to_lowercase().contains(query)
        || workspace_label(&session.metadata.cwd)
            .to_lowercase()
            .contains(query)
        || session
            .last_prompt
            .as_deref()
            .is_some_and(|prompt| prompt.to_lowercase().contains(query))
}

fn search_snippet(prompt: &str, query: &str) -> String {
    const RADIUS: usize = 40;
    let flat = prompt.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.is_empty() {
        return String::new();
    }
    let flat_chars = flat.chars().collect::<Vec<_>>();
    let normalized_query = query.trim().to_lowercase();
    let flat_lower = flat.to_lowercase();
    let Some(byte_index) = (!normalized_query.is_empty())
        .then(|| flat_lower.find(&normalized_query))
        .flatten()
    else {
        let mut head = flat_chars.iter().take(RADIUS * 2).collect::<String>();
        if flat_chars.len() > RADIUS * 2 {
            head.push('…');
        }
        return head;
    };
    let match_start = flat_lower[..byte_index].chars().count();
    let match_len = normalized_query.chars().count();
    let start = match_start.saturating_sub(RADIUS);
    let end = (match_start + match_len + RADIUS).min(flat_chars.len());
    let mut snippet = String::new();
    if start > 0 {
        snippet.push('…');
    }
    snippet.extend(flat_chars[start..end].iter());
    if end < flat_chars.len() {
        snippet.push('…');
    }
    snippet
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> Session {
        serde_json::from_value(serde_json::json!({
            "id": "session-1",
            "workspace_id": "workspace-1",
            "title": "Kimini native parity",
            "created_at": "2026-07-19T00:00:00Z",
            "updated_at": "2026-07-19T00:00:00Z",
            "busy": false,
            "last_prompt": "Match the Web experience",
            "metadata": { "cwd": "/workspace/kimini" },
            "agent_config": { "model": "kimi" },
            "usage": {
                "input_tokens": 0,
                "output_tokens": 0,
                "cache_read_tokens": 0,
                "cache_creation_tokens": 0,
                "total_cost_usd": 0.0,
                "context_tokens": 0,
                "context_limit": 0,
                "turn_count": 0
            },
            "message_count": 1,
            "last_seq": 1
        }))
        .expect("session fixture")
    }

    #[test]
    fn search_matches_web_fields_case_insensitively() {
        let session = session();
        assert!(session_matches(&session, "native"));
        assert!(session_matches(
            &session,
            "WEB EXPERIENCE".to_lowercase().as_str()
        ));
        assert!(session_matches(&session, "kimini"));
        assert!(!session_matches(&session, "unrelated"));
    }

    #[test]
    fn search_snippet_uses_the_first_non_empty_line_and_truncates() {
        assert_eq!(
            search_snippet("\n  first line  \nsecond", ""),
            "first line second"
        );
        assert_eq!(search_snippet(&"a".repeat(81), "").chars().count(), 81);
        assert!(search_snippet(&"a".repeat(81), "").ends_with('…'));
        assert_eq!(
            search_snippet("prefix target suffix", "TARGET"),
            "prefix target suffix"
        );
    }
}
