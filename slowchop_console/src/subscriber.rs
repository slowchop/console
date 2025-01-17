use std::fmt::Debug;
use std::sync::mpsc;

use bevy::log::tracing_subscriber::Layer;
use bevy::log::{BoxedLayer, Level};
use bevy::prelude::*;
use bevy::utils::tracing::{self, Subscriber};

pub fn slowchop_log_layer(app: &mut App) -> Option<BoxedLayer> {
    let (sender, receiver) = mpsc::channel();

    let layer = CaptureLayer { sender };
    let resource = CapturedLogEvents(receiver);

    app.insert_non_send_resource(resource);
    app.add_event::<LogEvent>();
    app.add_systems(Update, transfer_log_events);

    Some(layer.boxed())
}

/// A basic message. This is what we will be sending from the [`CaptureLayer`] to [`CapturedLogEvents`] non-send resource.
#[derive(Debug, Event)]
pub(crate) struct LogEvent {
    pub level: Level,
    pub message: String,
}

/// This non-send resource temporarily stores [`LogEvent`]s before they are
/// written to [`Events<LogEvent>`] by [`transfer_log_events`].
#[derive(Deref, DerefMut)]
struct CapturedLogEvents(mpsc::Receiver<LogEvent>);

/// Transfers information from the `LogEvents` resource to [`Events<LogEvent>`](LogEvent).
fn transfer_log_events(
    receiver: NonSend<CapturedLogEvents>,
    mut log_events: EventWriter<LogEvent>,
) {
    // Make sure to use `try_iter()` and not `iter()` to prevent blocking.
    log_events.send_batch(receiver.try_iter());
}

/// This is the [`Layer`] that we will use to capture log events and then send them to Bevy's
/// ECS via it's [`mpsc::Sender`].
struct CaptureLayer {
    sender: mpsc::Sender<LogEvent>,
}
impl<S: Subscriber> Layer<S> for CaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // In order to obtain the log message, we have to create a struct that implements
        // Visit and holds a reference to our string. Then we use the `record` method and
        // the struct to modify the reference to hold the message string.
        let mut message = None;
        event.record(&mut CaptureLayerVisitor(&mut message));
        if let Some(message) = message {
            let metadata = event.metadata();

            self.sender
                .send(LogEvent {
                    message,
                    level: *metadata.level(),
                })
                .expect("LogEvents resource no longer exists!");
        }
    }
}

/// A [`Visit`](tracing::field::Visit)or that records log messages that are transferred to [`CaptureLayer`].
struct CaptureLayerVisitor<'a>(&'a mut Option<String>);
impl tracing::field::Visit for CaptureLayerVisitor<'_> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        // This if statement filters out unneeded events sometimes show up
        if field.name() == "message" {
            *self.0 = Some(format!("{value:?}"));
        }
    }
}
