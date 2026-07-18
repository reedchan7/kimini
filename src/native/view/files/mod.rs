mod format;
mod preview;
mod tree;

use gpui::{Context, IntoElement, Role, div, prelude::*, px, rgb};
use gpui_component::{StyledExt, input::Input};

use crate::native::{app::Shell, theme::*};

use super::{accessible_input::accessible_input, panel::panel_button};

impl Shell {
    pub(super) fn file_panel(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("file-panel")
            .role(Role::Group)
            .aria_label(self.strings.native.files_panel)
            .w(px(FILE_PANEL_WIDTH))
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .border_l_1()
            .border_color(rgb(BORDER))
            .bg(rgb(SURFACE))
            .child(self.file_panel_header(cx))
            .child(
                div().px_3().py_2().child(
                    accessible_input(
                        "file-search-input",
                        &self.file_search,
                        Role::SearchInput,
                        self.strings.native.search_files,
                        self.strings.native.search_files,
                        Input::new(&self.file_search),
                        cx,
                    )
                    .h(px(32.0)),
                ),
            )
            .children(self.files.git.as_ref().map(|git| {
                div()
                    .px_3()
                    .pb_2()
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_xs()
                    .text_color(rgb(TEXT_MUTED))
                    .child(format!(
                        "{} · ↑{} ↓{}",
                        if git.branch.is_empty() {
                            "—"
                        } else {
                            &git.branch
                        },
                        git.ahead,
                        git.behind
                    ))
                    .child(format!("+{} −{}", git.additions, git.deletions))
            }))
            .child(self.file_tree(cx))
            .child(self.file_preview(cx))
    }

    fn file_panel_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .h(px(48.0))
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
                    .child(self.strings.native.files),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .child(
                        panel_button(self.strings.native.refresh_files, "refresh-files").on_click(
                            cx.listener(|this, _, _, cx| this.refresh_workspace_files(cx)),
                        ),
                    )
                    .child(
                        panel_button(self.strings.native.close_files, "close-files")
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_file_panel(cx))),
                    ),
            )
    }
}
