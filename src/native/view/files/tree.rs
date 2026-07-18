use gpui::{AnyElement, Context, IntoElement, Role, div, list, prelude::*, px, rgb};
use gpui_component::scroll::ScrollableElement;

use crate::{
    native::{app::Shell, files::FileRow, theme::*},
    protocol::FsGitStatus,
};

impl Shell {
    pub(super) fn file_tree(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.files.list.clone();
        let shell = cx.entity();
        div()
            .id("workspace-file-tree")
            .role(Role::Tree)
            .aria_label(self.strings.native.workspace_files)
            .h(px(290.0))
            .flex_none()
            .relative()
            .border_t_1()
            .border_b_1()
            .border_color(rgb(BORDER))
            .when(self.files.rows.is_empty(), |tree| {
                tree.child(div().p_4().text_sm().text_color(rgb(TEXT_MUTED)).child(
                    if self.files.loading {
                        self.strings.native.loading_files
                    } else {
                        self.strings.native.no_files
                    },
                ))
            })
            .when(!self.files.rows.is_empty(), |tree| {
                tree.child(
                    list(state.clone(), move |index, _, cx| {
                        shell.update(cx, |this, cx| this.file_tree_row(index, cx))
                    })
                    .size_full(),
                )
                .vertical_scrollbar(&state)
            })
    }

    fn file_tree_row(&self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let Some(row) = self.files.rows.get(index).cloned() else {
            return div().into_any_element();
        };
        let path = row.path.clone();
        let directory = row.is_directory();
        let selected = self
            .files
            .preview
            .as_ref()
            .is_some_and(|preview| preview.file.path == row.path);
        div()
            .id(("workspace-file", index))
            .role(Role::TreeItem)
            .aria_label(file_row_label(&row))
            .aria_level(row.depth + 1)
            .aria_selected(selected)
            .when(directory, |item| item.aria_expanded(row.expanded))
            .focusable()
            .tab_stop(true)
            .cursor_pointer()
            .h(px(30.0))
            .pl(px(10.0 + row.depth as f32 * 16.0))
            .pr_3()
            .flex()
            .items_center()
            .gap_2()
            .bg(rgb(if selected { SURFACE_ACTIVE } else { SURFACE }))
            .hover(|item| item.bg(rgb(SURFACE_ACTIVE)))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.activate_file_row(path.clone(), directory, cx)
            }))
            .child(
                div()
                    .w(px(12.0))
                    .text_xs()
                    .text_color(rgb(TEXT_MUTED))
                    .child(if directory {
                        if row.expanded { "▾" } else { "▸" }
                    } else {
                        ""
                    }),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_1()
                    .text_sm()
                    .line_clamp(1)
                    .child(row.name),
            )
            .children(row.git_status.map(|status| {
                div()
                    .text_xs()
                    .text_color(rgb(git_status_color(status)))
                    .child(git_status_badge(status))
            }))
            .into_any_element()
    }
}

fn file_row_label(row: &FileRow) -> String {
    let kind = if row.is_directory() {
        "Directory"
    } else {
        "File"
    };
    row.git_status.map_or_else(
        || format!("{kind}: {}", row.path),
        |status| format!("{kind}: {} · {status:?}", row.path),
    )
}

fn git_status_badge(status: FsGitStatus) -> &'static str {
    match status {
        FsGitStatus::Clean => "",
        FsGitStatus::Modified => "M",
        FsGitStatus::Added => "A",
        FsGitStatus::Deleted => "D",
        FsGitStatus::Renamed => "R",
        FsGitStatus::Untracked => "?",
        FsGitStatus::Ignored => "I",
        FsGitStatus::Conflicted => "!",
    }
}

fn git_status_color(status: FsGitStatus) -> u32 {
    match status {
        FsGitStatus::Conflicted | FsGitStatus::Deleted => ERROR,
        FsGitStatus::Clean | FsGitStatus::Ignored => TEXT_MUTED,
        _ => ACCENT,
    }
}
