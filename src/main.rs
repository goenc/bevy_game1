use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::{fs, io};

const WINDOW_REVEAL_DELAY_FRAMES: u8 = 3;
const DEFAULT_FONT_RELATIVE_PATH: &str = "fonts/NotoSansJP-Regular.ttf";
const DEFAULT_TEXT_CONFIG_RELATIVE_PATH: &str = "config/center_text.default.json";
const OVERRIDE_TEXT_CONFIG_FILE_NAME: &str = "center_text.override.json";
const APP_NAME: &str = env!("CARGO_PKG_NAME");

fn main() {
    let asset_root = resolve_asset_root();
    let required_font_path = asset_root.join(DEFAULT_FONT_RELATIVE_PATH);
    let default_text_config_path = asset_root.join(DEFAULT_TEXT_CONFIG_RELATIVE_PATH);

    if !required_font_path.is_file() {
        panic!(
            "Required bundled font is missing: {}",
            required_font_path.display()
        );
    }
    if !default_text_config_path.is_file() {
        panic!(
            "Required center text config is missing: {}",
            default_text_config_path.display()
        );
    }

    let loaded_center_text = load_center_text(&default_text_config_path).unwrap_or_else(|err| {
        panic!("Failed to load center text config: {err}");
    });

    App::new()
        .insert_resource(BundledFontPath(required_font_path))
        .insert_resource(CenterText(loaded_center_text.text))
        .insert_resource(CenterTextSource(loaded_center_text.source_path))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().to_string(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        visible: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, make_window_visible)
        .run();
}

#[derive(Resource)]
struct BundledFontPath(PathBuf);

#[derive(Resource)]
struct CenterText(String);

#[derive(Resource)]
struct CenterTextSource(PathBuf);

struct LoadedCenterText {
    text: String,
    source_path: PathBuf,
}

#[derive(Deserialize)]
struct CenterTextConfig {
    center_text: String,
}

fn resolve_asset_root() -> PathBuf {
    let mut candidates = vec![PathBuf::from("assets")];

    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        candidates.push(exe_dir.join("assets"));
        if let Some(parent_dir) = exe_dir.parent() {
            candidates.push(parent_dir.join("assets"));
        }
    }

    for candidate in candidates {
        if Path::new(&candidate).is_dir() {
            return candidate;
        }
    }

    PathBuf::from("assets")
}

fn resolve_override_config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|app_data| app_data.join(APP_NAME).join(OVERRIDE_TEXT_CONFIG_FILE_NAME))
}

fn load_center_text(default_config_path: &Path) -> io::Result<LoadedCenterText> {
    let source_path = resolve_override_config_path()
        .filter(|override_path| override_path.is_file())
        .unwrap_or_else(|| default_config_path.to_path_buf());
    let text = read_center_text_from_json(&source_path)?;
    Ok(LoadedCenterText { text, source_path })
}

fn read_center_text_from_json(path: &Path) -> io::Result<String> {
    let raw = fs::read_to_string(path)?;
    let config: CenterTextConfig = serde_json::from_str(&raw).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid center text JSON: {err}"),
        )
    })?;
    let text = config.center_text.trim().to_string();
    if text.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "center_text is empty",
        ));
    }
    Ok(text)
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    bundled_font_path: Res<BundledFontPath>,
    center_text: Res<CenterText>,
    center_text_source: Res<CenterTextSource>,
) {
    info!("Using bundled font: {}", bundled_font_path.0.display());
    let font = asset_server.load(DEFAULT_FONT_RELATIVE_PATH);
    info!("Using center text JSON: {}", center_text_source.0.display());

    commands.spawn(Camera2d);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.1)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(center_text.0.clone()),
                TextFont {
                    font,
                    font_size: 96.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn make_window_visible(mut windows: Query<&mut Window>, mut frame_counter: Local<u8>) {
    if *frame_counter < WINDOW_REVEAL_DELAY_FRAMES {
        *frame_counter += 1;
        return;
    }

    for mut window in &mut windows {
        if !window.visible {
            window.visible = true;
        }
    }
}
