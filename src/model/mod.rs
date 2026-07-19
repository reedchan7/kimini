mod reducer;
mod state;

pub use reducer::ApplyOutcome;
#[cfg(feature = "native")]
pub(crate) use state::OptimisticUserMessage;
pub use state::{AppModel, Conversation};
