use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px};
use gpui_component::{
    Icon, IconName, Sizable as _, StyledExt, scroll::ScrollableElement, text::TextView,
};

use super::super::app::Shell;
use super::super::presentation::ToolCard;
use super::super::theme::*;

impl Shell {
    pub(super) fn tool_card(
        &self,
        key: usize,
        tool: &ToolCard,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.tool_group(key, &[tool], cx)
    }

    pub(super) fn tool_group(
        &self,
        key: usize,
        tools: &[&ToolCard],
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let running = tools.iter().any(|tool| tool.running);
        let failed = tools.iter().any(|tool| tool.is_error);
        let status = if running {
            self.strings.native.task_running
        } else if failed {
            self.strings.native.task_failed
        } else {
            self.strings.native.task_completed
        };
        let rows = tools
            .iter()
            .enumerate()
            .map(|(index, tool)| self.tool_row(key + index, tool, index > 0, cx))
            .collect::<Vec<_>>();

        div()
            .id(("tool-group", key))
            .mt_2()
            .rounded_lg()
            .border_1()
            .border_color(theme_rgb(if failed { ERROR } else { BORDER }))
            .bg(theme_rgb(SURFACE))
            .overflow_hidden()
            .child(
                div()
                    .h(px(32.0))
                    .px_3()
                    .flex()
                    .items_center()
                    .gap_2()
                    .bg(theme_rgb(SURFACE_SUBTLE))
                    .child(status_mark(running, failed))
                    .child(
                        div()
                            .text_size(font_px(11.0))
                            .font_semibold()
                            .child(format!("{} {}", tools.len(), self.strings.native.tool)),
                    )
                    .child(
                        div()
                            .text_size(font_px(11.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(format!("· {status}")),
                    ),
            )
            .children(rows)
            .into_any_element()
    }

    fn tool_row(
        &self,
        key: usize,
        tool: &ToolCard,
        separated: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let expanded = self.expanded_tools.contains(&tool.id);
        let expandable = tool.has_details();
        let tool_id = tool.id.clone();
        let label = tool
            .name
            .as_deref()
            .map(|name| tool_label(name, self.strings.native))
            .unwrap_or_else(|| self.strings.native.tool_result.to_owned());
        let status = if tool.running {
            self.strings.native.task_running
        } else if tool.is_error {
            self.strings.native.task_failed
        } else {
            self.strings.native.task_completed
        };
        let action_label = if expanded {
            self.strings.native.hide_tool_details
        } else {
            self.strings.native.show_tool_details
        };
        let header_label = if expandable {
            format!("{label}: {} · {status} · {action_label}", tool.summary)
        } else {
            format!("{label}: {} · {status}", tool.summary)
        };

        div()
            .when(separated, |row| {
                row.border_t_1().border_color(theme_rgb(BORDER))
            })
            .child(
                div()
                    .id(("tool-card-header", key))
                    .role(if expandable {
                        Role::Button
                    } else {
                        Role::Group
                    })
                    .aria_label(header_label)
                    .aria_expanded(expanded)
                    .when(expandable, |header| {
                        header
                            .focusable()
                            .tab_stop(true)
                            .cursor_pointer()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                if !this.expanded_tools.insert(tool_id.clone()) {
                                    this.expanded_tools.remove(&tool_id);
                                }
                                cx.notify();
                            }))
                            .hover(|item| item.bg(theme_rgb(SURFACE_SUBTLE)))
                    })
                    .min_h(px(34.0))
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_1()
                    .child(
                        Icon::new(tool_icon(tool.name.as_deref()))
                            .xsmall()
                            .text_color(theme_rgb(TEXT_MUTED)),
                    )
                    .child(
                        div()
                            .text_size(font_px(11.0))
                            .font_semibold()
                            .text_color(theme_rgb(TEXT_SECONDARY))
                            .child(label),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .text_size(font_px(11.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .line_clamp(1)
                            .child(tool.summary.clone()),
                    )
                    .when_some(tool.diff.as_ref(), |header, diff| {
                        header.child(
                            div()
                                .text_size(font_px(10.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(format!("+{} −{}", diff.added, diff.removed)),
                        )
                    })
                    .child(tool_result_mark(tool.running, tool.is_error))
                    .when(expandable, |header| {
                        header.child(
                            Icon::new(if expanded {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronRight
                            })
                            .xsmall()
                            .text_color(theme_rgb(TEXT_MUTED)),
                        )
                    }),
            )
            .when(expanded, |card| card.child(self.tool_details(key, tool)))
            .into_any_element()
    }

    fn tool_details(&self, key: usize, tool: &ToolCard) -> impl IntoElement {
        div()
            .border_t_1()
            .border_color(theme_rgb(BORDER))
            .max_h(px(420.0))
            .overflow_y_scrollbar()
            .bg(theme_rgb(SURFACE_SUBTLE))
            .p_3()
            .when_some(tool.diff.as_ref(), |details, diff| {
                details.child(
                    TextView::markdown(("tool-diff", key), diff.markdown())
                        .selectable(true)
                        .text_size(font_px(12.0)),
                )
            })
            .when_some(tool.detail.clone(), |details, detail| {
                details.child(
                    TextView::markdown(("tool-input", key), detail)
                        .selectable(true)
                        .text_size(font_px(12.0)),
                )
            })
            .when_some(tool.output.clone(), |details, output| {
                details.child(
                    div()
                        .when(
                            tool.diff.is_some() || tool.detail.is_some(),
                            |output_block| {
                                output_block
                                    .mt_3()
                                    .pt_3()
                                    .border_t_1()
                                    .border_color(theme_rgb(BORDER))
                            },
                        )
                        .child(
                            TextView::markdown(("tool-output", key), output)
                                .selectable(true)
                                .text_size(font_px(12.0)),
                        ),
                )
            })
    }
}

fn tool_icon(name: Option<&str>) -> IconName {
    let normalized = name
        .unwrap_or_default()
        .trim()
        .to_lowercase()
        .replace([' ', '-'], "_");
    match normalized.as_str() {
        "read" | "write" | "edit" | "multiedit" | "multi_edit" => IconName::File,
        "grep" | "rg" | "ripgrep" | "search" | "glob" => IconName::Search,
        "bash" | "shell" | "run" | "exec" => IconName::SquareTerminal,
        _ => IconName::Asterisk,
    }
}

fn tool_label(name: &str, strings: crate::i18n::NativeStrings) -> String {
    let normalized = name.trim().to_lowercase().replace([' ', '-'], "_");
    match normalized.as_str() {
        "read" => strings.tool_read.into(),
        "edit" | "multiedit" | "multi_edit" => strings.tool_edit.into(),
        "write" => strings.tool_write.into(),
        "bash" | "shell" | "run" | "exec" => strings.tool_command.into(),
        "grep" | "rg" | "ripgrep" | "search" | "glob" => strings.tool_search.into(),
        _ => name.to_owned(),
    }
}

fn status_mark(running: bool, error: bool) -> AnyElement {
    if running {
        div()
            .size(px(7.0))
            .rounded_full()
            .bg(theme_rgb(ACCENT))
            .into_any_element()
    } else {
        Icon::new(if error {
            IconName::CircleX
        } else {
            IconName::CircleCheck
        })
        .xsmall()
        .text_color(theme_rgb(if error { ERROR } else { SUCCESS }))
        .into_any_element()
    }
}

fn tool_result_mark(running: bool, error: bool) -> AnyElement {
    if running {
        div()
            .size(px(6.0))
            .rounded_full()
            .bg(theme_rgb(ACCENT))
            .into_any_element()
    } else {
        Icon::new(if error {
            IconName::CircleX
        } else {
            IconName::Check
        })
        .xsmall()
        .text_color(theme_rgb(if error { ERROR } else { SUCCESS }))
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_tool_names_receive_localized_labels() {
        let strings = crate::i18n::Lang::Zh.strings().native;
        assert_eq!(tool_label("Read", strings), "读取");
        assert_eq!(tool_label("rg", strings), "搜索");
        assert_eq!(tool_label("CustomTool", strings), "CustomTool");
    }

    #[test]
    fn known_tools_receive_a_semantic_icon() {
        assert!(matches!(tool_icon(Some("Read")), IconName::File));
        assert!(matches!(tool_icon(Some("rg")), IconName::Search));
        assert!(matches!(tool_icon(Some("Run")), IconName::SquareTerminal));
    }
}
