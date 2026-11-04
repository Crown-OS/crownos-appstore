use std::collections::VecDeque;

use crownos_apps_database::SourceKind;
use tokio::sync::mpsc::{UnboundedReceiver, error::TryRecvError};

use super::{Action, Job, QueueCommand, QueueEvent};
use crate::{
    source::{Invocation, backend, probe, stream},
    util::worker::MessageProxy,
};

/// The queue's worker: probes every source, then runs jobs in arrival order.
/// Commands sent while a job runs are handled before the next one starts.
pub async fn run_queue(
    events: MessageProxy<QueueEvent>,
    mut commands: UnboundedReceiver<QueueCommand>,
) {
    probe_sources(&events, &SourceKind::ALL).await;
    check_updates(&events, &SourceKind::ALL).await;
    let mut pending = VecDeque::new();
    loop {
        let command = match commands.try_recv() {
            Ok(command) => command,
            Err(TryRecvError::Disconnected) => return,
            Err(TryRecvError::Empty) => match pending.pop_front() {
                Some(job) => {
                    run_job(&events, job).await;
                    continue;
                }
                None => match commands.recv().await {
                    Some(command) => command,
                    None => return,
                },
            },
        };
        match command {
            QueueCommand::Run(job) => pending.push_back(job),
            QueueCommand::Cancel(id) => pending.retain(|job: &Job| job.id != id),
            QueueCommand::CheckUpdates => check_updates(&events, &SourceKind::ALL).await,
        }
    }
}

async fn run_job(events: &MessageProxy<QueueEvent>, job: Job) {
    let _ = events.message(QueueEvent::Started(job.id));
    let source = backend(job.kind);
    let steps: Vec<Invocation> = match &job.action {
        Action::Setup => source.setup(),
        Action::Install(package) => vec![source.install(package)],
        Action::Remove(package) => vec![source.remove(package)],
        Action::UpdateAll => vec![source.update_all()],
    };
    let mut outcome = Ok(());
    for step in &steps {
        let _ = events.message(QueueEvent::Output(
            job.id,
            crate::source::Output::Line(format!("$ {step}")),
        ));
        let result = stream(step, |output| {
            let _ = events.message(QueueEvent::Output(job.id, output));
        })
        .await;
        if let Err(error) = result {
            outcome = Err(error.to_string());
            break;
        }
    }
    let _ = events.message(QueueEvent::Finished(job.id, outcome));
    probe_sources(events, &[job.kind]).await;
    if job.action == Action::UpdateAll {
        check_updates(events, &[job.kind]).await;
    }
}

async fn probe_sources(events: &MessageProxy<QueueEvent>, kinds: &[SourceKind]) {
    let probes: Vec<_> = kinds
        .iter()
        .map(|&kind| tokio::task::spawn_blocking(move || probe(backend(kind))))
        .collect();
    for probe in probes {
        if let Ok(probe) = probe.await {
            let _ = events.message(QueueEvent::Probed(probe));
        }
    }
}

async fn check_updates(events: &MessageProxy<QueueEvent>, kinds: &[SourceKind]) {
    let checks: Vec<_> = kinds
        .iter()
        .map(|&kind| {
            (
                kind,
                tokio::task::spawn_blocking(move || backend(kind).updates()),
            )
        })
        .collect();
    for (kind, check) in checks {
        let updates = check.await.ok().and_then(Result::ok).unwrap_or_default();
        let _ = events.message(QueueEvent::UpdatesChecked { kind, updates });
    }
}
