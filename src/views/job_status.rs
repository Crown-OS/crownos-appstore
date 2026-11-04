use crownos_apps_database::AppId;
use crownui::prelude::Runtime;

use crate::{
    queue::{JobId, JobState},
    state::Store,
};

/// The newest unfinished job of app `id`, with its state.
pub fn active_job(runtime: &mut Runtime, store: Store, id: &AppId) -> Option<(JobId, JobState)> {
    store.jobs.with(runtime, |jobs| {
        jobs.iter()
            .rev()
            .find(|entry| entry.state.is_active() && entry.job.app.as_ref() == Some(id))
            .map(|entry| (entry.job.id, entry.state.clone()))
    })
}

/// How far job `id` is, `0` until it reports.
pub fn job_fraction(runtime: &mut Runtime, store: Store, id: JobId) -> f32 {
    store.jobs.with(runtime, |jobs| {
        jobs.iter()
            .find(|entry| entry.job.id == id)
            .and_then(|entry| match entry.state {
                JobState::Running { progress } => progress,
                _ => None,
            })
            .unwrap_or_default()
    })
}

/// The last line job `id` printed.
pub fn job_last_line(runtime: &mut Runtime, store: Store, id: JobId) -> String {
    store.jobs.with(runtime, |jobs| {
        jobs.iter()
            .find(|entry| entry.job.id == id)
            .and_then(|entry| entry.recent.back().cloned())
            .unwrap_or_default()
    })
}
