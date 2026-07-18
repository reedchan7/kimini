use std::path::{Path, PathBuf};

use gpui::{AppContext, Context};

use super::super::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn export_active_session(&mut self, cx: &mut Context<Self>) {
        let Some((client, session_id)) = self.active_request_context() else {
            return;
        };
        let destination = default_export_directory();
        let suggested_name = archive_name(&session_id);
        let selection = cx.prompt_for_new_path(&destination, Some(&suggested_name));
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(path))) = selection.await else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                this.state = LoadState::Working(this.strings.native.exporting.into());
                let task = cx
                    .background_spawn(async move { client.export_session_to(&session_id, &path) });
                cx.spawn(async move |this, cx| {
                    let result = task.await.map_err(|error| error.to_string());
                    let _ = this.update(cx, |this, cx| match result {
                        Ok(exported) => {
                            this.state = LoadState::Ready;
                            cx.reveal_path(&exported.path);
                            cx.notify();
                        }
                        Err(error) => this.fail(error, cx),
                    });
                })
                .detach();
                cx.notify();
            });
        })
        .detach();
    }
}

fn default_export_directory() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Downloads"))
        .filter(|path| path.is_dir())
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| Path::new(".").to_owned())
}

fn archive_name(session_id: &str) -> String {
    let safe = session_id
        .chars()
        .take(48)
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '_' | '-') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>();
    format!(
        "kimi-session-{}.zip",
        if safe.is_empty() { "session" } else { &safe }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_names_are_bounded_and_safe() {
        assert_eq!(archive_name("session/a b"), "kimi-session-session_a_b.zip");
        assert_eq!(archive_name(""), "kimi-session-session.zip");
        assert!(archive_name(&"x".repeat(80)).len() <= 65);
    }
}
