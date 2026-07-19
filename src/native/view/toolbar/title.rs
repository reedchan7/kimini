use gpui::{AnyElement, Context, IntoElement, Role, div, prelude::*, px};
use gpui_component::{StyledExt, input::Input};

use crate::{
    native::{
        app::Shell,
        session_list::{display_title, workspace_label},
        theme::*,
    },
    protocol::Session,
};

use super::{super::accessible_input::accessible_input, controls::toolbar_button};

impl Shell {
    pub(super) fn session_title(&self, session: &Session, cx: &mut Context<Self>) -> AnyElement {
        let title = display_title(&session.title);
        let title = if title.is_empty() {
            self.strings.native.untitled_session.to_owned()
        } else {
            title
        };
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(if self.renaming_session {
                self.rename_editor(cx).into_any_element()
            } else {
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(font_px(13.0))
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(workspace_label(&session.metadata.cwd)),
                    )
                    .child(
                        div()
                            .text_size(font_px(13.0))
                            .text_color(theme_rgb(BORDER_STRONG))
                            .child("/"),
                    )
                    .child(
                        div()
                            .max_w(px(460.))
                            .line_clamp(1)
                            .text_size(font_px(13.0))
                            .font_medium()
                            .child(title),
                    )
                    .into_any_element()
            })
            .into_any_element()
    }

    fn rename_editor(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .gap_1()
            .child(
                accessible_input(
                    "rename-session-input",
                    &self.rename_editor,
                    Role::TextInput,
                    self.strings.native.rename_session,
                    self.strings.native.rename_session,
                    Input::new(&self.rename_editor),
                    cx,
                )
                .h(px(30.0))
                .w(px(280.0)),
            )
            .child(
                toolbar_button(self.strings.native.save, "save-session-rename").on_click(
                    cx.listener(|this, _, window, cx| this.commit_session_rename(window, cx)),
                ),
            )
            .child(
                toolbar_button(self.strings.native.cancel, "cancel-session-rename")
                    .on_click(cx.listener(|this, _, _, cx| this.cancel_session_rename(cx))),
            )
    }
}
