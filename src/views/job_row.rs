use crownui::{
    kit::{ButtonOptions, ButtonVariant, Theme, Tone, button, progress, spinner, use_theme},
    prelude::{
        Color, Cx, Listeners, Runtime, Signal, Styled, View, bind, hstack, show, text, vstack,
    },
};

use super::source_label::source_name;
use crate::{
    queue::{JobEntry, JobState},
    state::Store,
    util::theme::tint,
};

const SHOWN_LINES: usize = 2;

pub fn job_row(cx: &mut Cx, store: Store, entry: Signal<JobEntry>) -> impl View + use<> {
    let theme = use_theme(cx);
    let job = entry.with_untracked(cx, |entry| entry.job.clone());
    let state = move |runtime: &mut Runtime| entry.with(runtime, |entry| entry.state.clone());
    let running = move |runtime: &mut Runtime| matches!(state(runtime), JobState::Running { .. });
    let fraction = move |runtime: &mut Runtime| match state(runtime) {
        JobState::Running { progress } => progress,
        _ => None,
    };
    let bar = move || {
        show(
            bind(move |runtime| fraction(runtime).is_some()),
            move || progress(bind(move |runtime| fraction(runtime).unwrap_or_default())),
            spinner,
        )
    };
    let output = text(bind(move |runtime| {
        entry.with(runtime, |entry| {
            let skipped = entry.recent.len().saturating_sub(SHOWN_LINES);
            entry
                .recent
                .iter()
                .skip(skipped)
                .cloned()
                .collect::<Vec<_>>()
                .join("\n")
        })
    }))
    .text_size(11.0)
    .text_color(tint(theme, |theme| theme.text.muted));
    let id = job.id;
    vstack((
        hstack((
            vstack((
                text(format!("{} {}", job.verb(), job.title))
                    .text_size(14.0)
                    .font_semibold(),
                text(source_name(job.kind))
                    .text_size(12.0)
                    .text_color(tint(theme, |theme| theme.text.muted)),
            ))
            .gap(2.0)
            .flex_1(),
            text(bind(move |runtime| state_label(&state(runtime)).to_owned()))
                .text_size(12.0)
                .font_medium()
                .text_color(bind(move |runtime| {
                    let tone = state_tone(&state(runtime));
                    theme.with(runtime, |theme| tone_color(tone, theme))
                })),
            show(
                bind(move |runtime| state(runtime) == JobState::Waiting),
                move || {
                    button("Cancel")
                        .variant(ButtonVariant::Ghost)
                        .on_click(move |cx, _| store.cancel(cx, id))
                },
                || vstack(()),
            ),
        ))
        .items_center()
        .gap(12.0),
        show(bind(running), bar, || vstack(())),
        output,
    ))
    .gap(8.0)
    .p(14.0)
    .rounded(12.0)
    .bg(tint(theme, |theme| theme.surface.raised))
}

fn state_label(state: &JobState) -> &str {
    match state {
        JobState::Waiting => "Waiting",
        JobState::Running { .. } => "Running",
        JobState::Succeeded => "Done",
        JobState::Failed(error) => error,
        JobState::Cancelled => "Cancelled",
    }
}

fn state_tone(state: &JobState) -> Tone {
    match state {
        JobState::Succeeded => Tone::Success,
        JobState::Failed(_) => Tone::Danger,
        JobState::Running { .. } => Tone::Warning,
        JobState::Waiting | JobState::Cancelled => Tone::Neutral,
    }
}

fn tone_color(tone: Tone, theme: &Theme) -> Color {
    tone.text_color(theme)
}
