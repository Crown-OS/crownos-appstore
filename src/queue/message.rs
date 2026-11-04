use crownos_apps_database::SourceKind;

use super::{Job, JobId};
use crate::source::{Output, PendingUpdate, Probe};

pub enum QueueCommand {
    Run(Job),
    Cancel(JobId),
    CheckUpdates,
}

pub enum QueueEvent {
    Probed(Probe),
    UpdatesChecked {
        kind: SourceKind,
        updates: Vec<PendingUpdate>,
    },
    Started(JobId),
    Output(JobId, Output),
    Finished(JobId, Result<(), String>),
}
