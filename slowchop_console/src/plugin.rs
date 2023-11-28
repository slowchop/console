use crate::ActionsHandler;
use bevy::log::Level;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

/// A container for entries that are queued up to be added to the console.
///
/// This was created specifically to use with the tracing subscriber.
#[derive(Debug, Clone, Default)]
pub struct QueuedEntries(pub(crate) Arc<Mutex<Vec<Entry>>>);

#[derive(Debug)]
struct EntityEntry {
    entity: Entity,
    // entry: Entry,
}

#[derive(Debug)]
pub(crate) struct Entry {
    // pub(crate) when: Instant, // TODO: Use this to show the time of the entry, or fade out old entries.
    pub(crate) level: Level,
    pub(crate) message: String,
}

#[derive(Resource, Debug)]
pub struct Console<A> {
    input: String,

    pub max_lines: usize,

    pub toggle_key_code: Option<KeyCode>,
    pub open_key_code: Option<KeyCode>,
    pub close_key_code: Option<KeyCode>,

    pub font_size: f32,

    /// The console is open if this is true.
    pub open: bool,

    /// To stop the toggle from opening and closing the console on the same frame.
    did_close_this_frame: bool,

    /// How far down the console will expand to, as a percentage of the screen height.
    /// 1.0 for expanding all the way down to the bottom. 0.5 for half way.
    pub expand_fraction: f32,

    /// When the console changed from open to closed or vice versa.
    ///
    /// Triggers a visibility check.
    console_did_toggle: bool,

    /// When the input text changed.
    input_did_update: bool,

    entity_entries: VecDeque<EntityEntry>,
    queued_entries: QueuedEntries,
    phantom_data: std::marker::PhantomData<A>,
}

impl<A> Console<A> {
    pub fn with_lines(queued_entries: QueuedEntries) -> Self {
        Console {
            input: "".to_string(),
            toggle_key_code: Some(KeyCode::Grave),
            close_key_code: Some(KeyCode::Escape),
            open_key_code: None,
            queued_entries,
            entity_entries: Default::default(),
            max_lines: 100,
            font_size: 20.0,
            open: false,
            did_close_this_frame: false,
            expand_fraction: 0.5,
            input_did_update: true,
            console_did_toggle: true,
            phantom_data: Default::default(),
        }
    }

    pub fn open(&mut self) {
        self.open = true;
        self.console_did_toggle = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.console_did_toggle = true;
    }

    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.console_did_toggle = true;
    }
}

#[derive(Clone, Debug)]
pub struct ConsolePlugin<A>
where
    A: Send + Sync + 'static,
{
    pub(crate) queued_entries: QueuedEntries,
    phantom_data: std::marker::PhantomData<A>,
}

impl<A> Default for ConsolePlugin<A>
where
    A: Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            queued_entries: Default::default(),
            phantom_data: Default::default(),
        }
    }
}

impl<A> Plugin for ConsolePlugin<A>
where
    A: ActionsHandler + Debug + Event + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(Console::<A>::with_lines(self.queued_entries.clone()));
        app.add_event::<A>();
        app.add_event::<SubmittedText>();
        app.add_systems(Startup, setup_console::<A>);
        app.add_systems(
            Update,
            (
                update_history::<A>,
                (
                    reset_did_close_flag::<A>,
                    close_shortcuts::<A>.run_if(|console: Res<Console<A>>| console.open),
                    (
                        get_keyboard_input::<A>,
                        update_input_text::<A>.run_if(needs_update::<A>),
                        handle_submitted_text::<A>,
                    )
                        .chain()
                        .run_if(|console: Res<Console<A>>| console.open),
                    open_shortcuts::<A>.run_if(|console: Res<Console<A>>| {
                        !console.open && !console.did_close_this_frame
                    }),
                    update_visibility::<A>,
                )
                    .chain(),
            ),
        );
    }
}

#[derive(Component)]
struct Root;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct History;

#[derive(Component)]
struct InputText;

fn setup_console<A>(
    mut commands: Commands,
    console: Res<Console<A>>,
    window: Query<&Window, With<PrimaryWindow>>,
) where
    A: Send + Sync + 'static,
{
    let window = window.single();

    let mut group = commands.spawn((
        Name::new("Console"),
        Root,
        NodeBundle {
            visibility: Visibility::Hidden,
            style: Style {
                width: Val::Percent(100.),
                height: Val::Px(
                    window.resolution.height()
                        * window.resolution.scale_factor() as f32
                        * console.expand_fraction,
                ),
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            // background_color: Color::PURPLE.into(),
            background_color: Color::INDIGO.into(),
            ..default()
        },
    ));

    group.with_children(|parent| {
        parent.spawn((
            Name::new("Input"),
            InputText,
            TextBundle {
                style: Style {
                    flex_grow: 0.,
                    min_height: Val::Px(console.font_size),
                    margin: UiRect::all(Val::Px(5.)),
                    ..default()
                },
                background_color: Color::BLACK.into(),
                text: Text::from_section(
                    "",
                    TextStyle {
                        color: Color::ANTIQUE_WHITE,
                        font_size: console.font_size,
                        ..default()
                    },
                ),
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..default()
            },
        ));

        parent.spawn((
            Name::new("History"),
            History,
            NodeBundle {
                // background_color: Color::INDIGO.into(),
                style: Style {
                    overflow: Overflow::clip_y(),
                    flex_direction: FlexDirection::Column,
                    align_content: AlignContent::FlexStart,
                    ..default()
                },
                ..default()
            },
        ));
    });
}

fn needs_update<A>(console: Res<Console<A>>) -> bool
where
    A: Send + Sync + 'static,
{
    console.input_did_update
}

fn update_input_text<A>(
    mut console: ResMut<Console<A>>,
    mut text_query: Query<&mut Text, With<InputText>>,
) where
    A: Send + Sync + 'static,
{
    console.input_did_update = false;
    let mut text = text_query.single_mut();
    text.sections[0].value = console.input.clone();
}

fn update_history<A>(
    mut commands: Commands,
    mut console: ResMut<Console<A>>,
    mut history_query: Query<Entity, With<History>>,
) where
    A: Send + Sync + 'static,
{
    // Mutex lock and remove all items from queued vec.
    let new_entries = {
        let mut queued_entries = console.queued_entries.0.lock().unwrap();
        std::mem::take(&mut *queued_entries)
    };

    if new_entries.is_empty() {
        return;
    }

    let history = history_query.single_mut();

    // For each new item, spawn a new entity with a Text component and add it to the children of the
    // history node.
    for entry in new_entries {
        let color = match entry.level {
            Level::TRACE => Color::GRAY,
            Level::DEBUG => Color::WHITE,
            Level::INFO => Color::LIME_GREEN,
            Level::WARN => Color::YELLOW,
            Level::ERROR => Color::ORANGE_RED,
        };

        let entity = commands
            .spawn((
                Name::new("HistoryItem"),
                TextBundle {
                    text: Text::from_section(
                        &entry.message,
                        TextStyle {
                            color,
                            font_size: console.font_size,
                            ..default()
                        },
                    ),
                    style: Style {
                        margin: UiRect::all(Val::Px(5.)),
                        ..default()
                    },
                    ..default()
                },
            ))
            .id();

        commands.entity(history).push_children(&[entity]);

        console.entity_entries.push_back(EntityEntry { entity });
    }

    // Check for older items that need to be removed from the history node. Remove them from the
    // children and destroy them.
    while console.entity_entries.len() > console.max_lines {
        if let Some(entry) = console.entity_entries.pop_front() {
            commands.entity(entry.entity).despawn_recursive();
        }
    }
}

#[derive(Event, Debug, Deref)]
struct SubmittedText(String);

fn get_keyboard_input<A>(
    mut console: ResMut<Console<A>>,
    mut key_events: EventReader<ReceivedCharacter>,
    mut submitted_text_writer: EventWriter<SubmittedText>,
) where
    A: Send + Sync + 'static,
{
    for key in key_events.read() {
        if key.char == '\r' {
            submitted_text_writer.send(SubmittedText(console.input.clone()));
            console.input.clear();
        } else if key.char == '\u{7F}' {
            console.input.pop();
        } else {
            console.input.push(key.char);
        }

        console.input_did_update = true;
    }
}

fn reset_did_close_flag<A>(mut console: ResMut<Console<A>>)
where
    A: Send + Sync + 'static,
{
    console.did_close_this_frame = false;
}

/// Run this before handling keyboard to close the console if it is open.
fn close_shortcuts<A>(mut console: ResMut<Console<A>>, keyboard_input: Res<Input<KeyCode>>)
where
    A: Send + Sync + 'static,
{
    let mut to_close = false;

    if let Some(key_code) = console.toggle_key_code {
        if keyboard_input.just_pressed(key_code) {
            to_close = true;
        }
    }

    if let Some(key_code) = console.close_key_code {
        if keyboard_input.just_pressed(key_code) {
            to_close = true;
        }
    }

    if to_close {
        console.close();
        console.did_close_this_frame = true;
    }
}

/// Run this after handling keyboard to open the console if it is closed.
fn open_shortcuts<A>(mut console: ResMut<Console<A>>, keyboard_input: Res<Input<KeyCode>>)
where
    A: Send + Sync + 'static,
{
    let mut to_open = false;

    if let Some(key_code) = console.toggle_key_code {
        if keyboard_input.just_pressed(key_code) {
            to_open = true;
        }
    }

    if let Some(key_code) = console.open_key_code {
        if keyboard_input.just_pressed(key_code) {
            to_open = true;
        }
    }

    if to_open {
        console.open();
    }
}

fn update_visibility<A>(
    mut console: ResMut<Console<A>>,
    mut query: Query<&mut Visibility, With<Root>>,
) where
    A: Send + Sync + 'static,
{
    if console.console_did_toggle {
        if console.open {
            *query.single_mut() = Visibility::Visible;
        } else {
            *query.single_mut() = Visibility::Hidden;
        }
        console.console_did_toggle = false;
    }
}

fn handle_submitted_text<A>(
    mut submitted_text_reader: EventReader<SubmittedText>,
    mut actions_writer: EventWriter<A>,
) where
    A: ActionsHandler + Event + Debug + Send + Sync + 'static,
{
    for text in submitted_text_reader.read() {
        info!("> {}", &**text);
        match A::resolve(&text.0) {
            Ok(action) => {
                actions_writer.send(action);
            }
            Err(e) => {
                error!("{e:#?}");
            }
        }
    }
}
