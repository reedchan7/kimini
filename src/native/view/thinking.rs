use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::{
    IconName, Selectable, Sizable as _, StyledExt,
    button::{Button, ButtonVariants},
    scroll::ScrollableElement,
    text::TextView,
};

use super::super::app::{Shell, UtilityPanel};
use super::super::theme::*;

impl Shell {
    pub(super) fn thinking_button(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        self.transcript.latest_thinking()?;
        Some(
            Button::new("thinking-preview-toggle")
                .xsmall()
                .ghost()
                .selected(self.utility_panel == Some(UtilityPanel::Thinking))
                .icon(IconName::PanelRight)
                .tooltip(self.strings.native.preview_thinking)
                .on_click(cx.listener(|this, _, _, cx| {
                    if this.utility_panel == Some(UtilityPanel::Thinking) {
                        this.utility_panel = None;
                    } else {
                        this.preview_thinking = None;
                        this.utility_panel = Some(UtilityPanel::Thinking);
                    }
                    cx.notify();
                }))
                .into_any_element(),
        )
    }

    pub(super) fn thinking_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let thinking = self
            .preview_thinking
            .as_deref()
            .or_else(|| self.transcript.latest_thinking());
        div()
            .id("thinking-preview")
            .role(Role::Complementary)
            .aria_label(self.strings.native.preview_thinking)
            .w(gpui::relative(0.38))
            .min_w(px(400.0))
            .max_w(px(600.0))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(
                div()
                    .h(px(40.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .border_b_1()
                    .border_color(rgb(BORDER))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_semibold()
                            .text_color(rgb(TEXT_SECONDARY))
                            .child(self.strings.native.preview_thinking),
                    )
                    .child(
                        Button::new("close-thinking-preview")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Close)
                            .tooltip(self.strings.native.close_thinking)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.utility_panel = None;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .w_full()
                    .overflow_y_scrollbar()
                    .p_4()
                    .child(if let Some(text) = thinking {
                        div()
                            .w_full()
                            .child(
                                TextView::markdown("thinking-preview-content", text.to_owned())
                                    .selectable(true)
                                    .w_full()
                                    .text_sm()
                                    .text_color(rgb(TEXT_SECONDARY)),
                            )
                            .into_any_element()
                    } else {
                        div()
                            .text_sm()
                            .text_color(rgb(TEXT_MUTED))
                            .child(self.strings.native.thinking_preview_hint)
                            .into_any_element()
                    }),
            )
    }

    pub(super) fn open_thinking_preview(&mut self, text: String, cx: &mut Context<Self>) {
        self.preview_thinking = Some(text);
        self.utility_panel = Some(UtilityPanel::Thinking);
        cx.notify();
    }
}
