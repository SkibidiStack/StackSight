// On Windows release builds, don't show console window
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod components;
mod router;
mod services;
mod state;

#[cfg(windows)]
use devenv_backend::core::config::AppConfig;
#[cfg(windows)]
use devenv_backend::core::service_manager::ServiceManager;
use dioxus_desktop::{launch::launch, Config, WindowBuilder};

fn main() {
    init_tracing();

    // Start the backend right from the frontend application on Windows
    #[cfg(windows)]
    {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                if let Ok(config) = AppConfig::load("~/.config/devenv/config.toml").await {
                    if let Ok(mut manager) = ServiceManager::new(config).await {
                        if let Ok(_) = manager.start().await {
                            let _ = manager.run().await;
                        }
                    }
                }
            });
        });
    }

    let icon = load_icon();

    let window = WindowBuilder::new()
        .with_title("StackSight - DevEnv Manager")
        .with_resizable(true)
        .with_maximized(true);

    // Set the window icon if available
    let window = if let Some(icon) = icon {
        window.with_window_icon(Some(icon))
    } else {
        window
    };

    let cfg = Config::default().with_window(window);

    launch(app::AppRoot, vec![], vec![Box::new(cfg)]);
}

fn load_icon() -> Option<dioxus_desktop::tao::window::Icon> {
    let icon_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("icon.png");

    if !icon_path.exists() {
        tracing::warn!("Icon file not found at {:?}", icon_path);
        return None;
    }

    match image::open(&icon_path) {
        Ok(img) => {
            // Resize to common icon sizes for better compatibility
            // Many Linux DEs prefer 48x48, 64x64, or 128x128
            let img = img.resize_exact(128, 128, image::imageops::FilterType::Lanczos3);
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let raw_pixels = rgba.into_raw();

            tracing::info!(
                "Loading icon: {}x{}, {} bytes",
                width,
                height,
                raw_pixels.len()
            );

            match dioxus_desktop::tao::window::Icon::from_rgba(raw_pixels, width, height) {
                Ok(icon) => {
                    tracing::info!("Successfully loaded app icon ({}x{})", width, height);
                    Some(icon)
                }
                Err(e) => {
                    tracing::error!("Failed to create icon: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to load icon image: {}", e);
            None
        }
    }
}

fn init_tracing() {
    let env_filter =
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info,devenv_frontend=debug".to_string());

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .try_init();
}
