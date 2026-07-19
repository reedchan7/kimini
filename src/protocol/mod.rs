mod auth;
mod control;
mod event;
mod file;
mod fs;
mod goal;
mod interaction;
mod message;
mod model;
mod prompt;
mod session;
mod side_chat;
mod skill;
mod snapshot;
mod status;
mod task;
mod terminal;
mod workspace;

pub use auth::{
    AuthSummary, ManagedProviderSummary, OAuthCancelResult, OAuthFlowSnapshot, OAuthFlowStart,
    OAuthFlowStatus, OAuthLogoutResult,
};
pub use control::{ClientControl, SessionCursor};
pub use event::WireEvent;
pub use file::FileMeta;
pub use fs::{
    FsDiff, FsEntry, FsGitStatus, FsGitStatusSummary, FsKind, FsList, FsPreview, FsSearchHit,
    FsSearchResults,
};
pub use goal::{GoalBudget, GoalControl, GoalSnapshot};
pub use interaction::{ApprovalRequest, QuestionAnswer, QuestionAnswers, QuestionRequest};
pub use message::{MediaSource, Message, MessageContent, MessagePage, MessageRole};
pub use model::{Catalog, ModelCatalogItem};
pub use prompt::{
    PromptAbortResult, PromptItem, PromptOptions, PromptPart, PromptQueue, PromptStatus,
    PromptSteerResult,
};
pub use session::{Page, Session};
pub use side_chat::SideChatStart;
pub use skill::{ActivateSkillRequest, ActivateSkillResult, SkillDescriptor, SkillList};
pub use snapshot::{InFlightTurn, SessionSnapshot};
pub use status::SessionStatus;
pub use task::{Task, TaskKind, TaskList, TaskStatus};
pub use terminal::{
    CreateTerminal, Terminal, TerminalExit, TerminalList, TerminalOutput, TerminalStatus,
};
#[cfg(feature = "native")]
pub(crate) use terminal::{TerminalExitFrame, TerminalOutputFrame};
pub use workspace::{Workspace, WorkspaceList};
