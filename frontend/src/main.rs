mod app;
mod components;
mod router;
mod state;

use dioxus_desktop::{launch::launch, Config, WindowBuilder};

fn main() {
    init_tracing();

    let cfg = Config::default().with_window(
        WindowBuilder::new()
            .with_title("DevEnv Manager")
            .with_resizable(true)
            .with_maximized(true),
    );

    launch(app::AppRoot, vec![], vec![Box::new(cfg)]);
}

fn init_tracing() {
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,devenv_frontend=debug".to_string());

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .try_init();
}
