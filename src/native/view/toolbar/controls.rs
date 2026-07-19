use crate::native::{
    app::{LoadState, Shell},
    theme::*,
};

impl Shell {
    pub(in crate::native) fn status_text(&self) -> String {
        match &self.state {
            LoadState::Connecting => self.strings.native.connecting.into(),
            LoadState::Ready => self.strings.native.connected.into(),
            LoadState::Working(message) | LoadState::Failed(message) => message.clone(),
        }
    }

    pub(super) fn connection_status_color(&self) -> ColorToken {
        match self.state {
            LoadState::Ready => SUCCESS,
            LoadState::Failed(_) => ERROR,
            LoadState::Connecting | LoadState::Working(_) => TEXT_MUTED,
        }
    }
}
