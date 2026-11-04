//! Package operations run one at a time, so two never fight over the pacman lock.

mod entry;
mod job;
mod message;
mod worker;

pub use entry::{JobEntry, JobState};
pub use job::{Action, Job, JobId};
pub use message::{QueueCommand, QueueEvent};
pub use worker::run_queue;
