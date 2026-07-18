use gpui::{AnyElement, Context, Role, div, prelude::*, rgb};
use gpui_component::StyledExt;

use crate::native::{app::Shell, theme::*};
use crate::native::{skills::SkillSuggestion, slash::SlashCommand};

impl Shell {
    pub(super) fn slash_suggestions(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let input = self.composer.read(cx).value();
        let mut suggestions = SlashCommand::suggestions(input.as_ref(), 8)
            .into_iter()
            .map(|command| SkillSuggestion {
                command: command.into(),
                name: command.trim_start_matches('/').into(),
                description: self.strings.native.built_in_command.into(),
            })
            .collect::<Vec<_>>();
        suggestions.extend(
            self.skills
                .suggestions(input.as_ref(), 8usize.saturating_sub(suggestions.len())),
        );
        (!suggestions.is_empty()).then(|| {
            div()
                .id("slash-suggestions")
                .role(Role::List)
                .aria_label(self.strings.native.slash_commands)
                .mb_2()
                .rounded_md()
                .border_1()
                .border_color(rgb(BORDER))
                .bg(rgb(CANVAS))
                .p_1()
                .children(
                    suggestions
                        .into_iter()
                        .enumerate()
                        .map(|(index, suggestion)| {
                            let command = format!("{} ", suggestion.command);
                            div()
                                .id(("slash-suggestion", index))
                                .role(Role::ListItem)
                                .aria_label(format!(
                                    "{}: {}",
                                    suggestion.command, suggestion.description
                                ))
                                .focusable()
                                .tab_stop(true)
                                .cursor_pointer()
                                .rounded_md()
                                .px_2()
                                .py_1()
                                .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.composer.update(cx, |input, cx| {
                                        input.set_value(command.clone(), window, cx);
                                        input.focus(window, cx);
                                    });
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_semibold()
                                                .child(suggestion.command),
                                        )
                                        .child(
                                            div()
                                                .min_w_0()
                                                .flex_1()
                                                .text_xs()
                                                .text_color(rgb(TEXT_MUTED))
                                                .line_clamp(1)
                                                .child(suggestion.description),
                                        ),
                                )
                        }),
                )
                .into_any_element()
        })
    }
}
