use crate::prelude::*;

pub(crate) fn setup_windows(mut commands: Commands) {
    commands.spawn((
        Window {
            title: "mlsmpm-particles-rs".to_string(),
            //WindowResolution::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT),
            mode: BorderlessFullscreen,
            ..Default::default()
        },
        PrimaryWindow,
    ));
}
