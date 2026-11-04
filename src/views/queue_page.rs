use crownui::{
    kit::{ButtonOptions, ButtonVariant, button, icons},
    prelude::{Cx, Listeners, Styled, View, bind, for_each, scroll_view, show, vstack},
};

use super::{empty_state::empty_state, job_row::job_row, page_header::page_header};
use crate::{
    queue::{JobEntry, JobState},
    state::Store,
};

pub fn queue_page(cx: &mut Cx, store: Store) -> impl View + use<> {
    let subtitle = bind(move |runtime| {
        store.jobs.with(runtime, |jobs| {
            let running = jobs
                .iter()
                .filter(|entry| matches!(entry.state, JobState::Running { .. }))
                .count();
            let waiting = jobs
                .iter()
                .filter(|entry| entry.state == JobState::Waiting)
                .count();
            match (running, waiting) {
                (0, 0) => "Nothing running".to_owned(),
                (running, 0) => format!("{running} running"),
                (running, waiting) => format!("{running} running · {waiting} waiting"),
            }
        })
    });
    let clear = button("Clear Finished")
        .variant(ButtonVariant::Secondary)
        .on_click(move |cx, _| store.clear_finished(cx));
    let list = move || {
        scroll_view(
            vstack(for_each(
                store.jobs,
                |entry: &JobEntry| entry.job.id,
                move |cx, entry| job_row(cx, store, entry),
            ))
            .gap(8.0)
            .px(24.0)
            .pb(24.0),
        )
        .flex_1()
    };
    vstack((
        page_header(cx, "Queue", subtitle, (clear,)),
        show(
            bind(move |runtime| store.jobs.with(runtime, |jobs| !jobs.is_empty())),
            list,
            || {
                empty_state(
                    icons::LIST_ORDERED,
                    "The queue is empty",
                    "Installs, removals and updates run here one at a time.",
                )
            },
        )
        .flex_col()
        .flex_1()
        .min_h(0.0),
    ))
    .flex_1()
    .min_h(0.0)
}
