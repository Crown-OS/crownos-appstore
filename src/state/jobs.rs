use crownos_apps_database::{App, SourceKind};
use crownui::prelude::Cx;

use super::Store;
use crate::{
    queue::{Action, Job, JobEntry, JobId, JobState, QueueCommand, QueueEvent},
    source::Output,
    views::source_name,
};

impl Store {
    /// Queues installing `app` from its source at `source_index`, setting the
    /// source up first when it isn't ready.
    pub fn install(self, cx: &mut Cx, app: &App, source_index: usize) {
        let Some(source) = app.sources.get(source_index) else {
            return;
        };
        if !self.sources.with(cx, |states| states.is_ready(source.kind)) {
            self.set_up(cx, source.kind);
        }
        let action = Action::Install(source.clone());
        self.enqueue(cx, source.kind, action, app, app.name.clone());
    }

    /// Queues removing `app` from the source it is installed from.
    pub fn remove(self, cx: &mut Cx, app: &App) {
        let Some(source) = self
            .sources
            .with(cx, |states| states.installed_source(app).cloned())
        else {
            return;
        };
        let action = Action::Remove(source.package);
        self.enqueue(cx, source.kind, action, app, app.name.clone());
    }

    pub fn update_source(self, cx: &mut Cx, kind: SourceKind) {
        let job = self.new_job(
            cx,
            kind,
            Action::UpdateAll,
            None,
            format!("{} updates", source_name(kind)),
        );
        self.submit(cx, job);
    }

    pub fn update_everything(self, cx: &mut Cx) {
        let pending: Vec<SourceKind> = self.sources.with(cx, |states| {
            SourceKind::ALL
                .into_iter()
                .filter(|&kind| !states.get(kind).updates.is_empty())
                .collect()
        });
        pending
            .into_iter()
            .for_each(|kind| self.update_source(cx, kind));
    }

    pub fn check_updates(self, cx: &mut Cx) {
        self.send(cx, QueueCommand::CheckUpdates);
    }

    pub fn cancel(self, cx: &mut Cx, id: JobId) {
        self.jobs.update(cx, |jobs| {
            if let Some(entry) = jobs
                .iter_mut()
                .find(|entry| entry.job.id == id && entry.state == JobState::Waiting)
            {
                entry.state = JobState::Cancelled;
            }
        });
        self.send(cx, QueueCommand::Cancel(id));
    }

    pub fn clear_finished(self, cx: &mut Cx) {
        self.jobs
            .update(cx, |jobs| jobs.retain(|entry| entry.state.is_active()));
    }

    pub fn apply(self, cx: &mut Cx, event: QueueEvent) {
        match event {
            QueueEvent::Probed(probe) => self.sources.update(cx, |states| {
                let state = states.get_mut(probe.kind);
                state.ready = Some(probe.ready);
                state.installed = probe.installed;
            }),
            QueueEvent::UpdatesChecked { kind, updates } => {
                self.sources
                    .update(cx, |states| states.get_mut(kind).updates = updates);
            }
            QueueEvent::Started(id) => self.update_job(cx, id, |entry| {
                if entry.state == JobState::Waiting {
                    entry.state = JobState::Running { progress: None };
                }
            }),
            QueueEvent::Output(id, output) => self.update_job(cx, id, |entry| match output {
                Output::Line(line) => entry.push_line(line),
                Output::Progress(fraction) => {
                    if let JobState::Running { progress } = &mut entry.state {
                        *progress = Some(fraction);
                    }
                }
            }),
            QueueEvent::Finished(id, outcome) => self.finish(cx, id, outcome),
        }
    }

    fn finish(self, cx: &mut Cx, id: JobId, outcome: Result<(), String>) {
        let mut message = None;
        self.update_job(cx, id, |entry| {
            let job = &entry.job;
            message = Some(match &outcome {
                Ok(()) => format!("{} {}", past_tense(&job.action), job.title),
                Err(error) => format!(
                    "Couldn't {} {}: {error}",
                    job.verb().to_lowercase(),
                    job.title
                ),
            });
            entry.state = match outcome {
                Ok(()) => JobState::Succeeded,
                Err(error) => JobState::Failed(error),
            };
        });
        if let Some(message) = message {
            self.toasts.show(cx, message);
        }
    }

    fn set_up(self, cx: &mut Cx, kind: SourceKind) {
        let already_queued = self.jobs.with(cx, |jobs| {
            jobs.iter().any(|entry| {
                entry.job.kind == kind
                    && entry.job.action == Action::Setup
                    && entry.state.is_active()
            })
        });
        if !already_queued {
            let job = self.new_job(cx, kind, Action::Setup, None, source_name(kind).to_owned());
            self.submit(cx, job);
        }
    }

    fn enqueue(self, cx: &mut Cx, kind: SourceKind, action: Action, app: &App, title: String) {
        let job = self.new_job(cx, kind, action, Some(app), title);
        self.submit(cx, job);
    }

    fn new_job(
        self,
        cx: &mut Cx,
        kind: SourceKind,
        action: Action,
        app: Option<&App>,
        title: String,
    ) -> Job {
        let id = JobId(self.next_job.update(cx, |next| {
            *next += 1;
            *next
        }));
        Job {
            id,
            kind,
            action,
            app: app.map(|app| app.id.clone()),
            title,
        }
    }

    fn submit(self, cx: &mut Cx, job: Job) {
        self.jobs
            .update(cx, |jobs| jobs.push(JobEntry::new(job.clone())));
        self.send(cx, QueueCommand::Run(job));
    }

    fn send(self, cx: &mut Cx, command: QueueCommand) {
        self.commands.with(cx, |commands| {
            if let Some(commands) = commands {
                let _ = commands.send(command);
            }
        });
    }

    fn update_job(self, cx: &mut Cx, id: JobId, change: impl FnOnce(&mut JobEntry)) {
        self.jobs.update(cx, |jobs| {
            if let Some(entry) = jobs.iter_mut().find(|entry| entry.job.id == id) {
                change(entry);
            }
        });
    }
}

fn past_tense(action: &Action) -> &'static str {
    match action {
        Action::Setup => "Set up",
        Action::Install(_) => "Installed",
        Action::Remove(_) => "Removed",
        Action::UpdateAll => "Updated",
    }
}
