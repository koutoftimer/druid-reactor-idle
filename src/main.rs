use std::sync::{ Arc }; 

use druid::{ AppLauncher, WindowDesc, LocalizedString };

use dri::{ build_root_widget, AppState, Delegate, Grid };

const WINDOW_TITLE: LocalizedString<AppState> = LocalizedString::new("Hello World!");

fn main() {
    let main_window = WindowDesc::new(build_root_widget)
        .title(WINDOW_TITLE)
        .window_size((850.0, 700.0));

    let initial_state = AppState {
        grid: Grid::new(20, 20),
        balance: 100_f64,
        ticks: Arc::new(1_f64),
    };

    let launcher = AppLauncher::with_window(main_window);

    launcher
        .delegate(Delegate)
        .launch(initial_state)
        .expect("Failed to launch application");

}