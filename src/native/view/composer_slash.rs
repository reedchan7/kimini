use gpui::{AnyElement, Context, Role, div, prelude::*};
use gpui_component::StyledExt;

use crate::native::{app::Shell, theme::*};
use crate::native::{skills::SkillSuggestion, slash::SlashCommand};

impl Shell {
    pub(super) fn slash_suggestions(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let input = self.composer.read(cx).value();
        let new_session = self.new_session_draft.is_some();
        let mut suggestions = SlashCommand::suggestions(input.as_ref(), 8)
            .into_iter()
            .filter(|command| {
                !new_session
                    || SlashCommand::parse(command)
                        .is_some_and(|command| command.available_in_new_session())
            })
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
                .border_color(theme_rgb(BORDER))
                .bg(theme_rgb(CANVAS))
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
                                .hover(|item| item.bg(theme_rgb(SURFACE_ACTIVE)))
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
                                                .text_size(font_px(13.0))
                                                .font_semibold()
                                                .child(suggestion.command),
                                        )
                                        .child(
                                            div()
                                                .min_w_0()
                                                .flex_1()
                                                .text_size(font_px(12.0))
                                                .text_color(theme_rgb(TEXT_MUTED))
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
