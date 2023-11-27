use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use slowchop_console::{Action, Actions, Console, ConsolePlugin, Error};

#[derive(Default, Event)]
struct Spawn(usize);

impl Action for Spawn {
    fn name(&self) -> &'static str {
        "spawn"
    }

    fn parse(&mut self, s: &[String]) -> Result<(), Error> {
        let arg = s.get(0).ok_or(Error::NotEnoughArguments {
            action: self.name().to_string(),
            args: s.to_vec(),
            expected: 1,
            given: 0,
        })?;

        let v = arg
            .parse::<usize>()
            .map_err(|e| Error::ParseIntError(arg.to_string(), e))?;

        self.0 = v;

        Ok(())
    }
}

#[derive(Default, Event, Debug)]
struct Help(String);

impl Action for Help {
    fn name(&self) -> &'static str {
        "help"
    }

    fn parse(&mut self, s: &[String]) -> Result<(), Error> {
        self.0 = s.join(" ");

        Ok(())
    }
}

#[derive(Default, Event)]
struct List;

impl Action for List {
    fn name(&self) -> &'static str {
        "list"
    }

    fn parse(&mut self, s: &[String]) -> Result<(), Error> {
        todo!()
    }
}

#[derive(Default)]
enum ColorChoices {
    #[default]
    AliceBlue,
    AntiqueWhite,
    Aqua,
}

#[derive(Default, Event)]
struct Color(ColorChoices);

impl Action for Color {
    fn name(&self) -> &'static str {
        "color"
    }

    fn parse(&mut self, s: &[String]) -> Result<(), Error> {
        let arg = s.get(0).ok_or(Error::NotEnoughArguments {
            action: self.name().to_string(),
            args: s.to_vec(),
            expected: 1,
            given: 0,
        })?;

        self.0 = match arg.as_str() {
            "aliceblue" => ColorChoices::AliceBlue,
            "antiquewhite" => ColorChoices::AntiqueWhite,
            "aqua" => ColorChoices::Aqua,
            _ => return Err(Error::BadArgument(0, arg.to_string())),
        };

        Ok(())
    }
}

pub fn main() {
    App::new()
        .add_plugins((DefaultPlugins, WorldInspectorPlugin::new(), ConsolePlugin))
        .add_systems(Startup, (setup_camera, setup_actions))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_actions(mut commands: Commands, mut console: ResMut<Console>) {
    console.register_action::<Help>(&mut commands);
    console.register_action::<List>(&mut commands);
    console.register_action::<Color>(&mut commands);
    console.register_action::<Spawn>(&mut commands);
}

fn on_help(mut events: EventReader<Help>) {
    for help in events.read() {
        println!("Help: {:?}", help);
    }
}
