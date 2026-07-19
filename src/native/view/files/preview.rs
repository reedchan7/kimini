use gpui::{AnyElement, Context, Div, IntoElement, Role, Stateful, div, prelude::*, px};
use gpui_component::{StyledExt, scroll::ScrollableElement, text::TextView};

use crate::native::{
    app::Shell,
    files::{FilePreview, PreviewMode},
    theme::*,
};

use super::{
    super::panel::panel_button,
    format::{diff_markdown, source_markdown},
};

impl Shell {
    pub(super) fn file_preview(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let preview = self.files.preview.as_ref();
        let has_diff = preview.and_then(|item| item.diff.as_ref()).is_some();
        div()
            .id("file-preview")
            .role(Role::Document)
            .aria_label(self.strings.native.file_preview)
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(40.0))
                    .flex_none()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .border_b_1()
                    .border_color(theme_rgb(BORDER))
                    .child(
                        div()
                            .min_w_0()
                            .text_size(font_px(12.0))
                            .font_semibold()
                            .line_clamp(1)
                            .child(
                                preview
                                    .map(|item| item.file.path.clone())
                                    .unwrap_or_else(|| self.strings.native.file_preview.to_owned()),
                            ),
                    )
                    .when(preview.is_some(), |header| {
                        header.child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1()
                                .child(
                                    preview_mode_button(
                                        self.strings.native.source,
                                        "preview-source",
                                        self.files.preview_mode == PreviewMode::Source,
                                    )
                                    .on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.set_file_preview_mode(PreviewMode::Source, cx)
                                        },
                                    )),
                                )
                                .when(has_diff, |actions| {
                                    actions.child(
                                        preview_mode_button(
                                            self.strings.native.diff,
                                            "preview-diff",
                                            self.files.preview_mode == PreviewMode::Diff,
                                        )
                                        .on_click(
                                            cx.listener(|this, _, _, cx| {
                                                this.set_file_preview_mode(PreviewMode::Diff, cx)
                                            }),
                                        ),
                                    )
                                }),
                        )
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p_3()
                    .when_some(self.files.error.clone(), |body, error| {
                        body.child(
                            div()
                                .text_size(font_px(12.0))
                                .text_color(theme_rgb(ERROR))
                                .child(error),
                        )
                    })
                    .when(self.files.preview_loading, |body| {
                        body.child(
                            div()
                                .text_size(font_px(13.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(self.strings.native.loading_file),
                        )
                    })
                    .when(!self.files.preview_loading && preview.is_none(), |body| {
                        body.child(
                            div()
                                .text_size(font_px(13.0))
                                .text_color(theme_rgb(TEXT_MUTED))
                                .child(self.strings.native.select_file),
                        )
                    })
                    .children(
                        (!self.files.preview_loading)
                            .then_some(preview)
                            .flatten()
                            .map(|item| match self.files.preview_mode {
                                PreviewMode::Diff => item
                                    .diff
                                    .as_ref()
                                    .map(|diff| {
                                        TextView::markdown(
                                            "workspace-diff",
                                            diff_markdown(&diff.diff),
                                        )
                                        .selectable(true)
                                        .text_size(font_px(12.0))
                                        .into_any_element()
                                    })
                                    .unwrap_or_else(|| self.source_preview(item)),
                                PreviewMode::Source => self.source_preview(item),
                            }),
                    ),
            )
    }

    fn source_preview(&self, preview: &FilePreview) -> AnyElement {
        if preview.file.is_binary || preview.file.encoding == "base64" {
            return div()
                .text_size(font_px(13.0))
                .text_color(theme_rgb(TEXT_MUTED))
                .child(format!(
                    "{} · {} · {} bytes",
                    self.strings.native.binary_file, preview.file.mime, preview.file.size
                ))
                .into_any_element();
        }
        TextView::markdown("workspace-source", source_markdown(&preview.file))
            .selectable(true)
            .text_size(font_px(12.0))
            .into_any_element()
    }
}

fn preview_mode_button(label: &'static str, id: &'static str, active: bool) -> Stateful<Div> {
    panel_button(label, id).bg(theme_rgb(if active { SURFACE_ACTIVE } else { SURFACE }))
}
