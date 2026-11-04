use std::future::Future;

use crownui::prelude::Cx;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

const BURST: usize = 64;

pub struct MessageProxy<E>(UnboundedSender<E>);

impl<E> MessageProxy<E> {
    /// Posts `event` to the UI thread; fails only once the window is gone.
    pub fn message(&self, event: E) -> Result<(), SendError<E>> {
        self.0.send(event)
    }
}

pub fn spawn_worker<C, E, F>(
    cx: &mut Cx,
    run: impl FnOnce(MessageProxy<E>, UnboundedReceiver<C>) -> F,
    on_event: impl FnMut(&mut Cx, E) + 'static,
) -> UnboundedSender<C>
where
    C: Send + 'static,
    E: Send + 'static,
    F: Future<Output = ()> + Send + 'static,
{
    let (commands, command_receiver) = unbounded_channel();
    let (events, event_receiver) = unbounded_channel();
    cx.spawn(run(MessageProxy(events), command_receiver), |_, ()| {});
    pump(cx, event_receiver, on_event);
    commands
}

fn pump<E: Send + 'static>(
    cx: &mut Cx,
    mut events: UnboundedReceiver<E>,
    mut on_event: impl FnMut(&mut Cx, E) + 'static,
) {
    let next_burst = async move {
        let mut burst = Vec::with_capacity(BURST);
        let open = events.recv_many(&mut burst, BURST).await > 0;
        (events, burst, open)
    };
    cx.spawn(next_burst, move |cx, (events, burst, open)| {
        for event in burst {
            on_event(cx, event);
        }
        if open {
            pump(cx, events, on_event);
        }
    });
}
