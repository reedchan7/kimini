use gpui::{AnyElement, div, prelude::*, px};
use gpui_component::StyledExt;

use crate::{
    native::{
        app::Shell,
        session_list::{display_title, workspace_label},
        theme::*,
    },
    protocol::Session,
};

impl Shell {
    pub(super) fn session_title(&self, session: &Session) -> AnyElement {
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
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(ui_font_px())
                            .text_color(theme_rgb(TEXT_MUTED))
                            .child(workspace_label(&session.metadata.cwd)),
                    )
                    .child(
                        div()
                            .text_size(ui_font_px())
                            .text_color(theme_rgb(BORDER_STRONG))
                            .child("/"),
                    )
                    .child(
                        div()
                            .max_w(px(460.))
                            .line_clamp(1)
                            .text_size(body_font_px())
                            .font_medium()
                            .child(title),
                    ),
            )
            .into_any_element()
    }
}
