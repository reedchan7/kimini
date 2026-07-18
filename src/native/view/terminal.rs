use std::path::Path;

use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::{StyledExt, input::Input, scroll::ScrollableElement};

use crate::protocol::TerminalStatus;

use super::super::app::Shell;
use super::super::theme::*;
use super::accessible_input::accessible_input;
use super::panel::panel_button;

const TAB_BAR_HEIGHT: f32 = 36.0;

impl Shell {
    pub(super) fn terminal_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let session_id = self
            .model
            .active_session()
            .map(|session| session.id.as_str())
            .unwrap_or_default();
        let tabs = self
            .terminals
            .tabs(session_id)
            .iter()
            .map(|tab| {
                (
                    tab.terminal.id.clone(),
                    shell_label(&tab.terminal.shell),
                    tab.terminal.status,
                    tab.is_local(),
                )
            })
            .collect::<Vec<_>>();
        let active = self.terminals.active(session_id);
        let active_id = active.map(|tab| tab.terminal.id.as_str());
        let output = active.map(|tab| tab.output()).unwrap_or_default();
        let status = active.map(|tab| tab.terminal.status);
        let exit_code = active.and_then(|tab| tab.terminal.exit_code);
        let loading = self.terminals.is_loading(session_id);
        let notice = self.terminals.notice(session_id).map(str::to_owned);
        let error = self.terminals.error(session_id).map(str::to_owned);

        div()
            .id("terminal-panel")
            .role(Role::Complementary)
            .aria_label(self.strings.native.terminal_panel)
            .w(px(TERMINAL_PANEL_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(self.terminal_header(cx))
            .child(self.terminal_tabs(&tabs, active_id, cx))
            .child(
                div()
                    .id("terminal-output")
                    .role(Role::Log)
                    .aria_label(self.strings.native.terminal_panel)
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .track_scroll(&self.terminal_scroll)
                    .overflow_y_scrollbar()
                    .bg(rgb(0x171716))
                    .p_3()
                    .when(loading && active.is_none(), |surface| {
                        surface.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xa7a7a2))
                                .child(self.strings.native.terminal_loading),
                        )
                    })
                    .when(!loading && active.is_none(), |surface| {
                        surface.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xa7a7a2))
                                .child(self.strings.native.terminal_empty),
                        )
                    })
                    .when(!output.is_empty(), |surface| {
                        surface.child(
                            div()
                                .id("terminal-output-text")
                                .w_full()
                                .font_family("SF Mono")
                                .text_xs()
                                .text_color(rgb(0xe8e8e3))
                                .child(output.clone()),
                        )
                    })
                    .when_some(notice, |surface, notice| {
                        surface.child(
                            div()
                                .mt_3()
                                .rounded_md()
                                .border_1()
                                .border_color(rgb(0x4a4431))
                                .bg(rgb(0x242116))
                                .px_2()
                                .py_1()
                                .text_xs()
                                .text_color(rgb(0xd8c98e))
                                .child(notice),
                        )
                    })
                    .when_some(error, |surface, error| {
                        surface.child(
                            div()
                                .mt_3()
                                .text_xs()
                                .text_color(rgb(0xff8a80))
                                .child(error),
                        )
                    })
                    .when(status == Some(TerminalStatus::Exited), |surface| {
                        surface.child(div().mt_3().text_xs().text_color(rgb(0xa7a7a2)).child(
                            match exit_code {
                                Some(code) => {
                                    format!("{} · {code}", self.strings.native.terminal_exited)
                                }
                                None => self.strings.native.terminal_exited.into(),
                            },
                        ))
                    }),
            )
            .child(self.terminal_composer(status, cx))
    }

    fn terminal_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(48.0))
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .justify_between()
            .px_3()
            .border_b_1()
            .border_color(rgb(BORDER))
            .child(
                div()
                    .text_sm()
                    .font_semibold()
                    .child(self.strings.native.terminal),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        panel_button(self.strings.native.terminal_new, "new-terminal")
                            .on_click(cx.listener(|this, _, _, cx| this.create_terminal(cx))),
                    )
                    .child(
                        panel_button(self.strings.native.terminal_close, "close-terminal-panel")
                            .on_click(
                                cx.listener(|this, _, window, cx| this.toggle_terminal(window, cx)),
                            ),
                    ),
            )
    }

    fn terminal_tabs(
        &self,
        tabs: &[(String, String, TerminalStatus, bool)],
        active_id: Option<&str>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .w_full()
            .h(px(TAB_BAR_HEIGHT))
            .flex_none()
            .flex()
            .items_center()
            .gap_1()
            .overflow_x_scrollbar()
            .border_b_1()
            .border_color(rgb(BORDER))
            .px_2()
            .py_1()
            .children(
                tabs.iter()
                    .enumerate()
                    .map(|(index, (id, shell, status, local))| {
                        let terminal_id = id.clone();
                        let selected = active_id == Some(id.as_str());
                        div()
                            .id(("terminal-tab", index))
                            .focusable()
                            .tab_stop(true)
                            .role(Role::Tab)
                            .aria_selected(selected)
                            .aria_label(shell.clone())
                            .cursor_pointer()
                            .rounded_md()
                            .px_2()
                            .py_1()
                            .text_xs()
                            .when(selected, |tab| tab.bg(rgb(SURFACE_ACTIVE)).font_semibold())
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.select_terminal(terminal_id.clone(), cx)
                            }))
                            .child(format!(
                                "{shell} · {}{}",
                                if *local {
                                    format!("{} · ", self.strings.native.terminal_local)
                                } else {
                                    String::new()
                                },
                                if *status == TerminalStatus::Running {
                                    self.strings.native.terminal_running
                                } else {
                                    self.strings.native.terminal_exited
                                }
                            ))
                    }),
            )
            .into_any_element()
    }

    fn terminal_composer(
        &self,
        status: Option<TerminalStatus>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let running = status == Some(TerminalStatus::Running);
        div()
            .w_full()
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .border_t_1()
            .border_color(rgb(BORDER))
            .p_3()
            .when(running, |composer| {
                composer
                    .child(
                        accessible_input(
                            "terminal-command-input",
                            &self.terminal_input,
                            Role::TextInput,
                            self.strings.native.terminal_placeholder,
                            self.strings.native.terminal_placeholder,
                            Input::new(&self.terminal_input),
                            cx,
                        )
                        .flex_1()
                        .min_w_0(),
                    )
                    .child(
                        panel_button(self.strings.native.terminal_send, "send-terminal-command")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.send_terminal_command(window, cx)
                            })),
                    )
                    .child(
                        panel_button(self.strings.native.terminal_close_tab, "close-terminal-tab")
                            .on_click(cx.listener(|this, _, _, cx| this.close_active_terminal(cx))),
                    )
            })
            .when(status == Some(TerminalStatus::Exited), |composer| {
                composer
                    .justify_end()
                    .child(
                        panel_button(self.strings.native.terminal_new, "restart-terminal")
                            .on_click(cx.listener(|this, _, _, cx| this.create_terminal(cx))),
                    )
                    .child(
                        panel_button(
                            self.strings.native.terminal_close_tab,
                            "remove-terminal-tab",
                        )
                        .on_click(cx.listener(|this, _, _, cx| this.close_active_terminal(cx))),
                    )
            })
    }
}

fn shell_label(shell: &str) -> String {
    Path::new(shell)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(shell)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_labels_use_the_executable_name() {
        assert_eq!(shell_label("/bin/zsh"), "zsh");
        assert_eq!(shell_label("custom-shell"), "custom-shell");
    }
}
