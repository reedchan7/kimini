#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

use gpui::Context;

use super::super::app::{LoadState, Shell};

impl Shell {
    pub(in crate::native) fn reconnect(&mut self, cx: &mut Context<Self>) {
        self.socket_generation = self.socket_generation.wrapping_add(1);
        self.socket = None;
        self.client = None;
        self.connection = None;
        self.state = LoadState::Connecting;
        self.start_bootstrap(cx);
        cx.notify();
    }

    pub(in crate::native) fn open_web_fallback(&mut self, cx: &mut Context<Self>) {
        let Some(url) = self
            .connection
            .as_ref()
            .map(|connection| connection.web_url())
        else {
            return;
        };
        #[cfg(target_os = "macos")]
        if let Ok(executable) = std::env::current_exe()
            && let Some(bundle) = sibling_web_bundle_for(&executable)
            && bundle.is_dir()
            && std::process::Command::new("open")
                .arg("-na")
                .arg(bundle)
                .arg("--args")
                .arg(&url)
                .spawn()
                .is_ok()
        {
            return;
        }
        cx.open_url(&url);
    }
}

#[cfg(target_os = "macos")]
fn sibling_web_bundle_for(executable: &Path) -> Option<PathBuf> {
    let macos = executable.parent()?;
    if macos.file_name()?.to_str()? != "MacOS" {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()?.to_str()? != "Contents" {
        return None;
    }
    let app = contents.parent()?;
    app.parent().map(|parent| parent.join("Kimini Web.app"))
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn packaged_native_binary_resolves_the_adjacent_web_bundle() {
        assert_eq!(
            sibling_web_bundle_for(Path::new("/Applications/Kimini.app/Contents/MacOS/kimini")),
            Some(PathBuf::from("/Applications/Kimini Web.app"))
        );
        assert!(sibling_web_bundle_for(Path::new("/tmp/kimini")).is_none());
    }
}
