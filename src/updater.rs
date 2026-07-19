use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::path::{Path, PathBuf};

use objc2::rc::{Allocated, Retained};
use objc2::runtime::NSObject;
use objc2::{ClassType, MainThreadOnly, extern_class, extern_methods, msg_send};

pub const LATEST_RELEASE_URL: &str = "https://github.com/reedchan7/kimini/releases/latest";

const RTLD_NOW: c_int = 2;

unsafe extern "C" {
    fn dlopen(path: *const c_char, mode: c_int) -> *mut c_void;
    fn dlerror() -> *const c_char;
}

extern_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SPUStandardUpdaterController"]
    struct StandardUpdaterController;
);

impl StandardUpdaterController {
    extern_methods!(
        #[unsafe(method(initWithStartingUpdater:updaterDelegate:userDriverDelegate:))]
        fn init_with_starting_updater(
            this: Allocated<Self>,
            starting_updater: bool,
            updater_delegate: Option<&NSObject>,
            user_driver_delegate: Option<&NSObject>,
        ) -> Retained<Self>;

        #[unsafe(method(checkForUpdates:))]
        fn check_for_updates(&self, sender: Option<&NSObject>);
    );
}

/// Owns Sparkle's standard updater controller for the lifetime of an app.
pub struct Updater {
    controller: Option<Retained<StandardUpdaterController>>,
}

impl Updater {
    /// Starts scheduled update checks when running from a packaged app.
    pub fn new() -> Self {
        let Some(framework) = std::env::current_exe()
            .ok()
            .and_then(|executable| bundled_framework_binary(&executable))
        else {
            return Self { controller: None };
        };
        let result = load_framework(&framework).map(|()| {
            let allocated: Allocated<StandardUpdaterController> =
                unsafe { msg_send![StandardUpdaterController::class(), alloc] };
            StandardUpdaterController::init_with_starting_updater(allocated, true, None, None)
        });

        match result {
            Ok(controller) => Self {
                controller: Some(controller),
            },
            Err(error) => {
                eprintln!("kimini: automatic updates unavailable: {error}");
                Self { controller: None }
            }
        }
    }

    /// Opens Sparkle's standard update window. Returns false outside a bundle.
    pub fn check_now(&self) -> bool {
        let Some(controller) = self.controller.as_ref() else {
            return false;
        };
        controller.check_for_updates(None);
        true
    }
}

impl Default for Updater {
    fn default() -> Self {
        Self::new()
    }
}

fn bundled_framework_binary(executable: &Path) -> Option<PathBuf> {
    let macos = executable.parent()?;
    if macos.file_name()?.to_str()? != "MacOS" {
        return None;
    }
    let contents = macos.parent()?;
    if contents.file_name()?.to_str()? != "Contents" {
        return None;
    }
    Some(contents.join("Frameworks/Sparkle.framework/Sparkle"))
}

fn load_framework(path: &Path) -> Result<(), String> {
    if !path.is_file() {
        return Err(format!("missing {}", path.display()));
    }
    let path = CString::new(path.as_os_str().as_encoded_bytes())
        .map_err(|_| "Sparkle framework path contains a NUL byte".to_owned())?;
    let handle = unsafe { dlopen(path.as_ptr(), RTLD_NOW) };
    if handle.is_null() {
        let message = unsafe {
            let error = dlerror();
            (!error.is_null()).then(|| CStr::from_ptr(error).to_string_lossy().into_owned())
        };
        return Err(message.unwrap_or_else(|| "could not load Sparkle.framework".to_owned()));
    }
    // Sparkle stays loaded for the process lifetime; the controller owns its objects.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_framework_next_to_a_packaged_executable() {
        assert_eq!(
            bundled_framework_binary(Path::new("/Applications/Kimini.app/Contents/MacOS/kimini")),
            Some(PathBuf::from(
                "/Applications/Kimini.app/Contents/Frameworks/Sparkle.framework/Sparkle"
            ))
        );
        assert_eq!(bundled_framework_binary(Path::new("/tmp/kimini")), None);
    }
}
