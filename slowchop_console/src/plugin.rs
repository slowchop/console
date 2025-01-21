use crate::render::{self, update_history, InputText, Root};
use crate::subscriber::{transfer_log_events, LogEvent};
use bevy::ecs::system::SystemId;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::utils::HashMap;
use shlex::Shlex;
use std::collections::VecDeque;
use std::fmt::Debug;

#[derive(Debug)]
pub(crate) struct EntityEntry {
    pub(crate) entity: Entity,
}

pub enum Function {
    Native(SystemId<In<FunctionArgs>>),
    Script(String),
}

#[derive(Resource)]
pub struct Console {
    functions: HashMap<String, Function>,

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

    /// How far down the console will expand to, as a percentage of the screen height.
    /// 1.0 for expanding all the way down to the bottom. 0.5 for half way.
    pub expand_fraction: f32,

    /// When the console changed from open to closed or vice versa.
    ///
    /// Triggers a visibility check.
    console_did_toggle: bool,

    /// When the input text changed.
    input_did_update: bool,

    pub(crate) entity_entries: VecDeque<EntityEntry>,
}

impl Debug for Console {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Console").finish()
    }
}

impl Console {
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

    pub fn add_system_command(&mut self, name: &str, system: SystemId<In<FunctionArgs>>) {
        self.functions
            .insert(name.to_string(), Function::Native(system));
    }

    pub fn run_system_command_in_world(&self, world: &mut World, command: &str) {
        println!("Running system command: {}", command);
        let system = match self.functions.get(command) {
            Some(Function::Native(system)) => system,
            _ => {
                error!("No system found with the name: {}", command);
                return;
            }
        };

        let args = FunctionArgs { args: vec![] };

        world.run_system_with_input(*system, args).unwrap();
    }

    pub fn run_system_command_in_commands(&self, commands: &mut Commands, command: &str) {
        let s: Vec<_> = Shlex::new(command).collect();
        let Some(command) = s.first() else {
            return;
        };

        let args = FunctionArgs {
            args: s
                .iter()
                .skip(1)
                .map(|x| SaneValue::String(x.to_string()))
                .collect(),
        };

        let system = match self.functions.get(command) {
            Some(Function::Native(system)) => system,
            _ => {
                error!("No system found with the name: {}", command);
                return;
            }
        };

        commands.run_system_with_input(*system, args);
    }
}

impl Default for Console {
    fn default() -> Self {
        Console {
            input: "".to_string(),
            functions: HashMap::default(),
            toggle_key_code: Some(KeyCode::Backquote),
            close_key_code: Some(KeyCode::Escape),
            open_key_code: None,
            entity_entries: Default::default(),
            max_lines: 100,
            font_size: 16.0,
            open: false,
            expand_fraction: 0.8,
            input_did_update: true,
            console_did_toggle: true,
            z_index: 100,

            background_color: Srgba::hex("#0E181A").unwrap().into(),
            input_background_color: Srgba::hex("#445055").unwrap().into(),
            input_text_color: Color::WHITE,
            error_text_color: Srgba::hex("#FD564C").unwrap().into(),
            warn_text_color: Srgba::hex("#FFE76A").unwrap().into(),
            info_text_color: Srgba::hex("#81C6DC").unwrap().into(),
            debug_text_color: Srgba::hex("#838A83").unwrap().into(),
            trace_text_color: Srgba::hex("#445055").unwrap().into(),
        }
    }
}

#[derive(Debug)]
pub struct FunctionArgs {
    pub args: Vec<SaneValue>,
}

#[derive(Debug)]
pub enum SaneValue {
    Bool(bool),
    String(String),
    Integer(i64),
}

#[derive(Clone, Debug, Default)]
pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Console>();
        app.add_event::<LogEvent>();
        app.add_event::<SubmittedText>();
        app.add_systems(Startup, render::setup_console);
        app.add_systems(
            Update,
            (
                update_history,
                transfer_log_events,
                (
                    (
                        update_input_text.run_if(needs_update),
                        get_keyboard_input,
                        handle_submitted_text,
                    )
                        .chain()
                        .run_if(|console: Res<Console>| console.open),
                    update_visibility,
                    handle_keyboard_shortcuts,
                )
                    .chain(),
            ),
        );
    }
}

#[derive(Event, Debug, Deref)]
struct SubmittedText(String);

fn needs_update(console: Res<Console>) -> bool
where
{
    console.input_did_update
}

fn update_input_text(
    mut console: ResMut<Console>,
    mut text_query: Query<&mut Text, With<InputText>>,
) {
    console.input_did_update = false;
    let mut text = text_query.single_mut();
    **text = console.input.clone();
}

fn get_keyboard_input(
    mut console: ResMut<Console>,
    mut key_events: EventReader<KeyboardInput>,
    mut submitted_text_writer: EventWriter<SubmittedText>,
) {
    for key in key_events.read() {
        if key.state == ButtonState::Released {
            continue;
        }

        match &key.logical_key {
            Key::Enter => {
                submitted_text_writer.send(SubmittedText(console.input.clone()));
                console.input.clear();
            }
            Key::Backspace => {
                console.input.pop();
            }
            Key::Space => {
                console.input += " ";
            }
            Key::Character(c) if c.as_str() == "`" => {
                warn_once!("Skipping backtick key for input for now. Programming is hard.");
            }
            Key::Character(c) => {
                console.input += &c.to_string();
            }
            _ => {}
        }

        console.input_did_update = true;
    }
}

/// Run this before handling keyboard to close the console if it is open.
fn handle_keyboard_shortcuts(
    mut console: ResMut<Console>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if let Some(key_code) = console.toggle_key_code {
        if keyboard_input.just_pressed(key_code) {
            if console.open {
                console.close();
                return;
            } else {
                console.open();
                return;
            }
        }
    }

    if let Some(key_code) = console.close_key_code {
        if keyboard_input.just_pressed(key_code) {
            console.close();
            return;
        }
    }

    if let Some(key_code) = console.open_key_code {
        if keyboard_input.just_pressed(key_code) {
            console.open();
            return;
        }
    }
}

fn update_visibility(mut console: ResMut<Console>, mut query: Query<&mut Visibility, With<Root>>) {
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

fn handle_submitted_text(
    mut commands: Commands,
    console: Res<Console>,
    mut submitted_text_reader: EventReader<SubmittedText>,
    // mut actions_writer: EventWriter<A>,
) {
    for text in submitted_text_reader.read() {
        info!("> {}", &**text);

        console.run_system_command_in_commands(&mut commands, &**text);
    }
}
