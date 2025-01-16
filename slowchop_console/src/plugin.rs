use crate::ActionsHandler;
use bevy::log::{BoxedSubscriber, Level};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use once_cell::sync::Lazy;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::sync::Mutex;
use tracing_subscriber::layer::SubscriberExt;

/// Unable to work out how to pass in an instance to LogPlugin update_subscriber, so using a global
/// static to give access to all QueuedEntries instances.
pub static QUEUED_ENTRIES: Lazy<QueuedEntries> = Lazy::new(QueuedEntries::default);

/// A container for entries that are queued up to be added to the console.
///
/// This was created specifically to use with the tracing subscriber.
#[derive(Debug, Default)]
pub struct QueuedEntries(pub(crate) Mutex<Vec<Entry>>);

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

    pub background_color: Color,
    pub input_background_color: Color,
    pub input_text_color: Color,
    pub error_text_color: Color,
    pub warn_text_color: Color,
    pub info_text_color: Color,
    pub debug_text_color: Color,
    pub trace_text_color: Color,

    /// Adjusts the z-index of the console.
    pub z_index: i32,

    pub font_size: f32,

    /// The console is open if this is true.
    ///
    /// use open(), toggle() or close() to change this.
    open: bool,

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
    phantom_data: std::marker::PhantomData<A>,
}

impl<A> Console<A> {
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

    fn take_queued_entries(&self) -> Vec<Entry> {
        let mut entries = QUEUED_ENTRIES.0.lock().unwrap();
        entries.drain(..).collect()
    }
}

impl<A> Default for Console<A> {
    fn default() -> Self {
        Console {
            input: "".to_string(),
            toggle_key_code: Some(KeyCode::Backquote),
            close_key_code: Some(KeyCode::Escape),
            open_key_code: None,
            entity_entries: Default::default(),
            max_lines: 100,
            font_size: 16.0,
            open: false,
            did_close_this_frame: false,
            expand_fraction: 0.8,
            input_did_update: true,
            console_did_toggle: true,
            phantom_data: Default::default(),
            z_index: 100,

            background_color: Color::hex("#0E181A").unwrap(),
            input_background_color: Color::hex("445055").unwrap(),
            input_text_color: Color::WHITE,
            error_text_color: Color::hex("FD564C").unwrap(),
            warn_text_color: Color::hex("FFE76A").unwrap(),
            info_text_color: Color::hex("81C6DC").unwrap(),
            debug_text_color: Color::hex("838A83").unwrap(),
            trace_text_color: Color::hex("445055").unwrap(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConsolePlugin<A>
where
    A: Send + Sync + 'static,
{
    phantom_data: std::marker::PhantomData<A>,
}

impl<A> ConsolePlugin<A>
where
    A: Send + Sync + ActionsHandler + Debug,
{
    pub fn update_subscriber(&self) -> fn(BoxedSubscriber) -> BoxedSubscriber {
        |subscriber| Box::new(subscriber.with(ConsolePlugin::<A>::default()))
    }
}

impl<A> Default for ConsolePlugin<A>
where
    A: Send + Sync + 'static,
{
    fn default() -> Self {
        Self {
            phantom_data: Default::default(),
        }
    }
}

impl<A> Plugin for ConsolePlugin<A>
where
    A: ActionsHandler + Debug + Event + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<Console<A>>();
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
struct History;

#[derive(Component)]
pub(crate) struct InputText;

fn setup_console<A>(
    mut commands: Commands,
    console: Res<Console<A>>,
    window: Query<&Window, With<PrimaryWindow>>,
) where
    A: Send + Sync + 'static,
{
    let Ok(window) = window.get_single() else {
        return;
    };

    let mut group = commands.spawn((
        Name::new("Console"),
        Root,
        NodeBundle {
            visibility: Visibility::Hidden,
            z_index: ZIndex::Global(console.z_index),
            style: Style {
                padding: UiRect::all(Val::Px(0.)),
                margin: UiRect::all(Val::Px(0.)),
                width: Val::Percent(100.),
                height: Val::Px(window.resolution.height() * console.expand_fraction),
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            background_color: console.background_color.into(),
            ..default()
        },
    ));

    group.with_children(|parent| {
        parent
            .spawn((
                Name::new("Input Container"),
                NodeBundle {
                    background_color: console.input_background_color.into(),
                    style: Style {
                        flex_grow: 0.,
                        padding: UiRect::new(Val::Px(10.), Val::Px(10.), Val::Px(2.), Val::Px(2.)),
                        margin: UiRect::top(Val::Px(7.)),
                        ..default()
                    },
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Input"),
                    InputText,
                    TextBundle {
                        style: Style {
                            min_height: Val::Px(console.font_size),
                            flex_grow: 0.,
                            padding: UiRect::all(Val::Px(0.)),
                            margin: UiRect::all(Val::Px(0.)),
                            ..default()
                        },
                        text: Text::from_section(
                            "",
                            TextStyle {
                                color: console.input_text_color,
                                font_size: console.font_size,
                                ..default()
                            },
                        ),
                        transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                        ..default()
                    },
                ));
            });

        parent.spawn((
            Name::new("History"),
            History,
            NodeBundle {
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
    let new_entries = console.take_queued_entries();

    if new_entries.is_empty() {
        return;
    }

    let history = history_query.single_mut();

    // For each new item, spawn a new entity with a Text component and add it to the children of the
    // history node.
    for entry in new_entries {
        let color = match entry.level {
            Level::TRACE => console.trace_text_color,
            Level::DEBUG => console.debug_text_color,
            Level::INFO => console.info_text_color,
            Level::WARN => console.warn_text_color,
            Level::ERROR => console.error_text_color,
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
                        margin: UiRect::new(Val::Px(10.), Val::Px(10.), Val::Px(0.), Val::Px(0.)),
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
        trace!(?key);
        match key.char.as_str() {
            // Enter
            "\r" => {
                submitted_text_writer.send(SubmittedText(console.input.clone()));
                console.input.clear();
            }
            // Backspace
            "\u{7F}" | "\u{8}" => {
                console.input.pop();
            }
            _ => {
                console.input += &key.char.to_string();
            }
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
fn close_shortcuts<A>(mut console: ResMut<Console<A>>, keyboard_input: Res<ButtonInput<KeyCode>>)
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
fn open_shortcuts<A>(mut console: ResMut<Console<A>>, keyboard_input: Res<ButtonInput<KeyCode>>)
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
        let mut visibility = query.single_mut();
        if console.open {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
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
        let mut text = text.to_string();
        info!("> {}", &text);
        match A::resolve(&mut text.as_str()) {
            Ok(action) => {
                actions_writer.send(action);
            }
            Err(e) => {
                error!("{e:#?}");
            }
        }
    }
}
