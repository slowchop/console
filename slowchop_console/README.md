# slowchop_console
A Quake style console and log plugin for Bevy.

## ⚠️ Warning ⚠️

This was rushed out for the Bevy Jam, and only tested on macOS. Not recommended to use, unless
you're willing to help fix it up. 😅

## Features
- Uses bevy_ui for rendering.
- A console that can be opened and closed with a keypress.
- Captures all bevy (tracing) log messages and displays them in the console.
- Colorizes log messages depending on log level.
- Uses an enum as the possible actions. ("Commands" IMO is a better term but that's taken by Bevy.)
  - Enum supports floats, integers, strings, bools, and Vec of those types.
  - `Optional` arguments work, which must be after any non-optional arguments.
  - Actions are validated against the enum.
- Actions executed are emitted as an event.

## Actions Enum

```rust
use bevy::prelude::Event;
use slowchop_console::Actions;

#[derive(Actions, Event)]
enum MyGameActions {
   Quit,
   Spawn(f32, f32, f32, Option<String>),
}
```

This will create a `quit` action that takes no arguments. The `spawn` action takes 3 floats, and
optionally a String.

Check out the [full example](examples/full.rs) on how to integrate.

License: MIT OR Apache-2.0
