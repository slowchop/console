use crate::{ActionsImpl, Error};
use bevy::ecs::system::SystemId;
use bevy::input::keyboard::KeyboardInput;
use bevy::log::Level;
use bevy::prelude::*;
use bevy::utils::tracing::field::{Field, Visit};
use bevy::utils::tracing::Subscriber;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

pub struct Entry {
    pub when: Instant,
    pub level: Level,
    pub message: String,
}

#[derive(Resource)]
pub struct Console<A> {
    pub text: String,

    lines: Arc<Mutex<VecDeque<Entry>>>,
    pub max_lines: usize,

    /// The console is open if this is true.
    pub open: bool,

    /// How far down the console will expand to, as a percentage of the screen height.
    /// 1.0 for expanding all the way down to the bottom. 0.5 for half way.
    pub expand_percentage: f32,

    needs_update: bool,

    phantom_data: std::marker::PhantomData<A>,
}

impl<A> Console<A> {
    pub fn with_lines(lines: Arc<Mutex<VecDeque<Entry>>>) -> Self {
        Console {
            text: "help".to_string(),
            lines,
            max_lines: 1000,
            open: false,
            expand_percentage: 0.5,
            needs_update: true,
            phantom_data: Default::default(),
        }
    }
}

/// The console is a text box that can be used to enter commands.
/// It will draw a background rect (e.g. black or a texture).
/// It will have an input field at the bottom, with a blinking cursor.
/// It will draw text starting from above the input, and scrolling up.
/// The user can toggle the console with a key (e.g. tilde), but they control how that's done maybe
/// via an event.
#[derive(Default, Clone)]
pub struct ConsolePlugin<A>
where
    A: Send + Sync + 'static,
{
    lines: Arc<Mutex<VecDeque<Entry>>>,
    phantom_data: std::marker::PhantomData<A>,
}

impl<A> ConsolePlugin<A>
where
    A: Send + Sync + 'static,
{
    pub fn new() -> Self {
        ConsolePlugin {
            lines: Arc::new(Mutex::new(Default::default())),
            phantom_data: Default::default(),
        }
    }
}

impl<A> Plugin for ConsolePlugin<A>
where
    A: ActionsImpl + Debug + Event + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        // app.init_resource::<Console<A>>();
        app.insert_resource(Console::<A>::with_lines(self.lines.clone()));
        app.add_event::<A>();
        app.add_event::<SubmittedText>();
        app.add_systems(Startup, (setup_console::<A>));
        app.add_systems(
            Update,
            (
                get_keyboard_input::<A>,
                update_text::<A>.run_if(needs_update::<A>),
                handle_submitted_text::<A>,
            )
                .chain(),
        );
    }
}

impl<A, S> Layer<S> for ConsolePlugin<A>
where
    A: ActionsImpl + Debug + Event + Send + Sync + 'static,
    S: Subscriber,
{
    fn on_event(&self, event: &bevy::utils::tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = ConsoleVisitor::new();

        let level = event.metadata().level();
        event.record(&mut visitor);

        let entry = Entry {
            when: Instant::now(),
            level: level.to_owned(),
            message: visitor.0,
        };

        let mut lines = self.lines.lock().unwrap();
        lines.push_back(entry);
    }
}

struct ConsoleVisitor(String);

impl ConsoleVisitor {
    fn new() -> Self {
        Self(String::new())
    }
}

impl Visit for ConsoleVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn Debug) {
        self.0.push_str(&format!("{:?} ", value));
    }
}

#[derive(Component)]
struct Group;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct InputText;

fn setup_console<A>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    console_state: Res<Console<A>>,
    window: Query<&Window, With<PrimaryWindow>>,
) where
    A: Send + Sync + 'static,
{
    let mut group = commands.spawn((
        Name::new("Console"),
        Group,
        NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(console_state.expand_percentage * 100.),
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            background_color: Color::PURPLE.into(),
            ..default()
        },
    ));

    group.with_children(|parent| {
        parent.spawn((
            Name::new("History"),
            NodeBundle {
                background_color: Color::INDIGO.into(),
                ..default()
            },
        ));

        parent.spawn((
            Name::new("Input"),
            InputText,
            TextBundle {
                style: Style { ..default() },
                background_color: Color::BLACK.into(),
                text: Text::from_section(
                    "",
                    TextStyle {
                        color: Color::ANTIQUE_WHITE,
                        font_size: 30.0,
                        ..default()
                    },
                ),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..default()
            },
        ));
    });
}

fn needs_update<A>(console: Res<Console<A>>) -> bool
where
    A: Send + Sync + 'static,
{
    console.needs_update
}

fn update_text<A>(
    mut console: ResMut<Console<A>>,
    mut text_query: Query<&mut Text, With<InputText>>,
) where
    A: Send + Sync + 'static,
{
    console.needs_update = false;
    let mut text = text_query.single_mut();
    text.sections[0].value = console.text.clone();
}

#[derive(Event, Debug)]
struct SubmittedText(String);

fn get_keyboard_input<A>(
    mut console: ResMut<Console<A>>,
    mut key_events: EventReader<ReceivedCharacter>,
    mut submitted_text_writer: EventWriter<SubmittedText>,
) where
    A: Send + Sync + 'static,
{
    for key in key_events.read() {
        info!("Key: {key:?}");

        if key.char == '\r' {
            info!("Enter!");
            submitted_text_writer.send(SubmittedText(console.text.clone()));
            console.text.clear();
        } else if key.char == '\u{7F}' {
            info!("Backspace!");
            console.text.pop();
        } else {
            console.text.push(key.char);
        }

        console.needs_update = true;
    }
}

fn handle_submitted_text<A>(
    mut commands: Commands,
    mut console: ResMut<Console<A>>,
    mut submitted_text_reader: EventReader<SubmittedText>,
    mut actions_writer: EventWriter<A>,
) where
    A: ActionsImpl + Event + Debug + Send + Sync + 'static,
{
    for text in submitted_text_reader.read() {
        // let Some(parts) = shlex::split(&text.0) else {
        //     error!("Could not parse text: {text:?}");
        //     continue;
        // };
        // let name = &parts[0];
        // let args = &parts[1..];

        let r = A::resolve(&text.0);

        match r {
            Ok(action) => {
                info!("yay Action: {:?}", action);
                actions_writer.send(action);
            }
            Err(e) => {
                error!("Error: {:?}", e);
            }
        }
    }
}
