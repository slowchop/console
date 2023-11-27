use crate::Error;
use bevy::ecs::system::SystemId;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy::window::PrimaryWindow;
use std::fmt::Debug;

pub trait Action {
    // fn new(&self) -> Box<dyn Action + Send + Sync>;
    fn name(&self) -> &'static str;
    fn parse(&mut self, s: &[String]) -> Result<(), Error>;
    // fn help(&self, args: &str) -> String {}
}

#[derive(Resource)]
pub struct Console {
    pub text: String,

    /// The console is open if this is true.
    pub open: bool,

    /// How far down the console will expand to, as a percentage of the screen height.
    /// 1.0 for expanding all the way down to the bottom. 0.5 for half way.
    pub expand_percentage: f32,

    actions: HashMap<String, SystemId>,
    needs_update: bool,
}

impl Console {
    pub fn register_action<A: Action + Event + Default + Send + Sync + 'static>(
        &mut self,
        commands: &mut Commands,
    ) {
        // self.actions.push(Box::new(A::default()));
        // commands.init_resource::<Events<A>>();
        todo!()
    }
}

impl Default for Console {
    fn default() -> Self {
        Console {
            text: "help".to_string(),
            open: false,
            expand_percentage: 0.5,
            actions: Default::default(),
            needs_update: true,
        }
    }
}

/// The console is a text box that can be used to enter commands.
/// It will draw a background rect (e.g. black or a texture).
/// It will have an input field at the bottom, with a blinking cursor.
/// It will draw text starting from above the input, and scrolling up.
/// The user can toggle the console with a key (e.g. tilde), but they control how that's done maybe
/// via an event.
#[derive(Default)]
pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Console>();
        app.add_event::<SubmittedText>();
        app.add_systems(Startup, (setup_console));
        app.add_systems(
            Update,
            (
                get_keyboard_input,
                update_text.run_if(needs_update),
                handle_submitted_text,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
struct Group;

#[derive(Component)]
struct Background;

#[derive(Component)]
struct InputText;

fn setup_console(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    console_state: Res<Console>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let mut group = commands.spawn((Name::new("Console"), Group, SpatialBundle::default()));

    let window = window.single();
    let res = &window.resolution;
    let custom_size = Vec2::new(res.width(), res.height() * console_state.expand_percentage);
    let custom_size = Some(custom_size);

    group.with_children(|parent| {
        parent.spawn((
            Name::new("Background"),
            Background,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.25, 0.25, 0.75),
                    custom_size,
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
                ..default()
            },
        ));

        parent.spawn((
            Name::new("Input"),
            InputText,
            Text2dBundle {
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

fn needs_update(console: Res<Console>) -> bool {
    console.needs_update
}

fn update_text(mut console: ResMut<Console>, mut text_query: Query<&mut Text, With<InputText>>) {
    console.needs_update = false;
    let mut text = text_query.single_mut();
    text.sections[0].value = console.text.clone();
}

#[derive(Event, Debug)]
struct SubmittedText(String);

fn get_keyboard_input(
    mut console: ResMut<Console>,
    mut key_events: EventReader<ReceivedCharacter>,
    mut submitted_text_writer: EventWriter<SubmittedText>,
) {
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

fn handle_submitted_text(
    mut commands: Commands,
    mut console: ResMut<Console>,
    mut submitted_text_reader: EventReader<SubmittedText>,
    mut action_writer: EventWriter<Box<dyn Action + Send + Sync>>,
) {
    for text in submitted_text_reader.read() {
        let Some(parts) = shlex::split(&text.0) else {
            error!("Could not parse text: {text:?}");
            continue;
        };
        let name = &parts[0];
        let args = &parts[1..];

        // for action in &mut console.actions {
        //     if action.name() != name {
        //         continue;
        //     }
        //
        //     if let Err(e) = action.parse(args) {
        //         error!("Could not parse args: {e:?}");
        //         return;
        //     }
        //
        //     action_writer.send(action);
        //
        //     return;
        // }

        commands.run_system(oneshot);

        error!("Could not find action: {name:?}");
    }
}