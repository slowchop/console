use crate::{ActionsHandler, ConsolePlugin};
use std::fmt::Debug;

use crate::plugin::{Entry, QUEUED_ENTRIES};
use bevy::log::tracing_subscriber::Layer;
use bevy::utils::tracing::field::Field;
use bevy::utils::tracing::Subscriber;
use tracing_subscriber::field::Visit;
use tracing_subscriber::layer::Context;

impl<A, S> Layer<S> for ConsolePlugin<A>
where
    A: ActionsHandler + Debug + Send + Sync + 'static,
    S: Subscriber,
{
    fn on_event(&self, event: &bevy::utils::tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = ConsoleVisitor::new();

        let level = event.metadata().level();
        event.record(&mut visitor);

        let entry = Entry {
            // when: Instant::now(),
            level: level.to_owned(),
            message: visitor.0,
        };

        let mut lines = QUEUED_ENTRIES.0.lock().unwrap();
        lines.push(entry);
    }
}

struct ConsoleVisitor(String);

impl ConsoleVisitor {
    fn new() -> Self {
        Self(String::new())
    }
}

impl Visit for ConsoleVisitor {
    fn record_debug(&mut self, _field: &Field, value: &dyn Debug) {
        self.0.push_str(&format!("{:?} ", value));
    }
}
