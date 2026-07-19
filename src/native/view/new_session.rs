use std::path::Path;

use gpui::{Anchor, AnyElement, Context, IntoElement, Role, Window, div, prelude::*, px};
use gpui_component::{
    Icon, IconName, Sizable as _, StyledExt,
    button::{Button, ButtonVariants},
    popover::Popover,
};

use crate::native::{app::Shell, session_list::workspace_label, theme::*};
use crate::protocol::Workspace;

impl Shell {
    pub(super) fn new_session_landing(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        if let Some(parts) = self
            .new_session_draft
            .as_ref()
            .filter(|draft| draft.submitting)
            .map(|draft| draft.submitted_parts.clone())
        {
            return div()
                .id("new-session-pending")
                .role(Role::Document)
                .aria_label(self.strings.native.conversation)
                .flex_1()
                .min_h_0()
                .w_full()
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .w_full()
                        .max_w(px(CONTENT_WIDTH))
                        .flex_1()
                        .min_h_0()
                        .pt_4()
                        .child(self.pending_prompt_preview(&parts, cx)),
                )
                .child(self.composer(window, cx))
                .into_any_element();
        }
        let cwd = self
            .new_session_draft
            .as_ref()
            .map(|draft| draft.cwd.as_str())
            .unwrap_or_default();
        let workspaces = self.model.workspaces().to_vec();
        let workspace_name = workspaces
            .iter()
            .find(|workspace| workspace.root == cwd)
            .map(|workspace| workspace.name.clone())
            .unwrap_or_else(|| workspace_label(cwd));
        let strings = self.strings.native;
        let workspace_trigger = Button::new("draft-workspace-button")
            .small()
            .secondary()
            .icon(IconName::Folder)
            .label(workspace_name)
            .dropdown_caret(true);
        let workspace_picker = Popover::new("draft-workspace-popover")
            .anchor(Anchor::BottomLeft)
            .open(self.draft_workspace_menu_open)
            .on_open_change(cx.listener(|this, open, _, cx| {
                this.draft_workspace_menu_open = *open;
                if !*open {
                    this.draft_workspace_show_all = false;
                }
                cx.notify();
            }))
            .trigger(workspace_trigger)
            .w(px(300.0))
            .max_h(px(430.0))
            .p(px(5.0))
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(BORDER))
            .bg(theme_rgb(SURFACE))
            .shadow_sm()
            .child(self.draft_workspace_menu(workspaces, cwd, cx));

        div()
            .id("new-session-landing")
            .role(Role::Group)
            .aria_label(strings.start_session)
            .flex_1()
            .min_h_0()
            .w_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .pb(px(56.0))
            .child(
                div()
                    .w_full()
                    .max_w(px(CONTENT_WIDTH))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .id("new-session-heading")
                            .role(Role::Heading)
                            .aria_level(2)
                            .aria_label(strings.start_session)
                            .text_size(font_px(26.0))
                            .font_semibold()
                            .text_color(theme_rgb(TEXT))
                            .child(strings.start_session),
                    )
                    .child(
                        div()
                            .mt_3()
                            .text_size(font_px(12.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(strings.start_session_hint),
                    )
                    .child(div().mt_4().child(workspace_picker))
                    .child(div().mt_4().w_full().child(self.composer(window, cx))),
            )
            .into_any_element()
    }

    fn draft_workspace_menu(
        &self,
        workspaces: Vec<Workspace>,
        selected_cwd: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let total = workspaces.len();
        let visible_count = if self.draft_workspace_show_all {
            total
        } else {
            total.min(5)
        };
        let remaining = total.saturating_sub(visible_count);
        let strings = self.strings.native;

        div()
            .id("draft-workspace-menu")
            .role(Role::Menu)
            .max_h(px(420.0))
            .overflow_y_scroll()
            .children(workspaces.into_iter().take(visible_count).enumerate().map(
                |(index, workspace)| {
                    let active = workspace.root == selected_cwd;
                    let cwd_for_click = workspace.root.clone();
                    div()
                        .id(("draft-workspace-option", index))
                        .focusable()
                        .tab_stop(true)
                        .role(Role::MenuItem)
                        .aria_label(format!(
                            "{} {}",
                            workspace.name,
                            compact_workspace_path(&workspace.root)
                        ))
                        .cursor_pointer()
                        .rounded_md()
                        .px_2()
                        .py_2()
                        .when(active, |row| row.bg(theme_rgb(ACCENT_SOFT)))
                        .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.draft_workspace_menu_open = false;
                            this.draft_workspace_show_all = false;
                            this.set_draft_workspace(cwd_for_click.clone(), cx);
                        }))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_size(font_px(12.0))
                                        .font_medium()
                                        .child(workspace.name),
                                )
                                .child(
                                    div()
                                        .text_size(font_px(10.0))
                                        .text_color(theme_rgb(TEXT_MUTED))
                                        .child(compact_workspace_path(&workspace.root)),
                                ),
                        )
                },
            ))
            .when(remaining > 0, |menu| {
                menu.child(
                    div()
                        .id("show-more-draft-workspaces")
                        .focusable()
                        .tab_stop(true)
                        .role(Role::MenuItem)
                        .aria_label(format!("{} ({remaining})", strings.more_workspaces))
                        .cursor_pointer()
                        .rounded_md()
                        .px_2()
                        .py_2()
                        .text_size(font_px(12.0))
                        .font_medium()
                        .text_color(theme_rgb(TEXT_SECONDARY))
                        .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.draft_workspace_show_all = true;
                            cx.notify();
                        }))
                        .child(format!("{} ({remaining})", strings.more_workspaces)),
                )
            })
            .child(div().my_1().border_t_1().border_color(theme_rgb(BORDER)))
            .child(
                div()
                    .id("choose-new-draft-workspace")
                    .focusable()
                    .tab_stop(true)
                    .role(Role::MenuItem)
                    .aria_label(strings.new_workspace)
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .gap_2()
                    .rounded_md()
                    .px_2()
                    .py_2()
                    .text_size(font_px(12.0))
                    .hover(|row| row.bg(theme_rgb(SURFACE_ACTIVE)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.draft_workspace_menu_open = false;
                        this.draft_workspace_show_all = false;
                        this.choose_draft_workspace(cx);
                    }))
                    .child(Icon::new(IconName::Plus).xsmall())
                    .child(strings.new_workspace),
            )
    }
}

fn compact_workspace_path(path: &str) -> String {
    dirs::home_dir()
        .and_then(|home| Path::new(path).strip_prefix(home).ok().map(Path::to_owned))
        .map(|suffix| {
            if suffix.as_os_str().is_empty() {
                "~".to_owned()
            } else {
                format!("~{}{}", std::path::MAIN_SEPARATOR, suffix.display())
            }
        })
        .unwrap_or_else(|| path.to_owned())
}
