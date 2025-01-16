use slowchop_console::{ActionsHandler, Error};
use slowchop_console_derive::Actions2;

#[test]
fn sub_enum() {
    #[derive(Actions2, Debug, PartialEq)]
    enum Main {
        A,
        B,
    }

    #[derive(Actions2)]
    enum Sub {
        C,
        D,
    }

    assert_eq!(Main::resolve(&mut "A").unwrap(), Main::A);
    // assert_eq!(Actions::resolve("A D")?, Actions::A(Sub::D));
    // assert_eq!(Actions::resolve("B")?, Actions::B);
}
