use gpui::{AnyElement, Context, Role, div, prelude::*, px, rgb};

use super::super::app::Shell;
use super::super::attachment::AttachmentState;
use super::super::theme::*;

impl Shell {
    pub(super) fn attachment_strip(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(session) = self.model.active_session() else {
            return div().into_any_element();
        };
        let drafts = self.attachments.for_session(&session.id).to_vec();
        div()
            .when(!drafts.is_empty(), |strip| {
                strip
                    .pb_2()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .children(drafts.into_iter().map(|draft| {
                        let id = draft.id;
                        let (status, failed) = match draft.state {
                            AttachmentState::Uploading => {
                                (self.strings.native.uploading_file.to_owned(), false)
                            }
                            AttachmentState::Ready(file) => (format_size(file.size), false),
                            AttachmentState::Failed(error) => (
                                format!("{} · {error}", self.strings.native.upload_failed),
                                true,
                            ),
                        };
                        div()
                            .id(("attachment", id))
                            .role(Role::Group)
                            .aria_label(format!("{} · {status}", draft.name))
                            .max_w(px(320.))
                            .flex()
                            .items_center()
                            .gap_2()
                            .rounded_md()
                            .border_1()
                            .border_color(rgb(if failed { ERROR } else { BORDER }))
                            .px_2()
                            .py_1()
                            .text_xs()
                            .child(div().line_clamp(1).child(draft.name))
                            .child(div().text_color(rgb(TEXT_MUTED)).child(status))
                            .child(
                                div()
                                    .id(("remove-attachment", id))
                                    .focusable()
                                    .tab_stop(true)
                                    .role(Role::Button)
                                    .aria_label(self.strings.native.remove_attachment)
                                    .cursor_pointer()
                                    .rounded_sm()
                                    .px_1()
                                    .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.remove_attachment(id, cx)
                                    }))
                                    .child("×"),
                            )
                    }))
            })
            .into_any_element()
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::format_size;

    #[test]
    fn attachment_sizes_stay_compact() {
        assert_eq!(format_size(42), "42 B");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(2 * 1024 * 1024), "2.0 MB");
    }
}
